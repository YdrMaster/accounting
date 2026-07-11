<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useBudgetStore } from '../stores/budget'
import { useAccountStore } from '../stores/account'
import type { BudgetDto, BudgetLimitRequest } from '../types/api'

const budgetStore = useBudgetStore()
const accountStore = useAccountStore()

onMounted(async () => {
  await Promise.all([
    budgetStore.loadBudgets(),
    accountStore.loadAccounts(),
  ])
})

const selectedBudgetId = ref<number | null>(null)
const showCreateDrawer = ref(false)
const editingBudget = ref<BudgetDto | null>(null)

function periodLabel(period: string): string {
  const labels: Record<string, string> = {
    daily: '每日',
    'weekly-sun': '每周（周日起）',
    'weekly-mon': '每周（周一起）',
    monthly: '每月',
    yearly: '每年',
  }
  return labels[period] ?? period
}

function onNewBudget() {
  editingBudget.value = null
  showCreateDrawer.value = true
}

function onEditBudget(budget: BudgetDto) {
  editingBudget.value = budget
  showCreateDrawer.value = true
}

function onDeleteBudget(id: number) {
  if (confirm('确定要删除这个预算表吗？')) {
    budgetStore.remove(id)
    if (selectedBudgetId.value === id) {
      selectedBudgetId.value = null
    }
  }
}

function onDrawerClosed() {
  showCreateDrawer.value = false
  editingBudget.value = null
}

function onBudgetCreated() {
  showCreateDrawer.value = false
  editingBudget.value = null
  budgetStore.loadBudgets()
}

// Create/Edit form state
const formName = ref('')
const formPeriod = ref('monthly')
const formCommodityId = ref(1)
const formLimits = ref<BudgetLimitRequest[]>([])

function addLimit() {
  formLimits.value.push({ account_id: 0, amount: '0' })
}

function removeLimit(index: number) {
  formLimits.value.splice(index, 1)
}

async function submitBudget() {
  if (!formName.value.trim()) {
    alert('请输入预算表名称')
    return
  }
  if (formLimits.value.length === 0) {
    alert('请至少添加一个限额')
    return
  }

  const data = {
    name: formName.value.trim(),
    period: formPeriod.value,
    commodity_id: formCommodityId.value,
    limits: formLimits.value.filter(l => l.account_id > 0),
  }

  try {
    if (editingBudget.value) {
      await budgetStore.update(editingBudget.value.id, data)
    } else {
      await budgetStore.create(data)
    }
    onBudgetCreated()
  } catch (e) {
    alert('保存失败: ' + (e instanceof Error ? e.message : String(e)))
  }
}
</script>

<template>
  <div class="budget">
    <div class="header-actions">
      <button class="new-budget-btn" @click="onNewBudget">+ 新建预算</button>
    </div>

    <div v-if="budgetStore.loading" class="loading">加载中...</div>
    <div v-else-if="budgetStore.error" class="error">{{ budgetStore.error }}</div>
    <template v-else>
      <div v-if="budgetStore.budgets.length === 0" class="empty">暂无预算表</div>

      <div v-for="budget in budgetStore.budgets" :key="budget.id" class="budget-card">
        <div class="budget-info">
          <h3>{{ budget.name }}</h3>
          <p class="budget-meta">{{ periodLabel(budget.period) }}</p>
        </div>
        <div class="budget-actions">
          <button class="edit-btn" @click="onEditBudget(budget)">编辑</button>
          <button class="delete-btn" @click="onDeleteBudget(budget.id)">删除</button>
        </div>
      </div>
    </template>

    <!-- Create/Edit Drawer -->
    <div v-if="showCreateDrawer" class="drawer-container">
      <div class="drawer-backdrop" @click="onDrawerClosed" />
      <div class="drawer">
        <div class="drawer-header">
          <span class="drawer-title">{{ editingBudget ? '编辑预算' : '新建预算' }}</span>
          <button class="drawer-close" @click="onDrawerClosed">×</button>
        </div>

        <div class="drawer-body">
          <div class="field">
            <label>预算表名称</label>
            <input v-model="formName" type="text" placeholder="输入预算表名称" />
          </div>

          <div class="field">
            <label>周期类型</label>
            <select v-model="formPeriod">
              <option value="daily">每日</option>
              <option value="weekly-sun">每周（周日起）</option>
              <option value="weekly-mon">每周（周一起）</option>
              <option value="monthly">每月</option>
              <option value="yearly">每年</option>
            </select>
          </div>

          <div class="section-title">限额列表</div>

          <div v-for="(limit, index) in formLimits" :key="index" class="limit-row">
            <select v-model="limit.account_id">
              <option :value="0" disabled>选择账户</option>
              <option v-for="acc in accountStore.accounts" :key="acc.id" :value="acc.id">
                {{ acc.name }}
              </option>
            </select>
            <input v-model="limit.amount" type="number" step="0.01" placeholder="限额" />
            <button class="remove-limit-btn" @click="removeLimit(index)">×</button>
          </div>

          <button class="add-limit-btn" @click="addLimit">+ 添加限额</button>

          <button class="submit-btn" @click="submitBudget">
            {{ editingBudget ? '保存' : '创建' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.budget {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
}

.header-actions {
  display: flex;
  justify-content: flex-end;
}

.new-budget-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.new-budget-btn:hover {
  opacity: 0.9;
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.budget-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--card-bg-alt, #252525);
  border-radius: 0.75rem;
  padding: 1rem;
}

.budget-info h3 {
  margin: 0 0 0.25rem;
  color: var(--text-heading);
  font-size: 1rem;
}

.budget-meta {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.8125rem;
}

.budget-actions {
  display: flex;
  gap: 0.5rem;
}

.edit-btn,
.delete-btn {
  background: none;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  cursor: pointer;
  color: var(--text-heading);
}

.edit-btn:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.delete-btn:hover {
  border-color: #e74c3c;
  color: #e74c3c;
}

/* Drawer styles */
.drawer-container {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.drawer-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
}

.drawer {
  position: relative;
  max-height: 60vh;
  max-width: 600px;
  margin: 0 auto;
  width: 100%;
  background: var(--card-bg, #1e1e1e);
  border-radius: 1rem 1rem 0 0;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease-out;
}

@keyframes slideUp {
  from { transform: translateY(100%); }
  to { transform: translateY(0); }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
}

.drawer-title {
  font-weight: 600;
  color: var(--text-heading);
}

.drawer-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  color: var(--text-muted);
  cursor: pointer;
  line-height: 1;
}

.drawer-body {
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.field label {
  color: var(--text-muted);
  font-size: 0.8125rem;
}

.field input,
.field select {
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  outline: none;
}

.field input:focus,
.field select:focus {
  border-color: var(--accent, #646cff);
}

.section-title {
  font-weight: 500;
  color: var(--text-heading);
  margin-top: 0.5rem;
}

.limit-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.limit-row select {
  flex: 2;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.375rem 0.5rem;
  color: var(--text-heading);
  font-size: 0.8125rem;
  outline: none;
}

.limit-row input {
  flex: 1;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.375rem 0.5rem;
  color: var(--text-heading);
  font-size: 0.8125rem;
  outline: none;
}

.remove-limit-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
}

.remove-limit-btn:hover {
  color: #e74c3c;
}

.add-limit-btn {
  background: var(--card-bg-alt, #252525);
  border: 1px dashed var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
}

.add-limit-btn:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.submit-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  margin-top: 0.5rem;
}

.submit-btn:hover {
  opacity: 0.9;
}
</style>
