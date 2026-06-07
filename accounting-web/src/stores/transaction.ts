import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Posting {
  id: number
  account: string
  commodity: string
  amount: string
}

export interface Transaction {
  id: number
  date_time: string
  description: string
  member_id?: number
  is_template: boolean
  postings: Posting[]
  tags?: string[]
}

export interface PostingInput {
  account: string
  commodity: string
  amount: string
}

export interface CreateTransactionData {
  date_time: string
  description: string
  member_id?: number
  postings: PostingInput[]
  tags: string[]
}

export const useTransactionStore = defineStore('transaction', () => {
  const transactions = ref<Transaction[]>([])
  const loading = ref(false)

  async function fetchTransactions(params?: Record<string, unknown>) {
    loading.value = true
    try {
      const res = await api.get<Transaction[]>('/transactions', { params })
      transactions.value = res.data
    } catch (e) {
      console.error('获取交易失败', e)
    } finally {
      loading.value = false
    }
  }

  async function createTransaction(data: CreateTransactionData) {
    await api.post('/transactions', data)
  }

  async function updateTransaction(id: number, data: CreateTransactionData) {
    await api.put(`/transactions/${id}`, data)
  }

  async function deleteTransaction(id: number) {
    await api.delete(`/transactions/${id}`)
  }

  return { transactions, loading, fetchTransactions, createTransaction, updateTransaction, deleteTransaction }
})
