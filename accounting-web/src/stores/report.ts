import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiFetch } from '../api/client'
import type { SummaryDto } from '../types/api'

export const useReportStore = defineStore('report', () => {
  const summary = ref<SummaryDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchSummary(from?: string, to?: string) {
    loading.value = true
    error.value = null
    try {
      const params = new URLSearchParams()
      if (from) params.set('from', from)
      if (to) params.set('to', to)
      const qs = params.toString() ? '?' + params.toString() : ''
      summary.value = await apiFetch<SummaryDto>(`/reports/summary${qs}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { summary, loading, error, fetchSummary }
})
