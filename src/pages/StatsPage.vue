<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as echarts from "echarts";
import { ElMessage } from "element-plus";
import { getMonthSummary, type MonthSummary } from "../api";
import { currentMonthStr, shiftMonth, monthLabel, formatYuan } from "../utils";

const month = ref(currentMonthStr());
const summary = ref<MonthSummary | null>(null);
const pieRef = ref<HTMLDivElement>();
const barRef = ref<HTMLDivElement>();
let pieChart: echarts.ECharts | null = null;
let barChart: echarts.ECharts | null = null;

/** 图表配色：首色为品牌绿，其余柔和色 */
const PALETTE = [
  "#4caf50", "#ffb74d", "#4fc3f7", "#f06292", "#ba68c8",
  "#a1887f", "#ff8a65", "#9575cd", "#4db6ac", "#dce775",
];

const daysInMonth = computed(() => {
  const [y, m] = month.value.split("-").map(Number);
  return new Date(y, m, 0).getDate();
});

const avgCents = computed(() =>
  summary.value ? summary.value.total_cents / daysInMonth.value : 0,
);

function renderCharts() {
  pieChart?.dispose();
  barChart?.dispose();
  pieChart = null;
  barChart = null;
  if (!summary.value) return;

  // 分类占比饼图
  if (pieRef.value && summary.value.by_category.length) {
    pieChart = echarts.init(pieRef.value);
    pieChart.setOption({
      color: PALETTE,
      tooltip: {
        trigger: "item",
        valueFormatter: (v: any) => "¥" + Number(v).toFixed(2),
      },
      legend: {
        type: "scroll",
        orient: "vertical",
        right: 0,
        top: "middle",
        itemWidth: 10,
        itemHeight: 10,
        textStyle: { fontSize: 11 },
      },
      series: [
        {
          type: "pie",
          radius: ["45%", "72%"],
          center: ["38%", "50%"],
          data: summary.value.by_category.map((c) => ({
            name: `${c.emoji} ${c.name}`,
            value: +(c.total_cents / 100).toFixed(2),
          })),
          label: { show: false },
        },
      ],
    });
  }

  // 每日支出柱状图
  if (barRef.value) {
    const data = Array.from({ length: daysInMonth.value }, (_, i) => {
      const d = String(i + 1).padStart(2, "0");
      const hit = summary.value!.daily.find((x) => x.day === d);
      return hit ? +(hit.total_cents / 100).toFixed(2) : 0;
    });
    barChart = echarts.init(barRef.value);
    barChart.setOption({
      grid: { left: 44, right: 10, top: 16, bottom: 22 },
      tooltip: {
        trigger: "axis",
        valueFormatter: (v: any) => "¥" + Number(v).toFixed(2),
      },
      xAxis: {
        type: "category",
        data: data.map((_, i) => i + 1),
        axisLabel: { fontSize: 9, interval: 4 },
        axisTick: { show: false },
      },
      yAxis: {
        type: "value",
        axisLabel: { fontSize: 9 },
        splitLine: { lineStyle: { color: "#f0f3f0" } },
      },
      series: [
        {
          type: "bar",
          data,
          itemStyle: { color: "#66bb6a", borderRadius: [3, 3, 0, 0] },
          barMaxWidth: 10,
        },
      ],
    });
  }
}

async function load() {
  try {
    summary.value = await getMonthSummary(month.value);
    await nextTick();
    renderCharts();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

function prevMonth() {
  month.value = shiftMonth(month.value, -1);
}
function nextMonth() {
  if (month.value < currentMonthStr()) month.value = shiftMonth(month.value, 1);
}

onMounted(load);
watch(month, load);
onBeforeUnmount(() => {
  pieChart?.dispose();
  barChart?.dispose();
});
</script>

<template>
  <div class="stats-page">
    <header class="stats-header">
      <button class="month-btn" @click="prevMonth">‹</button>
      <span class="month-label">{{ monthLabel(month) }}</span>
      <button class="month-btn" :disabled="month >= currentMonthStr()" @click="nextMonth">›</button>
    </header>

    <div class="cards">
      <div class="card">
        <div class="card-label">总支出</div>
        <div class="card-value">¥{{ formatYuan(summary?.total_cents ?? 0) }}</div>
      </div>
      <div class="card">
        <div class="card-label">笔数</div>
        <div class="card-value small">{{ summary?.count ?? 0 }} 笔</div>
      </div>
      <div class="card">
        <div class="card-label">日均</div>
        <div class="card-value small">¥{{ formatYuan(avgCents) }}</div>
      </div>
    </div>

    <section class="chart-card">
      <h3>分类占比</h3>
      <div v-if="summary && summary.by_category.length" ref="pieRef" class="chart pie"></div>
      <el-empty v-else description="本月还没有数据" :image-size="60" />
    </section>

    <section class="chart-card">
      <h3>每日支出</h3>
      <div ref="barRef" class="chart bar"></div>
    </section>
  </div>
</template>

<style scoped>
.stats-page {
  height: 100%;
  overflow-y: auto;
  padding: 14px;
}
.stats-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 12px;
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
.cards {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.card {
  flex: 1;
  background: #f6faf6;
  border: 1px solid #e3efe4;
  border-radius: 12px;
  padding: 10px;
  text-align: center;
}
.card-label {
  font-size: 11px;
  color: #90a4ae;
  margin-bottom: 4px;
}
.card-value {
  font-size: 15px;
  font-weight: 700;
  color: #2e7d32;
  white-space: nowrap;
}
.card-value.small {
  font-size: 13px;
}
.chart-card {
  background: #fbfdfb;
  border: 1px solid #eef4ee;
  border-radius: 12px;
  padding: 12px;
  margin-bottom: 12px;
}
.chart-card h3 {
  font-size: 13px;
  color: #37474f;
  margin-bottom: 6px;
}
.chart.pie {
  height: 190px;
}
.chart.bar {
  height: 170px;
}
</style>
