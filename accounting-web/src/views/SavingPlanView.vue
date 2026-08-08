<script setup lang="ts">
import { inject, onMounted, ref, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import AccountPicker from '../components/layout/AccountPicker.vue'
import { panelActionKey } from '../components/layout/panelAction'
import ProgressRing from '../components/ProgressRing.vue'
import { useAccountStore } from '../stores/account'
import { useSavingPlanStore } from '../stores/savingPlan'
import { alertDialog, confirmDialog } from '../utils/dialog'
import type { AccountAllocationDto, SavingPlanDto, SavingPlanStatusDto } from '../types/api'
import { formatDecimal } from '../utils/decimal'
import { PALETTE } from '../utils/palette'

const savingPlanStore = useSavingPlanStore()
const accountStore = useAccountStore()
const { t } = useI18n()

onMounted(async () => {
  await Promise.all([savingPlanStore.loadStatuses(), accountStore.loadAccounts()])
})

const expandedPlanId = ref<number | null>(null)
const showDrawer = ref(false)
const editingPlan = ref<SavingPlanDto | null>(null)

const RING_COLORS = {
  green: PALETTE.income,
  yellow: PALETTE.attention,
  gray: PALETTE.neutral,
} as const

function ringClass(status: SavingPlanStatusDto): string {
  if (status.expired) return 'ring-gray'
  return Number(status.satisfaction) >= 100 ? 'ring-green' : 'ring-yellow'
}

function ringColor(status: SavingPlanStatusDto): string {
  if (status.expired) return RING_COLORS.gray
  return Number(status.satisfaction) >= 100 ? RING_COLORS.green : RING_COLORS.yellow
}

function periodLabel(period: string | null): string {
  const labels: Record<string, string> = {
    daily: t('savingPlan.period.daily'),
    'weekly-sun': t('savingPlan.period.weeklySun'),
    'weekly-mon': t('savingPlan.period.weeklyMon'),
    monthly: t('savingPlan.period.monthly'),
    yearly: t('savingPlan.period.yearly'),
  }
  return period ? (labels[period] ?? period) : t('savingPlan.period.once')
}

function metaLabel(plan: SavingPlanDto): string {
  const parts = [periodLabel(plan.period)]
  if (plan.deadline) {
    parts.push(`${t('savingPlan.deadlinePrefix')} ${plan.deadline}`)
  }
  return parts.join(' · ')
}

function toggleExpand(planId: number) {
  expandedPlanId.value = expandedPlanId.value === planId ? null : planId
}

function accountName(accountId: number): string {
  return accountStore.accountPath(accountId) || `#${accountId}`
}

function occupiedWidth(alloc: AccountAllocationDto): string {
  const balance = Number(alloc.balance)
  const occupied = Number(alloc.occupied_by_earlier)
  if (!(balance > 0) || !(occupied > 0)) return '0%'
  return `${Math.min((occupied / balance) * 100, 100)}%`
}

function onNewPlan() {
  editingPlan.value = null
  resetForm()
  showDrawer.value = true
}

function onEditPlan(plan: SavingPlanDto) {
  editingPlan.value = plan
  formName.value = plan.name
  formPeriod.value = plan.period ?? ''
  formDeadline.value = plan.deadline ?? ''
  formTarget.value = plan.target_amount
  formAccountIds.value = [...plan.account_ids]
  showDrawer.value = true
}

async function onDeletePlan(id: number) {
  if (await confirmDialog(t('savingPlan.confirmDelete'))) {
    await savingPlanStore.remove(id)
    if (expandedPlanId.value === id) {
      expandedPlanId.value = null
    }
    savingPlanStore.loadStatuses()
  }
}

function onDrawerClosed() {
  showDrawer.value = false
  editingPlan.value = null
}

function onPlanSaved() {
  showDrawer.value = false
  editingPlan.value = null
  savingPlanStore.loadStatuses()
}

// Create/Edit form state
const formName = ref('')
const formPeriod = ref('monthly')
const formDeadline = ref('')
const formTarget = ref('')
const formAccountIds = ref<number[]>([])

function resetForm() {
  formName.value = ''
  formPeriod.value = 'monthly'
  formDeadline.value = ''
  formTarget.value = ''
  formAccountIds.value = []
}

function addAccount() {
  formAccountIds.value.push(0)
}

function removeAccount(index: number) {
  formAccountIds.value.splice(index, 1)
}

function setAccount(index: number, accountId: number) {
  if (formAccountIds.value.some((id, i) => i !== index && id === accountId)) {
    alertDialog(t('savingPlan.duplicateAccount'))
    return
  }
  formAccountIds.value[index] = accountId
}

async function submitPlan() {
  if (!formName.value.trim()) {
    alertDialog(t('savingPlan.nameRequired'))
    return
  }
  if (!(Number(formTarget.value) > 0)) {
    alertDialog(t('savingPlan.targetRequired'))
    return
  }
  const accountIds = [...new Set(formAccountIds.value.filter(id => id > 0))]
  if (accountIds.length === 0) {
    alertDialog(t('savingPlan.accountsRequired'))
    return
  }

  const data = {
    name: formName.value.trim(),
    period: formPeriod.value || null,
    deadline: formDeadline.value || null,
    commodity_id: 1,
    target_amount: String(formTarget.value),
    account_ids: accountIds,
  }

  try {
    if (editingPlan.value) {
      await savingPlanStore.update(editingPlan.value.id, data)
    } else {
      await savingPlanStore.create(data)
    }
    onPlanSaved()
  } catch (e) {
    alertDialog(t('savingPlan.saveFailed', { message: e instanceof Error ? e.message : String(e) }))
  }
}

const panelAction = inject(panelActionKey, null)
watchEffect(() => {
  if (!panelAction) return
  panelAction.value = showDrawer.value
    ? []
    : [{ label: t('savingPlan.new'), disabled: false, onClick: onNewPlan }]
})
</script>

<template>
  <div class="saving-plan" :class="{ 'no-scroll': showDrawer }">
    <!-- Show normal list view when drawer is not displayed -->
    <template v-if="!showDrawer">
      <div v-if="savingPlanStore.loading" class="loading">{{ t('common.loading') }}</div>
      <div v-else-if="savingPlanStore.error" class="error">{{ savingPlanStore.error }}</div>
      <template v-else>
        <div v-if="savingPlanStore.statuses.length === 0" class="empty">
          {{ t('savingPlan.empty') }}
        </div>

        <div
          v-for="status in savingPlanStore.statuses"
          :key="status.plan.id"
          class="budget-card plan-card"
          @click="toggleExpand(status.plan.id)"
        >
          <div class="plan-ring" :class="ringClass(status)">
            <ProgressRing
              :percentage="Number(status.satisfaction)"
              :color="ringColor(status)"
              :size="72"
              >{{ formatDecimal(status.satisfaction) }}%</ProgressRing
            >
          </div>
          <div class="budget-info">
            <h3>{{ status.plan.name }}</h3>
            <p class="budget-meta">{{ metaLabel(status.plan) }}</p>
            <p class="budget-meta">
              {{ t('savingPlan.detail.targetAmount') }}: {{ status.target_amount }}
            </p>
            <div class="plan-badges">
              <span v-if="status.expired" class="badge badge-expired">
                {{ t('savingPlan.expired') }}
              </span>
              <span v-else-if="status.met" class="badge badge-met">
                {{ t('savingPlan.metBadge') }}
              </span>
              <span v-else class="badge badge-gap">
                {{ t('savingPlan.gapBadge', { amount: formatDecimal(status.gap) }) }}
              </span>
            </div>
          </div>
          <div class="budget-actions" @click.stop>
            <button class="edit-btn" @click="onEditPlan(status.plan)">
              {{ t('common.edit') }}
            </button>
            <button class="delete-btn" @click="onDeletePlan(status.plan.id)">
              {{ t('common.delete') }}
            </button>
          </div>

          <!-- Inline expanded status detail -->
          <div v-if="expandedPlanId === status.plan.id" class="plan-detail" @click.stop>
            <div class="detail-group">
              <div class="detail-title">{{ t('savingPlan.detail.bookTitle') }}</div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.targetAmount') }}</span>
                <span>{{ status.target_amount }}</span>
              </div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.currentBalance') }}</span>
                <span>{{ status.current_balance }}</span>
              </div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.gap') }}</span>
                <span>{{ formatDecimal(status.gap) }}</span>
              </div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.met') }}</span>
                <span>
                  {{
                    status.met
                      ? t('savingPlan.detail.metYes')
                      : t('savingPlan.detail.metNo')
                  }}
                </span>
              </div>
            </div>

            <div class="detail-group">
              <div class="detail-title">{{ t('savingPlan.detail.allocationTitle') }}</div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.allocated') }}</span>
                <span>{{ status.allocated }}</span>
              </div>
              <div class="detail-row">
                <span>{{ t('savingPlan.detail.satisfaction') }}</span>
                <span>{{ formatDecimal(status.satisfaction) }}%</span>
              </div>
            </div>

            <div class="alloc-list">
              <div v-for="alloc in status.accounts" :key="alloc.account_id" class="alloc-row">
                <div class="alloc-name">{{ accountName(alloc.account_id) }}</div>
                <div class="balance-bar">
                  <div class="bar-occupied" :style="{ width: occupiedWidth(alloc) }" />
                </div>
                <div class="alloc-nums">
                  <span class="alloc-balance">
                    {{ t('savingPlan.detail.balance') }} {{ alloc.balance }}
                  </span>
                  <span class="alloc-occupied">
                    {{ t('savingPlan.detail.occupied') }} {{ alloc.occupied_by_earlier }}
                  </span>
                  <span class="alloc-allocated">
                    {{ t('savingPlan.detail.accountAllocated') }} {{ alloc.allocated }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- Create/Edit Drawer - completely replaces list view -->
    <div v-if="showDrawer" class="drawer-container">
      <div class="drawer-backdrop" @click="onDrawerClosed" />
      <div class="drawer">
        <div class="drawer-header">
          <span class="drawer-title">
            {{ editingPlan ? t('savingPlan.editTitle') : t('savingPlan.newTitle') }}
          </span>
          <button class="drawer-close" @click="onDrawerClosed">×</button>
        </div>

        <div class="drawer-body">
          <div class="field">
            <label>{{ t('savingPlan.nameLabel') }}</label>
            <input v-model="formName" type="text" :placeholder="t('savingPlan.namePlaceholder')" />
          </div>

          <div class="field">
            <label>{{ t('savingPlan.periodLabel') }}</label>
            <select v-model="formPeriod">
              <option value="daily">{{ t('savingPlan.period.daily') }}</option>
              <option value="weekly-sun">{{ t('savingPlan.period.weeklySun') }}</option>
              <option value="weekly-mon">{{ t('savingPlan.period.weeklyMon') }}</option>
              <option value="monthly">{{ t('savingPlan.period.monthly') }}</option>
              <option value="yearly">{{ t('savingPlan.period.yearly') }}</option>
              <option value="">{{ t('savingPlan.period.once') }}</option>
            </select>
          </div>

          <div class="field">
            <label>{{ t('savingPlan.deadlineLabel') }}</label>
            <input v-model="formDeadline" type="date" />
          </div>

          <div class="field">
            <label>{{ t('savingPlan.targetLabel') }}</label>
            <input
              v-model="formTarget"
              type="number"
              step="0.01"
              min="0"
              :placeholder="t('savingPlan.targetPlaceholder')"
            />
          </div>

          <div class="section-title">{{ t('savingPlan.accountsTitle') }}</div>

          <div v-for="(accountId, index) in formAccountIds" :key="index" class="account-row">
            <AccountPicker
              :model-value="accountId || null"
              :placeholder="t('savingPlan.selectAccount')"
              account-type="asset"
              @update:model-value="id => setAccount(index, id)"
            />
            <button class="remove-account-btn" @click="removeAccount(index)">×</button>
          </div>

          <button class="add-account-btn" @click="addAccount">
            + {{ t('savingPlan.addAccount') }}
          </button>

          <button class="submit-btn" @click="submitPlan">
            {{ editingPlan ? t('common.save') : t('savingPlan.create') }}
          </button>
        </div>
      </div>

      <!-- Portal container for AccountPicker Teleport -->
      <div class="picker-portal"></div>
    </div>
  </div>
</template>

<style scoped>
.saving-plan {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
  height: 100%;
  overflow-y: auto;
}

.saving-plan.no-scroll {
  overflow: hidden;
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.plan-card {
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

.plan-ring {
  flex-shrink: 0;
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

.plan-badges {
  display: flex;
  gap: 0.375rem;
  margin-top: 0.375rem;
}

.badge {
  border-radius: 0.375rem;
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1.4;
}

.badge-met {
  background: rgba(46, 204, 113, 0.15);
  color: var(--color-income);
}

.badge-gap {
  background: rgba(241, 196, 15, 0.15);
  color: var(--color-attention);
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
.plan-detail {
  flex-basis: 100%;
  border-top: 1px solid var(--border);
  padding-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  cursor: default;
}

.detail-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.detail-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
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

.alloc-list {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.alloc-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.alloc-name {
  font-size: 0.8125rem;
  color: var(--text-heading);
}

.balance-bar {
  position: relative;
  height: 0.5rem;
  border-radius: 0.25rem;
  background: var(--accent, #646cff);
  overflow: hidden;
}

.bar-occupied {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  background: rgba(241, 196, 15, 0.75);
}

.alloc-nums {
  display: flex;
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

.account-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.account-row .account-picker {
  flex: 1;
}

.remove-account-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
}

.remove-account-btn:hover {
  color: var(--color-expense);
}

.add-account-btn {
  background: var(--card-bg-alt, #252525);
  border: 1px dashed var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
}

.add-account-btn:hover {
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
