<script setup lang="ts">
import { inject, onMounted, ref, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import AccountPicker from '../components/layout/AccountPicker.vue'
import { panelActionKey } from '../components/layout/panelAction'
import ProgressRing from '../components/ProgressRing.vue'
import { useAccountStore } from '../stores/account'
import { useBudgetStore } from '../stores/budget'
import { alertDialog, confirmDialog } from '../utils/dialog'
import type { BudgetLimitRequest, BudgetStatusDto } from '../types/api'
import { formatDecimal } from '../utils/decimal'
import { PALETTE } from '../utils/palette'

const budgetStore = useBudgetStore()
const accountStore = useAccountStore()
const { t } = useI18n()

onMounted(async () => {
  await Promise.all([budgetStore.loadStatuses(), accountStore.loadAccounts()])
})

const expandedBudgetId = ref<number | null>(null)
const showCreateDrawer = ref(false)
const editingBudget = ref<BudgetStatusDto | null>(null)

const RING_COLORS = {
  green: PALETTE.income,
  red: PALETTE.expense,
  gray: PALETTE.neutral,
} as const

function totals(status: BudgetStatusDto): { limit: number; actual: number } {
  return status.items.reduce(
    (acc, item) => ({
      limit: acc.limit + Number(item.limit_amount),
      actual: acc.actual + Number(item.actual_amount),
    }),
    { limit: 0, actual: 0 }
  )
}

function isOverspent(status: BudgetStatusDto): boolean {
  const { limit, actual } = totals(status)
  return actual > limit
}

function ringPercentage(status: BudgetStatusDto): number {
  const { limit, actual } = totals(status)
  if (!(limit > 0)) return actual > 0 ? 100 : 0
  return (actual / limit) * 100
}

function ringClass(status: BudgetStatusDto): string {
  if (status.expired) return 'ring-gray'
  return isOverspent(status) ? 'ring-red' : 'ring-green'
}

function ringColor(status: BudgetStatusDto): string {
  if (status.expired) return RING_COLORS.gray
  return isOverspent(status) ? RING_COLORS.red : RING_COLORS.green
}

function ringAmount(status: BudgetStatusDto): string {
  const { limit, actual } = totals(status)
  return Math.abs(limit - actual).toFixed(2)
}

function periodLabel(period: string | null): string {
  const labels: Record<string, string> = {
    daily: t('budget.period.daily'),
    'weekly-sun': t('budget.period.weeklySun'),
    'weekly-mon': t('budget.period.weeklyMon'),
    monthly: t('budget.period.monthly'),
    yearly: t('budget.period.yearly'),
  }
  return period ? (labels[period] ?? period) : t('budget.period.once')
}

function metaLabel(status: BudgetStatusDto): string {
  const parts = [periodLabel(status.budget.period)]
  if (status.budget.deadline) {
    parts.push(`${t('budget.deadlinePrefix')} ${status.budget.deadline}`)
  }
  return parts.join(' · ')
}

function toggleExpand(budgetId: number) {
  expandedBudgetId.value = expandedBudgetId.value === budgetId ? null : budgetId
}

function accountName(accountId: number): string {
  return accountStore.accountPath(accountId) || `#${accountId}`
}

function onNewBudget() {
  editingBudget.value = null
  resetForm()
  showCreateDrawer.value = true
}

function onEditBudget(status: BudgetStatusDto) {
  editingBudget.value = status
  formName.value = status.budget.name
  formPeriod.value = status.budget.period ?? ''
  formDeadline.value = status.budget.deadline ?? ''
  formLimits.value = status.items.map(item => ({
    account_id: item.account_id,
    amount: item.limit_amount,
  }))
  showCreateDrawer.value = true
}

async function onDeleteBudget(id: number) {
  if (await confirmDialog(t('budget.confirmDelete'))) {
    await budgetStore.remove(id)
    if (expandedBudgetId.value === id) {
      expandedBudgetId.value = null
    }
  }
}

function onDrawerClosed() {
  showCreateDrawer.value = false
  editingBudget.value = null
}

function onBudgetSaved() {
  showCreateDrawer.value = false
  editingBudget.value = null
  budgetStore.loadStatuses()
}

// Create/Edit form state
const formName = ref('')
const formPeriod = ref('monthly')
const formDeadline = ref('')
const formCommodityId = ref(1)
const formLimits = ref<BudgetLimitRequest[]>([])

function resetForm() {
  formName.value = ''
  formPeriod.value = 'monthly'
  formDeadline.value = ''
  formLimits.value = []
}

function addLimit() {
  formLimits.value.push({ account_id: 0, amount: '0' })
}

function removeLimit(index: number) {
  formLimits.value.splice(index, 1)
}

async function submitBudget() {
  if (!formName.value.trim()) {
    alertDialog(t('budget.nameRequired'))
    return
  }
  if (formLimits.value.length === 0) {
    alertDialog(t('budget.limitRequired'))
    return
  }

  const data = {
    name: formName.value.trim(),
    period: formPeriod.value || null,
    deadline: formDeadline.value || null,
    commodity_id: formCommodityId.value,
    limits: formLimits.value
      .filter(l => l.account_id > 0)
      .map(l => ({ account_id: l.account_id, amount: String(l.amount) })),
  }

  try {
    if (editingBudget.value) {
      await budgetStore.update(editingBudget.value.budget.id, data)
    } else {
      await budgetStore.create(data)
    }
    onBudgetSaved()
  } catch (e) {
    alertDialog(t('budget.saveFailed', { message: e instanceof Error ? e.message : String(e) }))
  }
}

const panelAction = inject(panelActionKey, null)
watchEffect(() => {
  if (!panelAction) return
  panelAction.value = showCreateDrawer.value
    ? []
    : [{ label: t('budget.new'), disabled: false, onClick: onNewBudget }]
})
</script>

<template>
  <div class="budget" :class="{ 'no-scroll': showCreateDrawer }">
    <!-- Show normal budget view when drawer is not displayed -->
    <template v-if="!showCreateDrawer">
      <div v-if="budgetStore.loading" class="loading">{{ t('common.loading') }}</div>
      <div v-else-if="budgetStore.error" class="error">{{ budgetStore.error }}</div>
      <template v-else>
        <div v-if="budgetStore.statuses.length === 0" class="empty">{{ t('budget.empty') }}</div>

        <div
          v-for="status in budgetStore.statuses"
          :key="status.budget.id"
          class="budget-card"
          @click="toggleExpand(status.budget.id)"
        >
          <div class="budget-ring" :class="ringClass(status)">
            <ProgressRing :percentage="ringPercentage(status)" :color="ringColor(status)" :size="80">
              <div class="ring-center">
                <div class="ring-label">
                  {{ isOverspent(status) ? t('budget.overspentLabel') : t('budget.remainingLabel') }}
                </div>
                <div class="ring-amount">{{ ringAmount(status) }}</div>
              </div>
            </ProgressRing>
          </div>
          <div class="budget-info">
            <h3>{{ status.budget.name }}</h3>
            <p class="budget-meta">{{ metaLabel(status) }}</p>
            <div class="budget-badges">
              <span v-if="status.expired" class="badge badge-expired">
                {{ t('budget.expired') }}
              </span>
            </div>
          </div>
          <div class="budget-actions" @click.stop>
            <button class="edit-btn" @click="onEditBudget(status)">{{ t('common.edit') }}</button>
            <button class="delete-btn" @click="onDeleteBudget(status.budget.id)">
              {{ t('common.delete') }}
            </button>
          </div>

          <!-- Inline expanded status detail -->
          <div v-if="expandedBudgetId === status.budget.id" class="budget-detail" @click.stop>
            <div v-if="status.period_start && status.period_end" class="detail-row period-range">
              <span>{{ t('budget.detail.currentPeriod') }}</span>
              <span>{{ status.period_start }} ~ {{ status.period_end }}</span>
            </div>

            <div class="status-list">
              <div
                v-for="item in status.items"
                :key="item.account_id"
                class="status-row"
                :class="{ overspent: Number(item.remaining) < 0 }"
              >
                <div class="status-name">{{ accountName(item.account_id) }}</div>
                <div class="status-nums">
                  <span class="item-limit">
                    {{ t('budget.detail.limit') }} {{ item.limit_amount }}
                  </span>
                  <span class="item-actual">
                    {{ t('budget.detail.actual') }} {{ item.actual_amount }}
                  </span>
                  <span class="item-remaining">
                    {{ t('budget.detail.remaining') }} {{ item.remaining }}
                  </span>
                  <span class="item-percentage">{{ formatDecimal(item.percentage) }}%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- Create/Edit Drawer - completely replaces budget view -->
    <div v-if="showCreateDrawer" class="drawer-container">
      <div class="drawer-backdrop" @click="onDrawerClosed" />
      <div class="drawer">
        <div class="drawer-header">
          <span class="drawer-title">
            {{ editingBudget ? t('budget.editTitle') : t('budget.newTitle') }}
          </span>
          <button class="drawer-close" @click="onDrawerClosed">×</button>
        </div>

        <div class="drawer-body">
          <div class="field">
            <label>{{ t('budget.nameLabel') }}</label>
            <input v-model="formName" type="text" :placeholder="t('budget.namePlaceholder')" />
          </div>

          <div class="field">
            <label>{{ t('budget.periodLabel') }}</label>
            <select v-model="formPeriod">
              <option value="daily">{{ t('budget.period.daily') }}</option>
              <option value="weekly-sun">{{ t('budget.period.weeklySun') }}</option>
              <option value="weekly-mon">{{ t('budget.period.weeklyMon') }}</option>
              <option value="monthly">{{ t('budget.period.monthly') }}</option>
              <option value="yearly">{{ t('budget.period.yearly') }}</option>
              <option value="">{{ t('budget.period.once') }}</option>
            </select>
          </div>

          <div class="field">
            <label>{{ t('budget.deadlineLabel') }}</label>
            <input v-model="formDeadline" type="date" />
          </div>

          <div class="section-title">{{ t('budget.limitsTitle') }}</div>

          <div v-for="(limit, index) in formLimits" :key="index" class="limit-row">
            <AccountPicker
              :model-value="limit.account_id || null"
              :placeholder="t('budget.selectAccount')"
              account-type="expense"
              @update:model-value="
                id => {
                  formLimits[index].account_id = id
                }
              "
            />
            <input
              v-model="limit.amount"
              type="number"
              step="0.01"
              :placeholder="t('budget.limitPlaceholder')"
            />
            <button class="remove-limit-btn" @click="removeLimit(index)">×</button>
          </div>

          <button class="add-limit-btn" @click="addLimit">+ {{ t('budget.addLimit') }}</button>

          <button class="submit-btn" @click="submitBudget">
            {{ editingBudget ? t('common.save') : t('budget.create') }}
          </button>
        </div>
      </div>

      <!-- Portal container for AccountPicker Teleport -->
      <div class="picker-portal"></div>
    </div>
  </div>
</template>

<style scoped>
.budget {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
  height: 100%;
  overflow-y: auto;
}

.budget.no-scroll {
  overflow: hidden;
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
  gap: 1rem;
  flex-wrap: wrap;
  background: var(--card-bg-alt, #252525);
  border-radius: 0.75rem;
  padding: 1rem;
  cursor: pointer;
}

.budget-ring {
  flex-shrink: 0;
}

.ring-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1.2;
}

.ring-label {
  font-size: 0.625rem;
  font-weight: 400;
  color: var(--text-muted);
}

.ring-amount {
  font-size: 0.75rem;
}

.ring-red .ring-label,
.ring-red .ring-amount {
  color: var(--color-expense);
}

.budget-info {
  flex: 1;
  min-width: 0;
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

.budget-badges {
  display: flex;
  gap: 0.375rem;
  margin-top: 0.375rem;
}

.budget-badges:empty {
  margin-top: 0;
}

.badge {
  border-radius: 0.375rem;
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1.4;
}

.badge-expired {
  background: rgba(127, 140, 141, 0.2);
  color: var(--color-neutral);
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
  border-color: var(--color-expense);
  color: var(--color-expense);
}

/* Inline status detail */
.budget-detail {
  flex-basis: 100%;
  border-top: 1px solid var(--border);
  padding-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  cursor: default;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.8125rem;
  color: var(--text-heading);
}

.detail-row span:first-child {
  color: var(--text-muted);
}

.status-list {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.status-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.status-name {
  font-size: 0.8125rem;
  color: var(--text-heading);
}

.status-row.overspent .status-name,
.status-row.overspent .item-remaining,
.status-row.overspent .item-percentage {
  color: var(--color-expense);
}

.status-nums {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Drawer styles */
.drawer-container {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
}

.drawer-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
}

.drawer {
  position: relative;
  width: 100%;
  height: 100%;
  background: var(--card-bg, #1e1e1e);
  display: flex;
  flex-direction: column;
  animation: slideIn 0.2s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
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
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-height: 0;
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

.limit-row .account-picker {
  flex: 2;
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
  color: var(--color-expense);
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

<style>
/* Non-scoped styles for AccountPicker Teleport target */
.drawer .picker-portal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.drawer .picker-portal > * {
  pointer-events: auto;
}
</style>
