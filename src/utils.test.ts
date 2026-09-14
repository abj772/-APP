import { describe, it, expect } from "vitest";
import { formatYuan, todayStr, currentMonthStr, shiftMonth, monthLabel, dayLabel } from "./utils";

describe("formatYuan（分 → 元）", () => {
  it("常规金额换算", () => {
    expect(formatYuan(0)).toBe("0.00");
    expect(formatYuan(1)).toBe("0.01");
    expect(formatYuan(99)).toBe("0.99");
    expect(formatYuan(100)).toBe("1.00");
    expect(formatYuan(12345)).toBe("123.45");
    expect(formatYuan(123456)).toBe("1234.56");
    expect(formatYuan(-50)).toBe("-0.50");
  });
});

describe("todayStr（今天的日期）", () => {
  it("格式正确且与今天一致", () => {
    const s = todayStr();
    expect(s).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    const d = new Date();
    const expected = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
      d.getDate(),
    ).padStart(2, "0")}`;
    expect(s).toBe(expected);
  });
});

describe("currentMonthStr（当前月份）", () => {
  it("等于今天所属的年-月", () => {
    expect(currentMonthStr()).toBe(todayStr().slice(0, 7));
    expect(currentMonthStr()).toMatch(/^\d{4}-\d{2}$/);
  });
});

describe("shiftMonth（月份切换）", () => {
  it("普通加减", () => {
    expect(shiftMonth("2026-08", 0)).toBe("2026-08");
    expect(shiftMonth("2026-08", 1)).toBe("2026-09");
    expect(shiftMonth("2026-08", -1)).toBe("2026-07");
    expect(shiftMonth("2026-08", 12)).toBe("2027-08");
  });

  it("跨年边界", () => {
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
    expect(shiftMonth("2026-12", 1)).toBe("2027-01");
    expect(shiftMonth("2025-03", -4)).toBe("2024-11");
  });
});

describe("monthLabel（月份标题）", () => {
  it("显示成 中文 年月", () => {
    expect(monthLabel("2026-08")).toBe("2026年8月");
    expect(monthLabel("2026-12")).toBe("2026年12月");
  });
});

describe("dayLabel（账单日期分组标题）", () => {
  it("今天是『今天』", () => {
    expect(dayLabel(todayStr())).toBe("今天");
  });

  it("昨天是『昨天』", () => {
    const y = new Date();
    y.setDate(y.getDate() - 1);
    const yesterday = `${y.getFullYear()}-${String(y.getMonth() + 1).padStart(2, "0")}-${String(
      y.getDate(),
    ).padStart(2, "0")}`;
    expect(dayLabel(yesterday)).toBe("昨天");
  });

  it("已过去的固定日期显示『M月D日 星期X』", () => {
    // 选两个已经过去的日期，永远不可能是今天/昨天
    expect(dayLabel("2026-03-15")).toBe("3月15日 周日");
    expect(dayLabel("2025-12-25")).toBe("12月25日 周四");
  });

  it("任意较早日期显示『M月D日 星期X』", () => {
    // 取 40 天前的日期（保证不是今天/昨天），用独立算法算出期望标题
    const base = new Date();
    base.setDate(base.getDate() - 40);
    const ds = `${base.getFullYear()}-${String(base.getMonth() + 1).padStart(2, "0")}-${String(
      base.getDate(),
    ).padStart(2, "0")}`;
    const WEEK = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
    const wd = WEEK[new Date(ds + "T00:00:00").getDay()];
    const [, m, d] = ds.split("-").map(Number);
    expect(dayLabel(ds)).toBe(`${m}月${d}日 ${wd}`);
  });
});
