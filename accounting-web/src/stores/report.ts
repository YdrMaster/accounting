import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export const useReportStore = defineStore('report', () => {
  const balanceSheet = ref<unknown>(null)
  const incomeStatement = ref<unknown>(null)
  const stats = ref<unknown>(null)

  async function fetchBalanceSheet() {
    try {
      const res = await api.get('/reports/balance-sheet')
      balanceSheet.value = res.data
    } catch (e) {
      console.error('获取资产负债表失败', e)
    }
  }

  async function fetchIncomeStatement() {
    try {
      const res = await api.get('/reports/income-statement')
      incomeStatement.value = res.data
    } catch (e) {
      console.error('获取损益表失败', e)
    }
  }

  async function fetchStats(by: string, from?: string, to?: string) {
    try {
      const res = await api.get('/reports/stats', { params: { by, from, to } })
      stats.value = res.data
    } catch (e) {
      console.error('获取统计失败', e)
    }
  }

  return { balanceSheet, incomeStatement, stats, fetchBalanceSheet, fetchIncomeStatement, fetchStats }
})
