import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchAccounts } from '../api/client'
import type { AccountDto } from '../types/api'

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<AccountDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadAccounts() {
    loading.value = true
    error.value = null
    try {
      accounts.value = await fetchAccounts()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function refreshAccount(updated: AccountDto) {
    const idx = accounts.value.findIndex(a => a.id === updated.id)
    if (idx !== -1) {
      accounts.value[idx] = updated
    }
  }

  function removeAccount(id: number) {
    accounts.value = accounts.value.filter(a => a.id !== id)
  }

  const rootAccounts = computed(() => accounts.value.filter(a => a.parent_id === null))

  const accountTypeOrder = ['Asset', 'Income', 'Expense', 'Equity', 'Import'] as const

  const groupedAccounts = computed(() => {
    const rootMap = new Map<string, AccountDto[]>()
    for (const type of accountTypeOrder) {
      rootMap.set(type, [])
    }
    for (const acc of accounts.value) {
      if (acc.parent_id === null) {
        const list = rootMap.get(acc.account_type)
        if (list) list.push(acc)
      }
    }
    return rootMap
  })

  function getChildren(parentId: number): AccountDto[] {
    return accounts.value.filter(a => a.parent_id === parentId)
  }

  function isDescendant(accountId: number, ancestorId: number): boolean {
    let current: AccountDto | undefined = accounts.value.find(a => a.id === accountId)
    while (current) {
      if (current.id === ancestorId) return true
      if (current.parent_id === null) break
      current = accounts.value.find(a => a.id === current!.parent_id)
    }
    return false
  }

  function getRootType(account: AccountDto): string | null {
    let current: AccountDto | undefined = account
    while (current) {
      if (current.parent_id === null) return current.account_type
      current = accounts.value.find(a => a.id === current!.parent_id)
    }
    return null
  }

  function setAccountParent(id: number, parentId: number | null) {
    const acc = accounts.value.find(a => a.id === id)
    if (acc) {
      acc.parent_id = parentId
    }
  }

  return {
    accounts,
    loading,
    error,
    loadAccounts,
    refreshAccount,
    removeAccount,
    rootAccounts,
    groupedAccounts,
    getChildren,
    isDescendant,
    getRootType,
    setAccountParent,
  }
})
