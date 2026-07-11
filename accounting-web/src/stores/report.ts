import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchBalanceSheet } from '../api/client'
import type { BalanceSheetDto } from '../types/api'

export const useReportStore = defineStore('report', () => {
  const balanceSheet = ref<BalanceSheetDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

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

  function clearCache() {
    balanceSheet.value = null
  }

  return { balanceSheet, loading, error, loadBalanceSheet, clearCache }
})
