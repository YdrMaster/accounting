import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Account {
  id: number
  name: string
  account_type: string
  parent_id?: number
  closed_at?: string
  is_system: boolean
  billing_day?: number
  repayment_day?: number
  owner_ids?: number[]
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

  async function createAccount(
    name: string,
    parentId?: number,
    ownerIds: number[] = [],
    billingDay?: number,
    repaymentDay?: number
  ) {
    await api.post('/accounts', {
      name,
      parent_id: parentId,
      owner_ids: ownerIds,
      billing_day: billingDay,
      repayment_day: repaymentDay,
    })
    await fetchAccounts()
  }

  async function renameAccount(id: number, name: string) {
    await api.put(`/accounts/${id}/rename`, { name })
    await fetchAccounts()
  }

  async function closeAccount(id: number) {
    await api.put(`/accounts/${id}/close`)
    await fetchAccounts()
  }

  async function reopenAccount(id: number) {
    await api.put(`/accounts/${id}/open`)
    await fetchAccounts()
  }

  async function deleteAccount(id: number) {
    await api.delete(`/accounts/${id}`)
    await fetchAccounts()
  }

  async function setOwners(accountId: number, ownerIds: number[]) {
    await api.put(`/accounts/${accountId}/owner`, { owner_ids: ownerIds })
    await fetchAccounts()
  }

  return { accounts, loading, fetchAccounts, createAccount, renameAccount, closeAccount, reopenAccount, deleteAccount, setOwners }
})
