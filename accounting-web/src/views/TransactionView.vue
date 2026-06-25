<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, onMounted } from 'vue'
import { useReportStore } from '../stores/report'
import { useTransactionStore } from '../stores/transaction'
import type { TransactionDto } from '../types/api'

const txStore = useTransactionStore()
const reportStore = useReportStore()

onMounted(() => {
  const now = new Date()
  const from = new Date(now.getFullYear(), now.getMonth(), 1)
  const fromStr = formatDate(from)
  const toStr = formatDate(now)
  txStore.fetchTransactions({ from: fromStr, to: toStr })
  reportStore.fetchSummary(fromStr, toStr)
})

function formatDate(d: Date): string {
  return d.toISOString().slice(0, 10)
}

const monthlyExpense = computed(() => reportStore.summary?.expense ?? '0')
const monthlyIncome = computed(() => reportStore.summary?.income ?? '0')
const monthlyBalance = computed(() => {
  const inc = new Decimal(monthlyIncome.value)
  const exp = new Decimal(monthlyExpense.value)
  return inc.minus(exp).toFixed(2)
})

interface DayGroup {
  dateLabel: string
  weekday: string
  income: string
  expense: string
  transactions: TransactionDto[]
}

const WEEKDAYS = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']

const dayGroups = computed<DayGroup[]>(() => {
  const groups = new Map<string, TransactionDto[]>()
  for (const tx of txStore.transactions) {
    const dateStr = tx.date_time.slice(0, 10)
    if (!groups.has(dateStr)) groups.set(dateStr, [])
    groups.get(dateStr)!.push(tx)
  }

  const result: DayGroup[] = []
  for (const [dateStr, txs] of groups) {
    const d = new Date(dateStr + 'T00:00:00')
    const month = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    let dayIncome = new Decimal(0)
    let dayExpense = new Decimal(0)
    for (const tx of txs) {
      const amt = computeAmount(tx)
      if (amt.gt(0)) dayIncome = dayIncome.plus(amt)
      else dayExpense = dayExpense.plus(amt.abs())
    }
    result.push({
      dateLabel: `${month}.${day}`,
      weekday: WEEKDAYS[d.getDay()],
      income: dayIncome.toFixed(2),
      expense: dayExpense.toFixed(2),
      transactions: txs,
    })
  }
  return result
})

function computeAmount(tx: TransactionDto): Decimal {
  const assetPostings = tx.postings.filter(p => p.account_type === 'asset')
  const sum = assetPostings.reduce((acc, p) => acc.plus(new Decimal(p.amount)), new Decimal(0))
  if (!sum.isZero()) return sum
  const positiveSum = assetPostings.reduce((acc, p) => {
    const a = new Decimal(p.amount)
    return a.gt(0) ? acc.plus(a) : acc
  }, new Decimal(0))
  return positiveSum
}

function isTransfer(tx: TransactionDto): boolean {
  return !tx.postings.some(p => p.account_type === 'income' || p.account_type === 'expense')
}

function getIncomeExpenseAccounts(tx: TransactionDto): string[] {
  return tx.postings
    .filter(p => p.account_type === 'income' || p.account_type === 'expense')
    .map(p => shortAccountName(p.account))
}

function getAssetAccounts(tx: TransactionDto): string[] {
  return tx.postings.filter(p => p.account_type === 'asset').map(p => shortAccountName(p.account))
}

function shortAccountName(path: string): string {
  const parts = path.split(':')
  return parts[parts.length - 1] || path
}

function formatAmount(amt: Decimal): string {
  const fixed = amt.toFixed(2)
  const [intPart, decPart] = fixed.split('.')
  const sign = intPart.startsWith('-') ? '-' : ''
  const abs = intPart.replace('-', '')
  const formatted = abs.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${formatted}.${decPart}`
}

function isRefund(tx: TransactionDto): boolean {
  return tx.kind === 'refund'
}
</script>

<template>
  <div class="transaction">
    <div class="hero">
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

    <div v-for="group in dayGroups" :key="group.dateLabel" class="day-group">
      <div class="day-header">
        <span>{{ group.dateLabel }} {{ group.weekday }}</span>
        <span class="day-summary">收：¥{{ group.income }} 支：¥{{ group.expense }}</span>
      </div>
      <div v-for="tx in group.transactions" :key="tx.id" class="tx-card">
        <div class="tx-top">
          <span v-if="isTransfer(tx)" class="transfer-label">转账</span>
          <span v-else class="ie-accounts">{{ getIncomeExpenseAccounts(tx).join(' ') }}</span>
          <div v-if="tx.tags.length" class="tags">
            <span v-for="tag in tx.tags" :key="tag" class="tag">{{ tag }}</span>
          </div>
        </div>
        <div class="tx-middle">
          <div class="tx-info">
            <span class="tx-name" :class="{ refund: isRefund(tx) }">
              {{ isRefund(tx) ? '退款 · ' : '' }}{{ tx.description || tx.member_name || '' }}
            </span>
            <span v-if="tx.member_name && tx.description" class="tx-member">{{
              tx.member_name
            }}</span>
          </div>
          <div
            class="tx-amount"
            :class="{ refund: isRefund(tx), positive: computeAmount(tx).gt(0) }"
          >
            <span v-if="computeAmount(tx).gt(0)">+</span>¥{{ formatAmount(computeAmount(tx)) }}
          </div>
        </div>
        <div class="tx-bottom">
          <span class="asset-accounts">{{ getAssetAccounts(tx).join(' ') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transaction {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.hero {
  background: linear-gradient(135deg, #3b2f4a 0%, #2a2235 100%);
  border-radius: 1rem;
  padding: 1.5rem;
}

.label {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.875rem;
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

.day-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.day-header {
  display: flex;
  justify-content: space-between;
  color: var(--text-heading);
  font-weight: 500;
  font-size: 0.8125rem;
  padding: 0.25rem 0;
}

.day-summary {
  color: var(--text-muted);
  font-weight: 400;
  font-size: 0.75rem;
}

.tx-card {
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.tx-top {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.ie-accounts {
  color: var(--text-heading);
  font-size: 0.8125rem;
  font-weight: 500;
}

.transfer-label {
  color: var(--text-muted);
  font-size: 0.75rem;
  background: var(--border);
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
}

.tags {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
}

.tag {
  color: #e74c3c;
  font-size: 0.6875rem;
  border: 1px solid #e74c3c;
  border-radius: 0.25rem;
  padding: 0 0.375rem;
  line-height: 1.4;
}

.tx-middle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.tx-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.tx-name {
  color: var(--text-heading);
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tx-name.refund {
  color: #999;
}

.tx-member {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.tx-amount {
  color: #e74c3c;
  font-weight: 500;
  font-size: 0.9375rem;
  white-space: nowrap;
  text-align: right;
}

.tx-amount.positive {
  color: #27ae60;
}

.tx-amount.refund {
  color: #999;
}

.tx-bottom {
  display: flex;
  justify-content: flex-end;
}

.asset-accounts {
  color: var(--text-muted);
  font-size: 0.75rem;
  text-align: right;
}
</style>
