<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import {
  getCategories, addCategory, updateCategory, deleteCategory, getDbPath,
  type Category,
} from "../api";

const categories = ref<Category[]>([]);
const dbPath = ref("");

const majors = computed(() => categories.value.filter((c) => c.parent_id === null));
const subsOf = (majorId: number) => categories.value.filter((c) => c.parent_id === majorId);

interface CategoryDialog {
  visible: boolean;
  mode: "add" | "edit";
  id: number | null;
  parentId: number | null;
  name: string;
  emoji: string;
}
const dialog = ref<CategoryDialog>({
  visible: false,
  mode: "add",
  id: null,
  parentId: null,
  name: "",
  emoji: "",
});

async function load() {
  try {
    categories.value = await getCategories();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

onMounted(async () => {
  await load();
  try {
    dbPath.value = await getDbPath();
  } catch {
    /* 拿不到路径也不影响使用 */
  }
});

function openAddMajor() {
  dialog.value = { visible: true, mode: "add", id: null, parentId: null, name: "", emoji: "" };
}
function openAddSub(parent: Category) {
  dialog.value = { visible: true, mode: "add", id: null, parentId: parent.id, name: "", emoji: "" };
}
function openEdit(cat: Category) {
  dialog.value = {
    visible: true, mode: "edit", id: cat.id, parentId: cat.parent_id,
    name: cat.name, emoji: cat.emoji,
  };
}

async function saveDialog() {
  const { mode, id, parentId, name, emoji } = dialog.value;
  if (!name.trim()) {
    ElMessage.warning("名称不能为空");
    return;
  }
  try {
    if (mode === "add") {
      await addCategory(name.trim(), parentId, emoji.trim());
    } else {
      await updateCategory(id!, name.trim(), emoji.trim());
    }
    ElMessage.success(mode === "add" ? "已添加" : "已保存");
    dialog.value.visible = false;
    load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function remove(cat: Category) {
  try {
    await ElMessageBox.confirm(`确定删除「${cat.name}」吗？`, "删除分类", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
  } catch {
    return;
  }
  try {
    await deleteCategory(cat.id);
    ElMessage.success("已删除");
    load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}
</script>

<template>
  <div class="settings-page">
    <h2>分类管理</h2>
    <p class="hint">两类级分类都可以在这里增删改（有账单记录的分类不能删除）</p>

    <el-collapse class="cat-tree">
      <el-collapse-item v-for="m in majors" :key="m.id">
        <template #title>
          <div class="major-title">
            <span>{{ m.emoji }} {{ m.name }}</span>
            <span class="sub-count">{{ subsOf(m.id).length }} 个小类</span>
          </div>
        </template>
        <div class="subs">
          <div v-for="s in subsOf(m.id)" :key="s.id" class="sub-row">
            <span class="sub-name">{{ s.name }}</span>
            <span class="sub-actions">
              <el-button text size="small" @click="openEdit(s)">改名</el-button>
              <el-button text size="small" type="danger" @click="remove(s)">删除</el-button>
            </span>
          </div>
          <el-button size="small" text type="primary" @click="openAddSub(m)">
            ＋ 添加小类
          </el-button>
        </div>
      </el-collapse-item>
    </el-collapse>

    <div class="major-actions">
      <el-button type="primary" plain @click="openAddMajor">＋ 添加大类</el-button>
    </div>

    <h2 class="about-title">关于</h2>
    <div class="about">
      <p>记记账 v0.1.0</p>
      <p class="db-path">数据保存在：<br />{{ dbPath || "…" }}</p>
      <p class="tip">数据只存在你的电脑上，不会上传到任何地方。</p>
    </div>

    <el-dialog
      v-model="dialog.visible"
      :title="dialog.mode === 'add' ? '添加分类' : '编辑分类'"
      width="85%"
      append-to-body
    >
      <el-form label-position="top">
        <el-form-item label="名称">
          <el-input v-model="dialog.name" maxlength="10" placeholder="如：宠物开销" />
        </el-form-item>
        <el-form-item label="图标（选填，一个表情符号）">
          <el-input v-model="dialog.emoji" maxlength="4" placeholder="如：🐱" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">取消</el-button>
        <el-button type="primary" @click="saveDialog">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.settings-page {
  height: 100%;
  overflow-y: auto;
  padding: 14px;
}
h2 {
  font-size: 14px;
  color: #37474f;
  margin-bottom: 6px;
}
.hint {
  font-size: 11px;
  color: #b0bec5;
  margin-bottom: 10px;
}
.major-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  font-size: 13px;
}
.sub-count {
  font-size: 11px;
  color: #b0bec5;
  margin-right: 10px;
}
.subs {
  padding-left: 8px;
}
.sub-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 13px;
}
.sub-name {
  color: #546e7a;
}
.major-actions {
  margin-top: 10px;
  text-align: center;
}
.about-title {
  margin-top: 22px;
}
.about {
  background: #f6faf6;
  border: 1px solid #e3efe4;
  border-radius: 12px;
  padding: 12px;
  font-size: 12px;
  color: #607d8b;
  line-height: 1.8;
}
.db-path {
  word-break: break-all;
  color: #90a4ae;
}
.tip {
  color: #a5d6a7;
  margin-top: 4px;
}
</style>
