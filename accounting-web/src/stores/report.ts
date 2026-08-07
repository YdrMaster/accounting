import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchBalanceSheet, fetchCashFlow, fetchNetWorthTrend } from '../api/client'
import type { BalanceSheetDto, CashFlowDto, ChartPeriod, NetWorthTrendDto } from '../types/api'

export const useReportStore = defineStore('report', () => {
  const balanceSheet = ref<BalanceSheetDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const netWorthTrend = ref<NetWorthTrendDto | null>(null)
  const trendLoading = ref(false)
  const trendError = ref<string | null>(null)

  const cashFlow = ref<CashFlowDto | null>(null)
  const cashFlowLoading = ref(false)
  const cashFlowError = ref<string | null>(null)

  /** 最近一次加载使用的参数，用于数据变更后的静默重拉 */
  let lastTrendPeriod: ChartPeriod | null = null
  let lastCashFlowQuery: { date: string; period: ChartPeriod } | null = null

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
    lastTrendPeriod = period
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
    lastCashFlowQuery = { date, period }
    cashFlowLoading.value = true
    cashFlowError.value = null
    try {
      cashFlow.value = await fetchCashFlow(date, period)
    } catch (e) {
      cashFlowError.value = e instanceof Error ? e.message : String(e)
    } finally {
      cashFlowLoading.value = false
    }
  }

  function clearCache() {
    balanceSheet.value = null
  }

  /** 数据变更后的静默重拉：只刷新此前加载过的报表，不抛错 */
  async function refresh() {
    const jobs: Promise<unknown>[] = []
    if (balanceSheet.value !== null) jobs.push(loadBalanceSheet(true))
    if (lastTrendPeriod !== null) jobs.push(loadNetWorthTrend(lastTrendPeriod))
    if (lastCashFlowQuery !== null) {
      jobs.push(loadCashFlowTab(lastCashFlowQuery.date, lastCashFlowQuery.period))
    }
    await Promise.allSettled(jobs)
  }

  return {
    balanceSheet,
    loading,
    error,
    loadBalanceSheet,
    clearCache,
    refresh,
    netWorthTrend,
    trendLoading,
    trendError,
    loadNetWorthTrend,
    cashFlow,
    cashFlowLoading,
    cashFlowError,
    loadCashFlowTab,
  }
})
