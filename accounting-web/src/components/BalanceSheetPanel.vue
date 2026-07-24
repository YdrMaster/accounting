<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import NetWorthTrendChart from '../components/NetWorthTrendChart.vue'
import PeriodSelect from '../components/PeriodSelect.vue'
import { useCommodityStore } from '../stores/commodity'
import { useReportStore } from '../stores/report'
import type { ChartPeriod } from '../types/api'
import { formatAmount } from '../utils/amount'

const reportStore = useReportStore()
const commodityStore = useCommodityStore()
const { t } = useI18n()

const trendPeriod = ref<ChartPeriod>('monthly')

onMounted(() => {
  reportStore.loadBalanceSheet()
  commodityStore.load()
  reportStore.loadNetWorthTrend(trendPeriod.value)
})

watch(trendPeriod, p => reportStore.loadNetWorthTrend(p))

const totalAssets = computed(() => {
  if (!reportStore.balanceSheet) return new Decimal(0)
  let sum = new Decimal(0)
  for (const item of reportStore.balanceSheet.assets) {
    for (const b of item.balances) {
      const amt = new Decimal(b.amount)
      if (amt.gt(0)) sum = sum.plus(amt)
    }
  }
  return sum
})

const totalLiabilities = computed(() => {
  if (!reportStore.balanceSheet) return new Decimal(0)
  let sum = new Decimal(0)
  for (const item of reportStore.balanceSheet.assets) {
    for (const b of item.balances) {
      const amt = new Decimal(b.amount)
      if (amt.lt(0)) sum = sum.plus(amt.abs())
    }
  }
  return sum
})

const netWorth = computed(() => totalAssets.value.minus(totalLiabilities.value))

const positiveAccounts = computed(() => {
  if (!reportStore.balanceSheet) return []
  return reportStore.balanceSheet.assets.filter(item =>
    item.balances.some(b => new Decimal(b.amount).gt(0))
  )
})

const negativeAccounts = computed(() => {
  if (!reportStore.balanceSheet) return []
  return reportStore.balanceSheet.assets.filter(item =>
    item.balances.some(b => new Decimal(b.amount).lt(0))
  )
})

function commoditySymbol(id: number): string {
  return commodityStore.commodities.find(c => c.id === id)?.symbol ?? `#${id}`
}
</script>

<template>
  <div class="panel">
    <div class="toolbar">
      <PeriodSelect v-model="trendPeriod" />
    </div>

    <div class="card">
      <NetWorthTrendChart :points="reportStore.netWorthTrend?.points ?? []" :period="trendPeriod" />
    </div>

    <div v-if="reportStore.loading" class="loading">{{ t('common.loading') }}</div>
    <div v-else-if="reportStore.error" class="error">{{ reportStore.error }}</div>
    <template v-else>
      <div class="net-worth">
        <p class="label">{{ t('assets.netWorth') }}</p>
        <p class="amount">¥{{ formatAmount(netWorth) }}</p>
        <div class="row">
          <span>{{ t('assets.totalAssets') }} ¥{{ formatAmount(totalAssets) }}</span>
          <span>{{ t('assets.totalLiabilities') }} -¥{{ formatAmount(totalLiabilities) }}</span>
        </div>
      </div>

      <div v-if="positiveAccounts.length" class="card">
        <h3>{{ t('nav.assets') }}</h3>
        <div v-for="item in positiveAccounts" :key="item.account" class="account-item">
          <span class="account-name">{{ item.account }}</span>
          <div class="balances">
            <span
              v-for="b in item.balances.filter(b => new Decimal(b.amount).gt(0))"
              :key="b.commodity_id"
              class="balance positive"
            >
              {{ formatAmount(new Decimal(b.amount)) }} {{ commoditySymbol(b.commodity_id) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="negativeAccounts.length" class="card">
        <h3>{{ t('assets.liabilities') }}</h3>
        <div v-for="item in negativeAccounts" :key="item.account" class="account-item">
          <span class="account-name">{{ item.account }}</span>
          <div class="balances">
            <span
              v-for="b in item.balances.filter(b => new Decimal(b.amount).lt(0))"
              :key="b.commodity_id"
              class="balance negative"
            >
              {{ formatAmount(new Decimal(b.amount)) }} {{ commoditySymbol(b.commodity_id) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="!positiveAccounts.length && !negativeAccounts.length" class="empty">
        {{ t('assets.noData') }}
      </div>
    </template>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.toolbar {
  display: flex;
  justify-content: flex-end;
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.net-worth {
  text-align: center;
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1.5rem;
}

.net-worth .label {
  margin: 0;
  color: var(--text-muted);
}

.net-worth .amount {
  margin: 0.5rem 0;
  font-size: 2rem;
  font-weight: 600;
  color: var(--text-heading);
}

.net-worth .row {
  display: flex;
  justify-content: space-around;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.card {
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.card h3 {
  margin: 0 0 0.25rem;
  color: var(--text-heading);
  font-size: 0.9375rem;
}

.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border);
}

.account-item:last-child {
  border-bottom: none;
}

.account-name {
  color: var(--text-heading);
  font-size: 0.875rem;
}

.balances {
  display: flex;
  gap: 0.75rem;
}

.balance {
  font-weight: 500;
  font-size: 0.875rem;
  white-space: nowrap;
}

.balance.positive {
  color: #4ade80;
}

.balance.negative {
  color: #ff7b7b;
}
</style>
