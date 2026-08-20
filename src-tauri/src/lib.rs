use rusqlite::{params, Connection};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

/// 数据库连接（互斥锁包裹，保证多线程下安全）
struct Db(Mutex<Connection>);

#[derive(Serialize, Clone)]
struct Category {
    id: i64,
    name: String,
    parent_id: Option<i64>,
    emoji: String,
}

#[derive(Serialize, Clone)]
struct ExpenseRow {
    id: i64,
    amount_cents: i64,
    date: String,
    note: String,
    category_id: i64,
    category_name: String,
    category_emoji: String,
    parent_id: i64,
    parent_name: String,
    parent_emoji: String,
}

#[derive(Serialize, Clone)]
struct CategoryStat {
    category_id: i64,
    name: String,
    emoji: String,
    total_cents: i64,
}

#[derive(Serialize, Clone)]
struct DailyStat {
    day: String,
    total_cents: i64,
}

#[derive(Serialize, Clone)]
struct MonthSummary {
    total_cents: i64,
    count: i64,
    by_category: Vec<CategoryStat>,
    daily: Vec<DailyStat>,
}

/// 内置默认分类：一级大类（带图标）＋ 二级小类
const DEFAULT_CATEGORIES: &[(&str, &str, &[&str])] = &[
    ("餐饮饮食", "🍚", &["早餐", "午餐", "晚餐", "外卖", "零食饮料", "水果蔬菜", "聚餐"]),
    ("交通出行", "🚗", &["公交地铁", "出租网约车", "火车高铁", "飞机", "加油充电", "停车过路", "维修保养"]),
    ("购物消费", "🛒", &["服饰鞋包", "日用品", "数码家电", "美妆个护", "图书文具"]),
    ("居住生活", "🏠", &["房租房贷", "水电燃气", "物业费", "宽带话费", "家居用品", "维修装修"]),
    ("休闲娱乐", "🎮", &["电影演出", "游戏充值", "旅游度假", "运动健身", "宠物开销"]),
    ("医疗健康", "🏥", &["门诊买药", "住院体检", "保健营养"]),
    ("学习教育", "📚", &["学费培训", "考试报名", "学习用品"]),
    ("人情往来", "🎁", &["红包礼物", "请客送礼", "孝敬长辈"]),
    ("金融保险", "💰", &["保险缴费", "贷款还款", "手续费"]),
    ("其他", "📦", &["其他"]),
];

fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_id INTEGER,
            emoji TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            amount_cents INTEGER NOT NULL,
            category_id INTEGER NOT NULL REFERENCES categories(id),
            date TEXT NOT NULL,
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(date);",
    )
    .map_err(|e| e.to_string())?;

    // 首次启动时写入默认分类
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count == 0 {
        for (i, (name, emoji, subs)) in DEFAULT_CATEGORIES.iter().enumerate() {
            conn.execute(
                "INSERT INTO categories (name, parent_id, emoji, sort_order) VALUES (?1, NULL, ?2, ?3)",
                params![name, emoji, i as i64],
            )
            .map_err(|e| e.to_string())?;
            let parent_id = conn.last_insert_rowid();
            for sub in *subs {
                conn.execute(
                    "INSERT INTO categories (name, parent_id, emoji, sort_order) VALUES (?1, ?2, '', 0)",
                    params![sub, parent_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// 简易日期校验：YYYY-MM-DD
fn is_valid_date(date: &str) -> bool {
    date.len() == 10
        && date.as_bytes().get(4) == Some(&b'-')
        && date.as_bytes().get(7) == Some(&b'-')
        && date.chars().all(|c| c.is_ascii_digit() || c == '-')
}

// ==================== 分类 ====================

#[tauri::command]
fn get_categories(db: State<Db>) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, emoji FROM categories ORDER BY sort_order, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                parent_id: r.get(2)?,
                emoji: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
fn add_category(
    db: State<Db>,
    name: String,
    parent_id: Option<i64>,
    emoji: Option<String>,
) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO categories (name, parent_id, emoji) VALUES (?1, ?2, ?3)",
        params![name, parent_id, emoji.unwrap_or_default()],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn update_category(
    db: State<Db>,
    id: i64,
    name: String,
    emoji: Option<String>,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE categories SET name = ?1, emoji = ?2 WHERE id = ?3",
        params![name, emoji.unwrap_or_default(), id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_category(db: State<Db>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE parent_id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if child_count > 0 {
        return Err("该大类下还有小分类，请先删除小分类".into());
    }
    let used: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM expenses WHERE category_id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if used > 0 {
        return Err("该分类已有账单记录，不能删除".into());
    }
    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 账单 ====================

#[tauri::command]
fn add_expense(
    db: State<Db>,
    amount_cents: i64,
    category_id: i64,
    date: String,
    note: String,
) -> Result<i64, String> {
    if amount_cents <= 0 {
        return Err("金额必须大于 0".into());
    }
    if !is_valid_date(&date) {
        return Err("日期格式不正确".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO expenses (amount_cents, category_id, date, note) VALUES (?1, ?2, ?3, ?4)",
        params![amount_cents, category_id, date, note],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn update_expense(
    db: State<Db>,
    id: i64,
    amount_cents: i64,
    category_id: i64,
    date: String,
    note: String,
) -> Result<(), String> {
    if amount_cents <= 0 {
        return Err("金额必须大于 0".into());
    }
    if !is_valid_date(&date) {
        return Err("日期格式不正确".into());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE expenses SET amount_cents = ?1, category_id = ?2, date = ?3, note = ?4 WHERE id = ?5",
        params![amount_cents, category_id, date, note, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_expense(db: State<Db>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM expenses WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn list_expenses(
    db: State<Db>,
    month: Option<String>,
    keyword: Option<String>,
    category_id: Option<i64>,
) -> Result<Vec<ExpenseRow>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = "SELECT e.id, e.amount_cents, e.date, e.note, e.category_id,
            c.name AS category_name,
            COALESCE(NULLIF(c.emoji, ''), COALESCE(p.emoji, '📦')) AS category_emoji,
            COALESCE(p.id, c.id) AS parent_id,
            COALESCE(p.name, c.name) AS parent_name,
            COALESCE(NULLIF(p.emoji, ''), COALESCE(c.emoji, '📦')) AS parent_emoji
        FROM expenses e
        JOIN categories c ON c.id = e.category_id
        LEFT JOIN categories p ON p.id = c.parent_id
        WHERE (?1 IS NULL OR e.date LIKE ?1 || '%')
          AND (?2 IS NULL OR e.note LIKE '%' || ?2 || '%')
          AND (?3 IS NULL OR c.id = ?3 OR c.parent_id = ?3)
        ORDER BY e.date DESC, e.id DESC";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let keyword = keyword.filter(|k| !k.trim().is_empty());
    let rows = stmt
        .query_map(params![month, keyword, category_id], |r| {
            Ok(ExpenseRow {
                id: r.get(0)?,
                amount_cents: r.get(1)?,
                date: r.get(2)?,
                note: r.get(3)?,
                category_id: r.get(4)?,
                category_name: r.get(5)?,
                category_emoji: r.get(6)?,
                parent_id: r.get(7)?,
                parent_name: r.get(8)?,
                parent_emoji: r.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

// ==================== 统计 ====================

#[tauri::command]
fn get_month_summary(db: State<Db>, month: String) -> Result<MonthSummary, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let (total_cents, count): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_cents), 0), COUNT(*) FROM expenses WHERE date LIKE ?1 || '%'",
            params![&month],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(p.id, c.id) AS cid,
                    COALESCE(p.name, c.name) AS cname,
                    COALESCE(NULLIF(p.emoji, ''), COALESCE(c.emoji, '📦')) AS cemoji,
                    SUM(e.amount_cents) AS total
             FROM expenses e
             JOIN categories c ON c.id = e.category_id
             LEFT JOIN categories p ON p.id = c.parent_id
             WHERE e.date LIKE ?1 || '%'
             GROUP BY cid
             ORDER BY total DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![&month], |r| {
            Ok(CategoryStat {
                category_id: r.get(0)?,
                name: r.get(1)?,
                emoji: r.get(2)?,
                total_cents: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut by_category = Vec::new();
    for row in rows {
        by_category.push(row.map_err(|e| e.to_string())?);
    }

    let mut stmt = conn
        .prepare(
            "SELECT substr(date, 9, 2) AS day, SUM(amount_cents) AS total
             FROM expenses WHERE date LIKE ?1 || '%'
             GROUP BY date ORDER BY date",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![&month], |r| {
            Ok(DailyStat {
                day: r.get(0)?,
                total_cents: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut daily = Vec::new();
    for row in rows {
        daily.push(row.map_err(|e| e.to_string())?);
    }

    Ok(MonthSummary {
        total_cents,
        count,
        by_category,
        daily,
    })
}

#[tauri::command]
fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("jijizhang.db").to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = Connection::open(dir.join("jijizhang.db"))?;
            init_db(&conn)?;
            app.manage(Db(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_categories,
            add_category,
            update_category,
            delete_category,
            add_expense,
            update_expense,
            delete_expense,
            list_expenses,
            get_month_summary,
            get_db_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
