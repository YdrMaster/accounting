<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CashFlowItemDto } from '../types/api'
import { formatAmount } from '../utils/amount'
import { buildDetailRows } from '../utils/cashFlowList'

const props = defineProps<{
  items: CashFlowItemDto[]
  /** 当前钻入的账户 id（null = 未下钻，显示一级分类） */
  drillId: number | null
  /** 当前收支侧：支出金额红色、收入金额绿色 */
  side: 'expense' | 'income'
}>()

const { t } = useI18n()

const rows = computed(() => buildDetailRows(props.items, props.drillId))

function percent(ratio: number): string {
  return `${(ratio * 100).toFixed(2)}%`
}
</script>

<template>
  <div class="detail-list">
    <template v-if="rows.length">
      <div
        v-for="row in rows"
        :key="row.accountId"
        class="row"
        :style="{ paddingLeft: `${row.depth * 1.25}rem` }"
      >
        <div class="line">
          <span class="name">
            <span class="dot"></span>
            {{ row.name }}
          </span>
          <span class="percent">{{ percent(row.ratio) }}</span>
          <span class="amount" :class="side">{{ formatAmount(new Decimal(row.amount)) }}</span>
        </div>
        <div class="bar-track">
          <div class="bar" :style="{ width: `${Math.min(row.ratio * 100, 100)}%` }"></div>
        </div>
      </div>
    </template>
    <div v-else class="empty">{{ t('assets.cashFlow.empty') }}</div>
  </div>
</template>

<style scoped>
.detail-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  box-sizing: border-box;
}

.line {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-heading);
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.dot {
  flex: none;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: var(--color-info-soft);
}

.percent {
  color: var(--text-muted);
  font-size: 0.8125rem;
  white-space: nowrap;
}

.amount {
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.amount.expense {
  color: var(--color-expense-soft);
}

.amount.income {
  color: var(--color-income-soft);
}

.bar-track {
  height: 0.25rem;
  border-radius: 9999px;
  background: var(--border);
  overflow: hidden;
}

.bar {
  height: 100%;
  border-radius: 9999px;
  background: var(--color-info-soft);
}

.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
