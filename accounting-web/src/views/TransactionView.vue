<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'
import { useTransactionStore } from '../stores/transaction'
import { monthOf, todayStr } from '../utils/date'

const txStore = useTransactionStore()
const { t } = useI18n()

const showFormOverlay = ref(false)
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
  editingTxId.value = id
  showFormOverlay.value = true
}

function onDeleteTx(id: number) {
  if (confirm(t('transactions.confirmDelete'))) {
    txStore.remove(id)
  }
}

function onNewTx() {
  editingTxId.value = undefined
  showFormOverlay.value = true
}

function onFormClosed() {
  showFormOverlay.value = false
  editingTxId.value = undefined
}

function onFormSaved() {
  // Data is already updated via create/update in store
}
</script>

<template>
  <div
    ref="scrollContainer"
    class="transaction"
    :class="{ 'no-scroll': showFormOverlay }"
    @scroll="onScroll"
  >
    <template v-if="!showFormOverlay">
      <div class="header-actions">
        <button class="new-tx-btn" @click="onNewTx">+ {{ t('transactions.new') }}</button>
      </div>

      <div class="hero">
        <p class="month-label">
          {{
            txStore.loadedRange
              ? t('transactions.monthLabel', {
                  year: txStore.loadedRange.to.slice(0, 4),
                  month: txStore.loadedRange.to.slice(5, 7),
                })
              : ''
          }}
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

    <TransactionFormOverlay
      v-if="showFormOverlay"
      :edit-id="editingTxId"
      @close="onFormClosed"
      @saved="onFormSaved"
    />
  </div>
</template>

<style scoped>
.transaction {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  position: relative;
  height: 100%;
}

.transaction.no-scroll {
  overflow: hidden;
}

.transaction::-webkit-scrollbar {
  display: none;
}

.header-actions {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}

.new-tx-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.new-tx-btn:hover {
  opacity: 0.9;
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
