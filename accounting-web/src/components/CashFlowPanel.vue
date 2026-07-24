<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CashFlowTable from '../components/CashFlowTable.vue'
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

function load() {
  reportStore.loadCashFlowTab(refDate.value, period.value)
}

onMounted(load)
watch([refDate, period], load)
</script>

<template>
  <div class="panel">
    <div class="toolbar">
      <PeriodNav v-model="refDate" :period="period" />
      <PeriodSelect v-model="period" />
    </div>

    <div v-if="reportStore.cashFlowError" class="error">{{ reportStore.cashFlowError }}</div>
    <template v-else>
      <div class="card sunbursts">
        <CategorySunburst
          :title="t('assets.category.income')"
          :items="reportStore.categoryBreakdown?.income ?? []"
        />
        <CategorySunburst
          :title="t('assets.category.expense')"
          :items="reportStore.categoryBreakdown?.expense ?? []"
        />
      </div>

      <div class="card">
        <CashFlowTable v-if="reportStore.cashFlow" :data="reportStore.cashFlow" />
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
}

.sunbursts {
  display: flex;
  gap: 1rem;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
