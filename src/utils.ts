/** 分 → 元 字符串 */
export function formatYuan(cents: number): string {
  return (cents / 100).toFixed(2);
}

export function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate(),
  ).padStart(2, "0")}`;
}

export function currentMonthStr(): string {
  return todayStr().slice(0, 7);
}

export function shiftMonth(month: string, delta: number): string {
  const [y, m] = month.split("-").map(Number);
  const d = new Date(y, m - 1 + delta, 1);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

export function monthLabel(month: string): string {
  const [y, m] = month.split("-").map(Number);
  return `${y}年${m}月`;
}

function shiftDay(dateStr: string, delta: number): string {
  const [y, m, d] = dateStr.split("-").map(Number);
  const dt = new Date(y, m - 1, d + delta);
  return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, "0")}-${String(
    dt.getDate(),
  ).padStart(2, "0")}`;
}

const WEEK = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

/** 日期分组标题：今天 / 昨天 / M月D日 星期X */
export function dayLabel(dateStr: string): string {
  const today = todayStr();
  if (dateStr === today) return "今天";
  if (dateStr === shiftDay(today, -1)) return "昨天";
  const [, m, d] = dateStr.split("-").map(Number);
  const wd = new Date(dateStr + "T00:00:00").getDay();
  return `${m}月${d}日 ${WEEK[wd]}`;
}
