import { invoke } from "@tauri-apps/api/core";

// ============ 数据类型 ============

export interface Category {
  id: number;
  name: string;
  parent_id: number | null;
  emoji: string;
}

export interface ExpenseRow {
  id: number;
  amount_cents: number;
  date: string;
  note: string;
  category_id: number;
  category_name: string;
  category_emoji: string;
  parent_id: number;
  parent_name: string;
  parent_emoji: string;
}

export interface CategoryStat {
  category_id: number;
  name: string;
  emoji: string;
  total_cents: number;
}

export interface DailyStat {
  day: string;
  total_cents: number;
}

export interface MonthSummary {
  total_cents: number;
  count: number;
  by_category: CategoryStat[];
  daily: DailyStat[];
}

export type ExpenseInput = {
  amountCents: number;
  categoryId: number;
  date: string;
  note: string;
};

// ============ 分类 ============

export const getCategories = () => invoke<Category[]>("get_categories");
export const addCategory = (name: string, parentId: number | null, emoji: string) =>
  invoke<number>("add_category", { name, parentId, emoji });
export const updateCategory = (id: number, name: string, emoji: string) =>
  invoke<void>("update_category", { id, name, emoji });
export const deleteCategory = (id: number) => invoke<void>("delete_category", { id });

// ============ 账单 ============

export const addExpense = (payload: ExpenseInput) => invoke<number>("add_expense", payload);
export const updateExpense = (id: number, payload: ExpenseInput) =>
  invoke<void>("update_expense", { id, ...payload });
export const deleteExpense = (id: number) => invoke<void>("delete_expense", { id });

export const listExpenses = (filter: {
  month: string | null;
  keyword: string | null;
  categoryId: number | null;
}) => invoke<ExpenseRow[]>("list_expenses", filter);

// ============ 统计 ============

export const getMonthSummary = (month: string) =>
  invoke<MonthSummary>("get_month_summary", { month });

// ============ 其他 ============

export const getDbPath = () => invoke<string>("get_db_path");
