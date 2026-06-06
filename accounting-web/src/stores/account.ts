import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Account {
  id: number
  full_name: string
  account_type: string
  parent_id?: number
  is_system: boolean
  billing_day?: number
  repayment_day?: number
}

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<Account[]>([])
  const loading = ref(false)

  async function fetchAccounts() {
    loading.value = true
    try {
      const res = await api.get<Account[]>('/accounts')
      accounts.value = res.data
    } catch (e) {
      console.error('获取账户失败', e)
    } finally {
      loading.value = false
    }
  }

  async function createAccount(fullName: string, billingDay?: number, repaymentDay?: number) {
    await api.post('/accounts', {
      full_name: fullName,
      billing_day: billingDay,
      repayment_day: repaymentDay,
    })
    await fetchAccounts()
  }

  return { accounts, loading, fetchAccounts, createAccount }
})
