import { defineStore } from 'pinia'
import { ref } from 'vue'
import { apiFetch } from '../api/client'
import type { TransactionDto } from '../types/api'

export const useTransactionStore = defineStore('transaction', () => {
  const transactions = ref<TransactionDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchTransactions(params?: Record<string, string>) {
    loading.value = true
    error.value = null
    try {
      const qs = params ? '?' + new URLSearchParams(params).toString() : ''
      transactions.value = await apiFetch<TransactionDto[]>(`/transactions${qs}`)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { transactions, loading, error, fetchTransactions }
})
