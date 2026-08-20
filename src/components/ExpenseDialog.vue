<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { addExpense, updateExpense, type Category, type ExpenseRow } from "../api";
import { todayStr, formatYuan } from "../utils";

const props = defineProps<{
  modelValue: boolean;
  editing: ExpenseRow | null;
  categories: Category[];
}>();
const emit = defineEmits<{
  (e: "update:modelValue", v: boolean): void;
  (e: "saved"): void;
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

const amountYuan = ref("");
const categoryPath = ref<number[]>([]);
const date = ref(todayStr());
const note = ref("");
const saving = ref(false);

/** 两级分类树（大类 → 小类） */
const tree = computed(() => {
  const majors = props.categories.filter((c) => c.parent_id === null);
  return majors.map((m) => ({
    value: m.id,
    label: `${m.emoji} ${m.name}`,
    children: props.categories
      .filter((c) => c.parent_id === m.id)
      .map((c) => ({ value: c.id, label: c.name })),
  }));
});

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    if (props.editing) {
      amountYuan.value = formatYuan(props.editing.amount_cents);
      categoryPath.value = [props.editing.parent_id, props.editing.category_id];
      date.value = props.editing.date;
      note.value = props.editing.note;
    } else {
      amountYuan.value = "";
      categoryPath.value = [];
      date.value = todayStr();
      note.value = "";
    }
  },
);

async function save() {
  const amount = parseFloat(amountYuan.value);
  if (!amount || amount <= 0) {
    ElMessage.warning("请输入正确的金额");
    return;
  }
  if (categoryPath.value.length === 0) {
    ElMessage.warning("请选择分类");
    return;
  }
  const categoryId = categoryPath.value[categoryPath.value.length - 1];
  const payload = {
    amountCents: Math.round(amount * 100),
    categoryId,
    date: date.value,
    note: note.value.trim(),
  };
  saving.value = true;
  try {
    if (props.editing) {
      await updateExpense(props.editing.id, payload);
      ElMessage.success("已保存修改");
    } else {
      await addExpense(payload);
      ElMessage.success("已记下这笔账");
    }
    emit("saved");
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="editing ? '编辑这笔账' : '记一笔'"
    width="92%"
    :close-on-click-modal="false"
    append-to-body
  >
    <div class="amount-box">
      <span class="cny">¥</span>
      <input
        v-model="amountYuan"
        class="amount-input"
        type="number"
        min="0"
        step="0.01"
        placeholder="0.00"
      />
    </div>

    <el-cascader
      v-model="categoryPath"
      :options="tree"
      placeholder="选择分类（大类 → 小类）"
      class="full"
    />

    <div class="form-row">
      <span class="form-label">日期</span>
      <el-date-picker
        v-model="date"
        type="date"
        value-format="YYYY-MM-DD"
        :clearable="false"
        style="width: 200px"
      />
    </div>

    <div class="form-row">
      <span class="form-label">备注</span>
      <el-input v-model="note" placeholder="选填，如：和朋友聚餐" maxlength="100" />
    </div>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :loading="saving" @click="save">
        {{ editing ? "保存修改" : "记下这笔" }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.amount-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 4px 0 14px;
}
.cny {
  font-size: 22px;
  color: #2e7d32;
  font-weight: 600;
}
.amount-input {
  border: none;
  outline: none;
  font-size: 34px;
  font-weight: 700;
  color: #37474f;
  width: 190px;
  background: transparent;
  border-bottom: 2px solid #e3efe4;
  padding: 4px 8px;
}
.amount-input:focus {
  border-bottom-color: #4caf50;
}
.amount-input::-webkit-outer-spin-button,
.amount-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
}
.full {
  width: 100%;
  margin-bottom: 12px;
}
.form-row {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}
.form-label {
  width: 40px;
  font-size: 13px;
  color: #607d8b;
  flex-shrink: 0;
}
</style>
