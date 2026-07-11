import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  fetchTransactions as apiFetchTransactions,
  fetchTransaction as apiFetchTransaction,
  createTransaction as apiCreateTransaction,
  updateTransaction as apiUpdateTransaction,
  deleteTransaction as apiDeleteTransaction,
} from '../api/client'
import type { TransactionDto, CreateTransactionData } from '../types/api'

function monthKey(year: number, month: number): string {
  return `${year}-${String(month).padStart(2, '0')}`
}

function prevMonthKey(year: number, month: number): { year: number; month: number } {
  if (month === 1) return { year: year - 1, month: 12 }
  return { year, month: month - 1 }
}

function nextMonthKey(year: number, month: number): { year: number; month: number } {
  if (month === 12) return { year: year + 1, month: 1 }
  return { year, month: month + 1 }
}

export const useTransactionStore = defineStore('transaction', () => {
  const loadedMonths = ref<Set<string>>(new Set())
  const transactionsByMonth = ref<Map<string, TransactionDto[]>>(new Map())
  const loading = ref(false)
  const error = ref<string | null>(null)

  const allTransactions = computed(() => {
    const all: TransactionDto[] = []
    const sortedKeys = Array.from(transactionsByMonth.value.keys()).sort().reverse()
    for (const key of sortedKeys) {
      all.push(...(transactionsByMonth.value.get(key) || []))
    }
    return all
  })

  function getMonthTransactions(year: number, month: number): TransactionDto[] {
    const key = monthKey(year, month)
    return transactionsByMonth.value.get(key) || []
  }

  function isMonthLoaded(year: number, month: number): boolean {
    return loadedMonths.value.has(monthKey(year, month))
  }

  async function loadMonth(year: number, month: number, force = false) {
    const key = monthKey(year, month)
    if (!force && loadedMonths.value.has(key)) return

    loading.value = true
    error.value = null
    try {
      const from = `${key}-01`
      const next = nextMonthKey(year, month)
      const to = `${monthKey(next.year, next.month)}-01`
      const data = await apiFetchTransactions({ from, to })
      transactionsByMonth.value.set(key, data)
      loadedMonths.value.add(key)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadPrevMonth(currentYear: number, currentMonth: number) {
    const prev = prevMonthKey(currentYear, currentMonth)
    await loadMonth(prev.year, prev.month)
    return prev
  }

  async function loadNextMonth(currentYear: number, currentMonth: number) {
    const next = nextMonthKey(currentYear, currentMonth)
    await loadMonth(next.year, next.month)
    return next
  }

  async function create(data: CreateTransactionData): Promise<number> {
    const id = await apiCreateTransaction(data)
    const date = new Date(data.date_time)
    const key = monthKey(date.getFullYear(), date.getMonth() + 1)
    const tx = await apiFetchTransaction(id)
    const existing = transactionsByMonth.value.get(key) || []
    transactionsByMonth.value.set(key, [tx, ...existing])
    return id
  }

  async function update(id: number, data: CreateTransactionData): Promise<void> {
    await apiUpdateTransaction(id, data)
    const date = new Date(data.date_time)
    const key = monthKey(date.getFullYear(), date.getMonth() + 1)
    const tx = await apiFetchTransaction(id)
    for (const [mKey, txs] of transactionsByMonth.value) {
      const idx = txs.findIndex((t) => t.id === id)
      if (idx !== -1) {
        txs.splice(idx, 1)
        if (mKey !== key) {
          transactionsByMonth.value.set(mKey, [...txs])
        }
        break
      }
    }
    const target = transactionsByMonth.value.get(key) || []
    target.push(tx)
    transactionsByMonth.value.set(key, target)
  }

  async function remove(id: number): Promise<void> {
    await apiDeleteTransaction(id)
    for (const [key, txs] of transactionsByMonth.value) {
      const idx = txs.findIndex((t) => t.id === id)
      if (idx !== -1) {
        txs.splice(idx, 1)
        transactionsByMonth.value.set(key, [...txs])
        break
      }
    }
  }

  function clearCache() {
    transactionsByMonth.value.clear()
    loadedMonths.value.clear()
  }

  return {
    transactions: allTransactions,
    loading,
    error,
    loadedMonths,
    loadMonth,
    loadPrevMonth,
    loadNextMonth,
    isMonthLoaded,
    getMonthTransactions,
    create,
    update,
    remove,
    clearCache,
  }
})
