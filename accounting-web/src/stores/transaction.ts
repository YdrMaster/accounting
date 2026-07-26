import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  createTransaction as apiCreateTransaction,
  deleteTransaction as apiDeleteTransaction,
  fetchTransaction as apiFetchTransaction,
  fetchTransactions as apiFetchTransactions,
  updateTransaction as apiUpdateTransaction,
} from '../api/client'
import type { CreateTransactionData, TransactionDto, TxFilters } from '../types/api'
import { dateOf, formatDate, todayStr } from '../utils/date'
import { buildTxQuery, isFilterActive } from '../utils/txFilter'

interface LoadedRange {
  from: string
  to: string
}

const EXPAND_CAP_WHEN_FILTERED = 3

export const useTransactionStore = defineStore('transaction', () => {
  const loadedRange = ref<LoadedRange | null>(null)
  const transactions = ref<TransactionDto[]>([])
  const calendarDays = ref<Map<string, TransactionDto[]>>(new Map())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const activeFilter = ref<TxFilters | null>(null)
  let requestId = 0

  const filterActive = computed(() => isFilterActive(activeFilter.value))

  const byDateDesc = (a: TransactionDto, b: TransactionDto) =>
    b.date_time.localeCompare(a.date_time)

  const allTransactions = computed(() => {
    const seen = new Set<number>()
    const result: TransactionDto[] = []
    for (const tx of transactions.value) {
      if (!seen.has(tx.id)) {
        seen.add(tx.id)
        result.push(tx)
      }
    }
    const outside: TransactionDto[] = []
    for (const [dayStr, dayTxs] of calendarDays.value) {
      if (loadedRange.value && dayStr >= loadedRange.value.from && dayStr <= loadedRange.value.to) {
        continue
      }
      for (const tx of dayTxs) {
        if (!seen.has(tx.id)) {
          seen.add(tx.id)
          outside.push(tx)
        }
      }
    }
    if (outside.length > 0) {
      outside.sort(byDateDesc)
      const merged: TransactionDto[] = []
      let i = 0,
        j = 0
      while (i < result.length && j < outside.length) {
        if (result[i].date_time >= outside[j].date_time) {
          merged.push(result[i++])
        } else {
          merged.push(outside[j++])
        }
      }
      while (i < result.length) merged.push(result[i++])
      while (j < outside.length) merged.push(outside[j++])
      return merged
    }
    return result
  })

  const transactionDates = computed(() => {
    const dates = new Set<string>()
    for (const tx of transactions.value) {
      dates.add(dateOf(tx))
    }
    for (const [dayStr] of calendarDays.value) {
      dates.add(dayStr)
    }
    return dates
  })

  async function expandSameDay(
    toDate: string,
    initialData: TransactionDto[],
    initialLimit: number,
    capped: boolean
  ): Promise<TransactionDto[]> {
    let currentLimit = initialLimit * 2
    let expansions = 1
    let data = initialData
    while (data.length > 0) {
      const newest = dateOf(data[0])
      const oldest = dateOf(data[data.length - 1])
      if (newest !== oldest) break
      if (capped && expansions >= EXPAND_CAP_WHEN_FILTERED) break
      const params = buildTxQuery(activeFilter.value, { to: toDate, limit: String(currentLimit) })
      data = await apiFetchTransactions(params)
      currentLimit *= 2
      expansions++
    }
    return data
  }

  async function loadInitial(toDate?: string, limit = 100) {
    const id = ++requestId
    loading.value = true
    error.value = null
    const to = toDate ?? activeFilter.value?.to ?? todayStr()
    try {
      const params = buildTxQuery(activeFilter.value, { to, limit: String(limit) })
      const data = await apiFetchTransactions(params)
      if (id !== requestId) return
      if (data.length === 0) {
        loadedRange.value = null
        transactions.value = []
        return
      }
      const newest = dateOf(data[0])
      const oldest = dateOf(data[data.length - 1])
      let finalData = data
      if (newest === oldest) {
        finalData = await expandSameDay(to, data, limit, filterActive.value)
        if (id !== requestId) return
      }
      if (finalData.length === 0) {
        loadedRange.value = null
        transactions.value = []
        return
      }
      transactions.value = finalData
      loadedRange.value = {
        from: dateOf(finalData[finalData.length - 1]),
        to: dateOf(finalData[0]),
      }
    } catch (e) {
      if (id !== requestId) return
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      if (id === requestId) loading.value = false
    }
  }

  async function loadMore() {
    if (!loadedRange.value || loading.value) return
    const { from } = loadedRange.value
    if (activeFilter.value?.from && from <= activeFilter.value.from) return
    const [y, m, d] = from.split('-').map(Number)
    const prevDay = new Date(y, m - 1, d - 1)
    const prevDayStr = formatDate(prevDay)
    loading.value = true
    error.value = null
    try {
      const params = buildTxQuery(activeFilter.value, { to: prevDayStr, limit: '100' })
      const data = await apiFetchTransactions(params)
      if (data.length === 0) return
      const oldest = dateOf(data[data.length - 1])
      if (oldest === prevDayStr) {
        const expanded = await expandSameDay(prevDayStr, data, 100, filterActive.value)
        if (expanded.length > 0) {
          transactions.value.push(...expanded)
          loadedRange.value = {
            from: dateOf(expanded[expanded.length - 1]),
            to: loadedRange.value.to,
          }
        }
      } else {
        transactions.value.push(...data)
        loadedRange.value = { from: oldest, to: loadedRange.value.to }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadDay(date: string) {
    if (calendarDays.value.has(date)) return
    if (loadedRange.value && date >= loadedRange.value.from && date <= loadedRange.value.to) return
    loading.value = true
    error.value = null
    try {
      const params = new URLSearchParams({ from: date, to: date })
      const data = await apiFetchTransactions(params)
      calendarDays.value.set(date, data)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function setFilter(filter: TxFilters | null) {
    activeFilter.value = filter
    transactions.value = []
    loadedRange.value = null
    loadInitial()
  }

  function clearFilter() {
    setFilter(null)
  }

  async function create(data: CreateTransactionData): Promise<number> {
    const id = await apiCreateTransaction(data)
    const tx = await apiFetchTransaction(id)
    transactions.value = [tx, ...transactions.value]
    if (loadedRange.value) {
      const txDate = dateOf(tx)
      if (txDate > loadedRange.value.to) {
        loadedRange.value = { ...loadedRange.value, to: txDate }
      }
    }
    return id
  }

  async function update(id: number, data: CreateTransactionData): Promise<void> {
    await apiUpdateTransaction(id, data)
    const tx = await apiFetchTransaction(id)
    const idx = transactions.value.findIndex(t => t.id === id)
    if (idx !== -1) {
      transactions.value.splice(idx, 1)
    }
    let lo = 0,
      hi = transactions.value.length
    while (lo < hi) {
      const mid = (lo + hi) >>> 1
      if (transactions.value[mid].date_time > tx.date_time) lo = mid + 1
      else hi = mid
    }
    transactions.value.splice(lo, 0, tx)
  }

  async function remove(id: number): Promise<void> {
    await apiDeleteTransaction(id)
    transactions.value = transactions.value.filter(t => t.id !== id)
    for (const [key, txs] of calendarDays.value) {
      const idx = txs.findIndex(t => t.id === id)
      if (idx !== -1) {
        if (txs.length === 1) {
          calendarDays.value.delete(key)
        } else {
          txs.splice(idx, 1)
        }
        break
      }
    }
  }

  function clearCache() {
    transactions.value = []
    calendarDays.value.clear()
    loadedRange.value = null
  }

  return {
    transactions: allTransactions,
    transactionDates,
    loading,
    error,
    loadedRange,
    calendarDays,
    activeFilter,
    filterActive,
    loadInitial,
    loadMore,
    loadDay,
    setFilter,
    clearFilter,
    create,
    update,
    remove,
    clearCache,
  }
})
