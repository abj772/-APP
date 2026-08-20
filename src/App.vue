<script setup lang="ts">
import { ref } from "vue";
import BillPage from "./pages/BillPage.vue";
import StatsPage from "./pages/StatsPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";

const page = ref<"bill" | "stats" | "settings">("bill");
const tabs = [
  { key: "bill" as const, icon: "📒", label: "账单" },
  { key: "stats" as const, icon: "📊", label: "统计" },
  { key: "settings" as const, icon: "⚙️", label: "设置" },
];
</script>

<template>
  <div class="app">
    <aside class="sidebar">
      <div class="logo">
        <span class="logo-icon">💰</span>
        <span class="logo-text">记记账</span>
      </div>
      <nav class="nav">
        <div
          v-for="t in tabs"
          :key="t.key"
          class="nav-item"
          :class="{ active: page === t.key }"
          @click="page = t.key"
        >
          <span class="nav-icon">{{ t.icon }}</span>
          <span class="nav-label">{{ t.label }}</span>
        </div>
      </nav>
      <div class="sidebar-footer">v0.1.0</div>
    </aside>
    <main class="content">
      <BillPage v-if="page === 'bill'" />
      <StatsPage v-else-if="page === 'stats'" />
      <SettingsPage v-else />
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100%;
}
.sidebar {
  width: 84px;
  flex-shrink: 0;
  background: #f6faf6;
  border-right: 1px solid #e3efe4;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 0 12px;
  user-select: none;
}
.logo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  margin-bottom: 24px;
}
.logo-icon {
  font-size: 26px;
}
.logo-text {
  font-size: 13px;
  font-weight: 600;
  color: #2e7d32;
}
.nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}
.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 10px 0;
  margin: 0 8px;
  border-radius: 10px;
  cursor: pointer;
  color: #6b7c6c;
  transition: all 0.15s;
}
.nav-item:hover {
  background: #edf7ee;
  color: #4caf50;
}
.nav-item.active {
  background: #e0f2e1;
  color: #2e7d32;
  font-weight: 600;
}
.nav-icon {
  font-size: 20px;
}
.nav-label {
  font-size: 12px;
}
.sidebar-footer {
  margin-top: auto;
  font-size: 11px;
  color: #b0beb0;
}
.content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: #fff;
}
</style>
