<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { listExpenses, getCategories, deleteExpense, type ExpenseRow, type Category } from "../api";
import { currentMonthStr, shiftMonth, monthLabel, dayLabel, formatYuan } from "../utils";
import ExpenseDialog from "../components/ExpenseDialog.vue";

const month = ref(currentMonthStr());
const keyword = ref("");
const categoryId = ref<number | null>(null);
const rows = ref<ExpenseRow[]>([]);
const categories = ref<Category[]>([]);
const loading = ref(false);

const totalCents = computed(() => rows.value.reduce((s, r) => s + r.amount_cents, 0));

/** 分类筛选选项（两级） */
const filterOptions = computed(() => {
  const majors = categories.value.filter((c) => c.parent_id === null);
  return majors.map((m) => ({
    value: m.id,
    label: `${m.emoji} ${m.name}`,
    children: categories.value
      .filter((c) => c.parent_id === m.id)
      .map((c) => ({ value: c.id, label: c.name })),
  }));
});

/** 按日期分组 */
const groups = computed(() => {
  const map = new Map<string, ExpenseRow[]>();
  for (const r of rows.value) {
    if (!map.has(r.date)) map.set(r.date, []);
    map.get(r.date)!.push(r);
  }
  return Array.from(map.entries()).map(([date, items]) => ({
    date,
    label: dayLabel(date),
    sum: items.reduce((s, i) => s + i.amount_cents, 0),
    items,
  }));
});

async function load() {
  loading.value = true;
  try {
    rows.value = await listExpenses({
      month: month.value,
      keyword: keyword.value.trim() || null,
      categoryId: categoryId.value,
    });
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
  }
}

