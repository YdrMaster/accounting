<script setup lang="ts">
import Decimal from 'decimal.js'
import { useI18n } from 'vue-i18n'
import type { CashFlowDto } from '../types/api'
import { formatAmount } from '../utils/amount'

defineProps<{ data: CashFlowDto }>()
const { t } = useI18n()
</script>

<template>
  <div class="cash-flow-table">
    <p class="range">
      {{ t('assets.cashFlow.periodRange', { start: data.period_start, end: data.period_end }) }}
    </p>
    <table v-if="data.items.length">
      <thead>
        <tr>
          <th class="account-col">{{ t('assets.cashFlow.account') }}</th>
          <th class="num-col">{{ t('assets.cashFlow.inflow') }}</th>
          <th class="num-col">{{ t('assets.cashFlow.outflow') }}</th>
          <th class="num-col">{{ t('assets.cashFlow.net') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in data.items" :key="item.account">
          <td class="account-col">{{ item.account }}</td>
          <td class="num-col positive">{{ formatAmount(new Decimal(item.inflow)) }}</td>
          <td class="num-col negative">{{ formatAmount(new Decimal(item.outflow)) }}</td>
          <td class="num-col" :class="new Decimal(item.net).lt(0) ? 'negative' : 'positive'">
            {{ formatAmount(new Decimal(item.net)) }}
          </td>
        </tr>
      </tbody>
      <tfoot>
        <tr>
          <td class="account-col">{{ t('assets.cashFlow.total') }}</td>
          <td class="num-col positive">{{ formatAmount(new Decimal(data.total.inflow)) }}</td>
          <td class="num-col negative">{{ formatAmount(new Decimal(data.total.outflow)) }}</td>
          <td class="num-col" :class="new Decimal(data.total.net).lt(0) ? 'negative' : 'positive'">
            {{ formatAmount(new Decimal(data.total.net)) }}
          </td>
        </tr>
      </tfoot>
    </table>
    <div v-else class="empty">{{ t('assets.cashFlow.empty') }}</div>
  </div>
</template>

<style scoped>
.cash-flow-table {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.range {
  margin: 0;
  text-align: center;
  font-size: 0.8125rem;
  color: var(--text-muted);
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

th,
td {
  padding: 0.5rem 0.25rem;
  border-bottom: 1px solid var(--border);
}

th {
  color: var(--text-muted);
  font-weight: 500;
  font-size: 0.8125rem;
}

tfoot td {
  font-weight: 600;
  border-bottom: none;
  border-top: 1px solid var(--border);
}

.account-col {
  text-align: left;
  color: var(--text-heading);
}

.num-col {
  text-align: right;
  white-space: nowrap;
}

.positive {
  color: #4ade80;
}

.negative {
  color: #ff7b7b;
}

.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
