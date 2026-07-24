import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  fetchBalanceSheet,
  fetchCashFlow,
  fetchCategoryBreakdown,
  fetchNetWorthTrend,
} from '../api/client'
import type {
  BalanceSheetDto,
  CashFlowDto,
  CategoryBreakdownDto,
  ChartPeriod,
  NetWorthTrendDto,
} from '../types/api'

export const useReportStore = defineStore('report', () => {
  const balanceSheet = ref<BalanceSheetDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const netWorthTrend = ref<NetWorthTrendDto | null>(null)
  const trendLoading = ref(false)
  const trendError = ref<string | null>(null)

  const categoryBreakdown = ref<CategoryBreakdownDto | null>(null)
  const cashFlow = ref<CashFlowDto | null>(null)
  const cashFlowLoading = ref(false)
  const cashFlowError = ref<string | null>(null)

  async function loadBalanceSheet(force = false) {
    if (!force && balanceSheet.value !== null) return

    loading.value = true
    error.value = null
    try {
      balanceSheet.value = await fetchBalanceSheet()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadNetWorthTrend(period: ChartPeriod) {
    trendLoading.value = true
    trendError.value = null
    try {
      netWorthTrend.value = await fetchNetWorthTrend(period)
    } catch (e) {
      trendError.value = e instanceof Error ? e.message : String(e)
    } finally {
      trendLoading.value = false
    }
  }

  async function loadCashFlowTab(date: string, period: ChartPeriod) {
    cashFlowLoading.value = true
    cashFlowError.value = null
    try {
      const [breakdown, flow] = await Promise.all([
        fetchCategoryBreakdown(date, period),
        fetchCashFlow(date, period),
      ])
      categoryBreakdown.value = breakdown
      cashFlow.value = flow
    } catch (e) {
      cashFlowError.value = e instanceof Error ? e.message : String(e)
    } finally {
      cashFlowLoading.value = false
    }
  }

  function clearCache() {
    balanceSheet.value = null
  }

  return {
    balanceSheet,
    loading,
    error,
    loadBalanceSheet,
    clearCache,
    netWorthTrend,
    trendLoading,
    trendError,
    loadNetWorthTrend,
    categoryBreakdown,
    cashFlow,
    cashFlowLoading,
    cashFlowError,
    loadCashFlowTab,
  }
})
