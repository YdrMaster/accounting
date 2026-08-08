<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, inject, onMounted, ref, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import TransactionFilterDrawer from '../components/TransactionFilterDrawer.vue'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'
import { panelActionKey } from '../components/layout/panelAction'
import { useTransactionStore } from '../stores/transaction'
import { confirmDialog } from '../utils/dialog'
import { monthOf, todayStr } from '../utils/date'

const txStore = useTransactionStore()
const { t } = useI18n()

const showFormOverlay = ref(false)
const showFilterDrawer = ref(false)
const editingTxId = ref<number | undefined>(undefined)
const scrollContainer = ref<HTMLElement | null>(null)

onMounted(async () => {
  await txStore.loadInitial(todayStr(), 100)
})

function onScroll() {
  const el = scrollContainer.value
  if (!el || txStore.loading) return
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
  if (nearBottom) {
    txStore.loadMore()
  }
}

const monthLabelDate = computed(
  () => txStore.loadedRange?.to ?? txStore.activeFilter?.to ?? todayStr()
)

const currentMonthStr = computed(() => {
  if (!txStore.loadedRange) return ''
  return monthOf({ date_time: txStore.loadedRange.to })
})

const monthlyExpense = computed(() => {
  const m = currentMonthStr.value
  if (!m) return '0.00'
  let sum = new Decimal(0)
  for (const tx of txStore.transactions) {
    if (monthOf(tx) === m) {
      for (const p of tx.postings) {
        if (p.account_type === 'expense') {
          sum = sum.plus(new Decimal(p.amount))
        }
      }
    }
  }
  return sum.toFixed(2)
})

const monthlyIncome = computed(() => {
  const m = currentMonthStr.value
  if (!m) return '0.00'
  let sum = new Decimal(0)
  for (const tx of txStore.transactions) {
    if (monthOf(tx) === m) {
      for (const p of tx.postings) {
        if (p.account_type === 'income') {
          sum = sum.plus(new Decimal(p.amount))
        }
      }
    }
  }
  return sum.negated().toFixed(2)
})

const monthlyBalance = computed(() => {
  const exp = new Decimal(monthlyExpense.value)
  const incomeVal = monthlyIncome.value
  return new Decimal(incomeVal).minus(exp).toFixed(2)
})

function formatAmount(amt: Decimal): string {
  const fixed = amt.toFixed(2)
  const [intPart, decPart] = fixed.split('.')
  const sign = intPart.startsWith('-') ? '-' : ''
  const abs = intPart.replace('-', '')
  const formatted = abs.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${formatted}.${decPart}`
}

function onEditTx(id: number) {
  showFilterDrawer.value = false
  editingTxId.value = id
  showFormOverlay.value = true
}

async function onDeleteTx(id: number) {
  if (await confirmDialog(t('transactions.confirmDelete'))) {
    txStore.remove(id)
  }
}

function onNewTx() {
  showFilterDrawer.value = false
  editingTxId.value = undefined
  showFormOverlay.value = true
}

function onToggleFilter() {
  showFilterDrawer.value = !showFilterDrawer.value
}

function onFormClosed() {
  showFormOverlay.value = false
  editingTxId.value = undefined
}

function onFormSaved() {
  // Data is already updated via create/update in store
}

const panelAction = inject(panelActionKey, null)
const filterIcon =
  '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/></svg>'
watchEffect(() => {
  if (!panelAction) return
  if (showFormOverlay.value) {
    panelAction.value = []
    return
  }
  panelAction.value = [
    { label: t('transactions.new'), disabled: false, onClick: onNewTx },
    { label: t('txFilter.filter'), icon: filterIcon, disabled: false, onClick: onToggleFilter },
  ]
})
</script>

<template>
  <div class="transaction-root">
    <div
      ref="scrollContainer"
      class="transaction"
      :class="{ 'no-scroll': showFormOverlay }"
      @scroll="onScroll"
    >
      <template v-if="!showFormOverlay">
        <div class="hero">
          <p class="month-label">
            {{
              t('transactions.monthLabel', {
                year: monthLabelDate.slice(0, 4),
                month: monthLabelDate.slice(5, 7),
              })
            }}
            <span v-if="txStore.filterActive" class="filtered-badge">{{ t('txFilter.filtered') }}</span>
          </p>
          <p class="label">{{ t('transactions.monthlyExpense') }}</p>
          <p class="amount">¥{{ formatAmount(new Decimal(monthlyExpense)) }}</p>
          <p class="sub">
            {{ t('transactions.monthlyIncome') }} ¥{{ formatAmount(new Decimal(monthlyIncome)) }} ·
            {{ t('transactions.monthlyBalance') }} ¥{{ formatAmount(new Decimal(monthlyBalance)) }}
          </p>
        </div>

        <div v-if="txStore.loading && txStore.transactions.length === 0" class="loading">
          {{ t('common.loading') }}
        </div>
        <div v-else-if="txStore.error" class="error">{{ txStore.error }}</div>

        <TransactionList :transactions="txStore.transactions" @edit="onEditTx" @delete="onDeleteTx" />

        <div v-if="txStore.loading && txStore.transactions.length > 0" class="loading-more">
          {{ t('transactions.loadingMore') }}
        </div>
      </template>
    </div>

    <TransactionFormOverlay
      v-if="showFormOverlay"
      :edit-id="editingTxId"
      @close="onFormClosed"
      @saved="onFormSaved"
    />

    <TransactionFilterDrawer
      v-if="showFilterDrawer && !showFormOverlay"
      @close="showFilterDrawer = false"
    />

    <div class="tx-filter-portal"></div>
  </div>
</template>

<style scoped>
.transaction-root {
  position: relative;
  height: 100%;
}

.tx-filter-portal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 60%;
  z-index: 95;
  pointer-events: none;
}

.tx-filter-portal > * {
  pointer-events: auto;
}

.transaction {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  height: 100%;
}

.transaction.no-scroll {
  overflow: hidden;
}

.transaction::-webkit-scrollbar {
  display: none;
}

.hero {
  background: linear-gradient(135deg, #3b2f4a 0%, #2a2235 100%);
  border-radius: 1rem;
  padding: 1.5rem;
  flex-shrink: 0;
}

.label {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.month-label {
  margin: 0 0 0.25rem;
  color: var(--text-heading);
  font-size: 0.75rem;
  font-weight: 500;
  opacity: 0.7;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.filtered-badge {
  display: inline-block;
  flex-shrink: 0;
  margin-left: auto;
  padding: 0.1rem 0.4rem;
  border-radius: 0.25rem;
  background: var(--accent, #646cff);
  color: #fff;
  font-size: 0.625rem;
  opacity: 1;
}

.hero .amount {
  margin: 0.25rem 0;
  font-size: 2rem;
  font-weight: 600;
  color: var(--text-heading);
}

.sub {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.8125rem;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.loading-more {
  text-align: center;
  padding: 1rem;
  color: var(--text-muted);
  font-size: 0.8125rem;
}
</style>
