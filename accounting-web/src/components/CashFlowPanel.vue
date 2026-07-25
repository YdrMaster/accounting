<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CashFlowDetailList from '../components/CashFlowDetailList.vue'
import CategorySunburst from '../components/CategorySunburst.vue'
import PeriodNav from '../components/PeriodNav.vue'
import PeriodSelect from '../components/PeriodSelect.vue'
import { useReportStore } from '../stores/report'
import type { ChartPeriod } from '../types/api'
import { todayStr } from '../utils/date'

const reportStore = useReportStore()
const { t } = useI18n()

const refDate = ref(todayStr())
const period = ref<ChartPeriod>('monthly')

type Side = 'expense' | 'income'
const side = ref<Side>('expense')
/** 当前钻入的账户 id（null = 未下钻）；toggle / 周期 / 日期变化时重置 */
const drillId = ref<number | null>(null)

const items = computed(() =>
  side.value === 'expense'
    ? (reportStore.cashFlow?.expense ?? [])
    : (reportStore.cashFlow?.income ?? [])
)

function load() {
  reportStore.loadCashFlowTab(refDate.value, period.value)
}

function onDrill(accountId: number | null) {
  drillId.value = accountId
}

onMounted(load)
watch([refDate, period], load)
watch([refDate, period, side], () => {
  drillId.value = null
})
</script>

<template>
  <div class="panel">
    <div class="toolbar">
      <PeriodNav v-model="refDate" :period="period" />
      <PeriodSelect v-model="period" />
    </div>

    <div v-if="reportStore.cashFlowError" class="error">{{ reportStore.cashFlowError }}</div>
    <template v-else>
      <div class="card">
        <p v-if="reportStore.cashFlow" class="range">
          {{
            t('assets.cashFlow.periodRange', {
              start: reportStore.cashFlow.period_start,
              end: reportStore.cashFlow.period_end,
            })
          }}
        </p>
        <CategorySunburst :items="items" @drill="onDrill" />
        <div class="toggle" role="tablist">
          <button
            type="button"
            class="toggle-btn"
            :class="{ active: side === 'expense' }"
            @click="side = 'expense'"
          >
            {{ t('assets.category.expense') }}
          </button>
          <button
            type="button"
            class="toggle-btn"
            :class="{ active: side === 'income' }"
            @click="side = 'income'"
          >
            {{ t('assets.category.income') }}
          </button>
        </div>
      </div>

      <div class="card">
        <CashFlowDetailList
          v-if="reportStore.cashFlow"
          :items="items"
          :drill-id="drillId"
          :side="side"
        />
        <div v-else class="loading">{{ t('common.loading') }}</div>
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
  justify-content: space-between;
  align-items: center;
}

.card {
  background: var(--card-bg-alt);
  border-radius: 1rem;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.range {
  margin: 0;
  text-align: center;
  font-size: 0.8125rem;
  color: var(--text-muted);
}

.toggle {
  display: flex;
  justify-content: center;
  gap: 0.25rem;
  background: var(--border);
  border-radius: 9999px;
  padding: 0.25rem;
  width: fit-content;
  margin: 0 auto;
}

.toggle-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  padding: 0.375rem 1.5rem;
  border-radius: 9999px;
  cursor: pointer;
  font-size: 0.875rem;
}

.toggle-btn.active {
  background: var(--card-bg-alt);
  color: var(--text-heading);
  font-weight: 600;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
