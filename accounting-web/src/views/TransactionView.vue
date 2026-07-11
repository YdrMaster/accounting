<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, onMounted, ref } from 'vue'
import { useTransactionStore } from '../stores/transaction'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'

const txStore = useTransactionStore()

const currentYear = ref(0)
const currentMonth = ref(0)

const showFormOverlay = ref(false)
const editingTxId = ref<number | undefined>(undefined)

onMounted(async () => {
  const now = new Date()
  currentYear.value = now.getFullYear()
  currentMonth.value = now.getMonth() + 1
  await txStore.loadMonth(currentYear.value, currentMonth.value)
  // If current month has no data, auto-load previous month
  const currentData = txStore.getMonthTransactions(currentYear.value, currentMonth.value)
  if (currentData.length === 0) {
    const { year, month } = getPrevMonth(currentYear.value, currentMonth.value)
    currentYear.value = year
    currentMonth.value = month
    await txStore.loadMonth(year, month)
  }
})

function getPrevMonth(year: number, month: number): { year: number; month: number } {
  if (month === 1) return { year: year - 1, month: 12 }
  return { year, month: month - 1 }
}

// Compute monthly summary from transaction data
const monthlyExpense = computed(() => {
  const txs = txStore.getMonthTransactions(currentYear.value, currentMonth.value)
  let sum = new Decimal(0)
  for (const tx of txs) {
    for (const p of tx.postings) {
      if (p.account_type === 'expense') {
        sum = sum.plus(new Decimal(p.amount))
      }
    }
  }
  return sum.toFixed(2)
})

const monthlyIncome = computed(() => {
  const txs = txStore.getMonthTransactions(currentYear.value, currentMonth.value)
  let sum = new Decimal(0)
  for (const tx of txs) {
    for (const p of tx.postings) {
      if (p.account_type === 'income') {
        sum = sum.plus(new Decimal(p.amount))
      }
    }
  }
  return sum.negated().toFixed(2)
})

const monthlyBalance = computed(() => {
  const inc = new Decimal(monthlyIncome.value)
  const exp = new Decimal(monthlyExpense.value)
  return inc.minus(exp).toFixed(2)
})

function formatAmount(amt: Decimal): string {
  const fixed = amt.toFixed(2)
  const [intPart, decPart] = fixed.split('.')
  const sign = intPart.startsWith('-') ? '-' : ''
  const abs = intPart.replace('-', '')
  const formatted = abs.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${formatted}.${decPart}`
}

const scrollContainer = ref<HTMLElement | null>(null)

async function onScroll() {
  const el = scrollContainer.value
  if (!el || txStore.loading) return

  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
  const nearTop = el.scrollTop < 100

  if (nearBottom) {
    const months = Array.from(txStore.loadedMonths).sort()
    if (months.length > 0) {
      const oldest = months[0]
      const [y, m] = oldest.split('-').map(Number)
      const prev = txStore.loadPrevMonth(y, m)
      await prev
    }
  }

  if (nearTop) {
    const months = Array.from(txStore.loadedMonths).sort()
    if (months.length > 0) {
      const newest = months[months.length - 1]
      const [y, m] = newest.split('-').map(Number)
      await txStore.loadNextMonth(y, m)
    }
  }
}

function onEditTx(id: number) {
  editingTxId.value = id
  showFormOverlay.value = true
}

function onDeleteTx(id: number) {
  if (confirm('确定要删除这条交易吗？')) {
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
  // Refresh current month data
  txStore.loadMonth(currentYear.value, currentMonth.value, true)
}
</script>

<template>
  <div ref="scrollContainer" class="transaction" :class="{ 'no-scroll': showFormOverlay }" @scroll="onScroll">
    <!-- Show normal transaction view when form is not displayed -->
    <template v-if="!showFormOverlay">
      <div class="header-actions">
        <button class="new-tx-btn" @click="onNewTx">+ 新建交易</button>
      </div>

      <div class="hero">
        <p class="month-label">{{ currentYear }}年{{ currentMonth }}月</p>
        <p class="label">月支出</p>
        <p class="amount">¥{{ formatAmount(new Decimal(monthlyExpense)) }}</p>
        <p class="sub">
          月收入 ¥{{ formatAmount(new Decimal(monthlyIncome)) }} · 本月结余 ¥{{
            formatAmount(new Decimal(monthlyBalance))
          }}
        </p>
      </div>

      <div v-if="txStore.loading" class="loading">加载中...</div>
      <div v-else-if="txStore.error" class="error">{{ txStore.error }}</div>

      <TransactionList
        :transactions="txStore.transactions"
        @edit="onEditTx"
        @delete="onDeleteTx"
      />
    </template>

    <!-- Show form overlay when editing/creating - completely replaces transaction view -->
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
</style>
