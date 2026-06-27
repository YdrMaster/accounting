import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiFetch } from '../api/client'
import type { TransactionDto } from '../types/api'

export const useTransactionStore = defineStore('transaction', () => {
  const transactions = ref<TransactionDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastParams = ref<string | null>(null)

  async function fetchTransactions(params?: Record<string, string>, force = false) {
    const paramKey = params ? new URLSearchParams(params).toString() : ''

    // Skip fetch if we already have data with the same params
    if (!force && transactions.value.length > 0 && lastParams.value === paramKey) {
      return
    }

    loading.value = true
    error.value = null
    try {
      const qs = params ? '?' + paramKey : ''
      transactions.value = await apiFetch<TransactionDto[]>(`/transactions${qs}`)
      lastParams.value = paramKey
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function clearCache() {
    transactions.value = []
    lastParams.value = null
  }

  return { transactions, loading, error, fetchTransactions, clearCache }
})