async function loadCategories() {
  try {
    categories.value = await getCategories();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

onMounted(() => {
  loadCategories();
  load();
});

watch([month, categoryId], load);

let kwTimer: ReturnType<typeof setTimeout> | undefined;
watch(keyword, () => {
  clearTimeout(kwTimer);
  kwTimer = setTimeout(load, 300);
});

function prevMonth() {
  month.value = shiftMonth(month.value, -1);
}
function nextMonth() {
  if (month.value < currentMonthStr()) month.value = shiftMonth(month.value, 1);
}

// ===== 记一笔 / 编辑 =====
const dialogVisible = ref(false);
const editing = ref<ExpenseRow | null>(null);

function openAdd() {
  editing.value = null;
  dialogVisible.value = true;
}
function openEdit(row: ExpenseRow) {
  editing.value = row;
  dialogVisible.value = true;
}
function onSaved() {
  dialogVisible.value = false;
  load();
}

// ===== 删除 =====
async function remove(row: ExpenseRow) {
  try {
    await ElMessageBox.confirm(
      `确定删除这笔「${row.category_name} ¥${formatYuan(row.amount_cents)}」吗？`,
      "删除确认",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }
  try {
    await deleteExpense(row.id);
    ElMessage.success("已删除");
    load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}
</script>

<template>
  <div class="bill-page">
    <header class="page-header">
      <div class="month-nav">
        <button class="month-btn" @click="prevMonth">‹</button>
        <span class="month-label">{{ monthLabel(month) }}</span>
        <button class="month-btn" :disabled="month >= currentMonthStr()" @click="nextMonth">›</button>
      </div>
      <div class="month-total">
        <div class="total-label">本月支出</div>
        <div class="total-value">¥{{ formatYuan(totalCents) }}</div>
      </div>
    </header>

    <div class="filters">
      <el-input v-model="keyword" placeholder="搜索备注…" clearable size="small" class="kw" />
      <el-cascader
        v-model="categoryId"
        :options="filterOptions"
        :props="{ checkStrictly: true }"
        placeholder="全部分类"
        clearable
        size="small"
        class="cat"
      />
    </div>

    <div v-loading="loading" class="list">
      <template v-if="groups.length">
        <section v-for="g in groups" :key="g.date" class="day-group">
          <div class="day-header">
            <span>{{ g.label }}</span>
            <span class="day-sum">¥{{ formatYuan(g.sum) }}</span>
          </div>
          <div v-for="r in g.items" :key="r.id" class="expense-row" @click="openEdit(r)">
            <span class="row-emoji">{{ r.category_emoji }}</span>
            <div class="row-mid">
              <div class="row-title">{{ r.category_name }}</div>
              <div class="row-sub">
                {{ r.parent_name }}<template v-if="r.note"> · {{ r.note }}</template>
              </div>
            </div>
            <div class="row-right">
              <span class="row-amount">-¥{{ formatYuan(r.amount_cents) }}</span>
              <el-button text size="small" class="row-del" @click.stop="remove(r)">✕</el-button>
            </div>
          </div>
        </section>
      </template>
      <div v-else class="empty">
        <div class="empty-icon">🌱</div>
        <p>这个月还没有账单</p>
        <p>点右下角 ＋ 记第一笔吧</p>
      </div>
    </div>

    <button class="fab" @click="openAdd">＋</button>

    <ExpenseDialog
      v-model="dialogVisible"
      :editing="editing"
      :categories="categories"
      @saved="onSaved"
    />
  </div>
</template>

<style scoped>
.bill-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}
.page-header {
  padding: 14px 14px 10px;
}
.month-nav {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 10px;
}
.month-btn {
  width: 26px;
  height: 26px;
  border: none;
  background: #edf7ee;
  color: #2e7d32;
  border-radius: 50%;
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
}
.month-btn:disabled {
  opacity: 0.35;
  cursor: default;
}
.month-label {
  font-size: 14px;
  font-weight: 600;
  color: #37474f;
  min-width: 90px;
  text-align: center;
}
.month-total {
  background: linear-gradient(135deg, #4caf50, #81c784);
  border-radius: 14px;
  padding: 14px 16px;
  color: #fff;
}
.total-label {
  font-size: 12px;
  opacity: 0.9;
}
.total-value {
  font-size: 26px;
  font-weight: 700;
  margin-top: 2px;
}
.filters {
  display: flex;
  gap: 8px;
  padding: 10px 14px 6px;
}
.filters .kw {
  flex: 1;
}
.filters .cat {
  width: 130px;
}
.list {
  flex: 1;
  overflow-y: auto;
  padding: 0 10px 70px;
}
.day-group {
  margin-top: 10px;
}
.day-header {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #90a4ae;
  padding: 4px 6px;
}
.day-sum {
  color: #78909c;
}
.expense-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 6px;
  border-radius: 10px;
  cursor: pointer;
}
.expense-row:hover {
  background: #f6faf6;
}
.row-emoji {
  width: 36px;
  height: 36px;
  background: #edf7ee;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  flex-shrink: 0;
}
.row-mid {
  flex: 1;
  min-width: 0;
}
.row-title {
  font-size: 13px;
  color: #37474f;
}
.row-sub {
  font-size: 11px;
  color: #b0bec5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}
.row-right {
  display: flex;
  align-items: center;
  gap: 2px;
}
.row-amount {
  font-size: 14px;
  font-weight: 600;
  color: #37474f;
}
.row-del {
  color: #cfd8dc;
  padding: 0 4px;
}
.row-del:hover {
  color: #e57373;
}
.empty {
  text-align: center;
  color: #b0bec5;
  padding-top: 90px;
  font-size: 13px;
  line-height: 1.8;
}
.empty-icon {
  font-size: 40px;
  margin-bottom: 8px;
}
.fab {
  position: absolute;
  right: 18px;
  bottom: 18px;
  width: 52px;
  height: 52px;
  border-radius: 50%;
  border: none;
  background: #4caf50;
  color: #fff;
  font-size: 26px;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(76, 175, 80, 0.4);
  transition: transform 0.1s;
  z-index: 5;
}
.fab:hover {
  transform: scale(1.06);
}
</style>
