import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiFetch } from '../api/client'
import type { SummaryDto } from '../types/api'

export const useReportStore = defineStore('report', () => {
  const summary = ref<SummaryDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastParams = ref<string | null>(null)

  async function fetchSummary(from?: string, to?: string, force = false) {
    const params = new URLSearchParams()
    if (from) params.set('from', from)
    if (to) params.set('to', to)
    const paramKey = params.toString()

    // Skip fetch if we already have data with the same params
    if (!force && summary.value !== null && lastParams.value === paramKey) {
      return
    }

    loading.value = true
    error.value = null
    try {
      const qs = paramKey ? '?' + paramKey : ''
      summary.value = await apiFetch<SummaryDto>(`/reports/summary${qs}`)
      lastParams.value = paramKey
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function clearCache() {
    summary.value = null
    lastParams.value = null
  }

  return { summary, loading, error, fetchSummary, clearCache }
})
