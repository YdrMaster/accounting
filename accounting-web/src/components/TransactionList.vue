<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed } from 'vue'
import TransactionCard from './TransactionCard.vue'
import type { TransactionDto } from '../types/api'

const props = defineProps<{
  transactions: TransactionDto[]
}>()

const emit = defineEmits<{
  (e: 'edit', id: number): void
  (e: 'delete', id: number): void
}>()

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
  for (const tx of props.transactions) {
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
  const assetPostings = tx.postings.filter((p) => p.account_type === 'asset')
  const sum = assetPostings.reduce(
    (acc, p) => acc.plus(new Decimal(p.amount)),
    new Decimal(0),
  )
  if (!sum.isZero()) return sum
  return assetPostings.reduce(
    (acc, p) => {
      const a = new Decimal(p.amount)
      return a.gt(0) ? acc.plus(a) : acc
    },
    new Decimal(0),
  )
}
</script>

<template>
  <div class="transaction-list">
    <div v-if="transactions.length === 0" class="empty">暂无交易记录</div>
    <div v-for="group in dayGroups" :key="group.dateLabel" class="day-group">
      <div class="day-header">
        <span>{{ group.dateLabel }} {{ group.weekday }}</span>
        <span class="day-summary">收：¥{{ group.income }} 支：¥{{ group.expense }}</span>
      </div>
      <TransactionCard
        v-for="tx in group.transactions"
        :key="tx.id"
        :tx="tx"
        @edit="emit('edit', $event)"
        @delete="emit('delete', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.transaction-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.day-group {
  display: flex;
  flex-direction: column;
  gap: 0;
  background: var(--card-bg-alt, #252525);
  border-radius: 0.75rem;
  padding: 0.75rem;
  margin-bottom: 0.75rem;
}

.day-header {
  display: flex;
  justify-content: space-between;
  color: var(--text-heading);
  font-weight: 500;
  font-size: 0.8125rem;
  padding: 0.25rem 0.25rem;
  margin-bottom: 0.25rem;
}

.day-summary {
  color: var(--text-muted);
  font-weight: 400;
  font-size: 0.75rem;
}
</style>
