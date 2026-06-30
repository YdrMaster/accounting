<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAccountStore } from '../stores/account'
import type { AccountDto } from '../types/api'
import AccountDrawer from '../components/layout/AccountDrawer.vue'
import AccountNode from '../components/layout/AccountNode.vue'

const store = useAccountStore()

onMounted(() => {
  if (store.accounts.length === 0) {
    store.loadAccounts()
  }
})

const typeLabels: Record<string, string> = {
  Asset: '资产',
  Income: '收入',
  Expense: '支出',
  Equity: '权益',
}

const typeOrder = ['Asset', 'Income', 'Expense', 'Equity'] as const

function getChildrenOfType(type: string): AccountDto[] {
  const roots = store.groupedAccounts.get(type) ?? []
  const children: AccountDto[] = []
  for (const root of roots) {
    children.push(...store.getChildren(root.id))
  }
  return children
}

const selectedAccountId = ref<number | null>(null)
const expandedAccountIds = ref<Set<number>>(new Set())
const drawerVisible = ref(false)

function isRootAccount(account: AccountDto): boolean {
  return account.parent_id === null
}

function isDescendantOf(accountId: number, ancestorId: number): boolean {
  return store.isDescendant(accountId, ancestorId)
}

function handleAccountClick(account: AccountDto) {
  if (isRootAccount(account)) return

  const clickedId = account.id
  const isSystemAccount = account.is_system

  // If clicking the already selected card, just reopen drawer (unless it's a system account)
  if (selectedAccountId.value === clickedId) {
    if (!isSystemAccount) {
      drawerVisible.value = true
    }
    return
  }

  // Toggle expansion: add clicked account to expanded set
  const newExpanded = new Set<number>()

  // Keep old expansions that are ancestors of clicked account
  for (const oldExpandedId of expandedAccountIds.value) {
    if (isDescendantOf(clickedId, oldExpandedId)) {
      newExpanded.add(oldExpandedId)
    }
  }

  // Add clicked account to expanded set (to show its children)
  newExpanded.add(clickedId)

  selectedAccountId.value = clickedId
  expandedAccountIds.value = newExpanded
  
  // Only show drawer for non-system accounts
  if (!isSystemAccount) {
    drawerVisible.value = true
  }
}

const selectedAccount = computed(() => {
  if (selectedAccountId.value === null) return null
  return store.accounts.find(a => a.id === selectedAccountId.value) ?? null
})

function onDrawerClosed() {
  drawerVisible.value = false
}

function onAccountUpdated(updated: AccountDto) {
  store.refreshAccount(updated)
}

function onAccountDeleted(id: number) {
  store.removeAccount(id)
  if (selectedAccountId.value === id) {
    selectedAccountId.value = null
    expandedAccountIds.value.clear()
    drawerVisible.value = false
  }
}
</script>

<template>
  <div class="accounts">
    <div v-if="store.loading" class="loading">加载中...</div>
    <div v-else-if="store.error" class="error">{{ store.error }}</div>

    <template v-else>
      <div class="card-area">
        <div v-for="type in typeOrder" :key="type" class="type-section">
          <h3 class="type-label">{{ typeLabels[type] }}</h3>
          <div class="card-grid">
            <AccountNode
              v-for="account in getChildrenOfType(type)"
              :key="account.id"
              :account="account"
              :selected-account-id="selectedAccountId"
              :expanded-account-ids="expandedAccountIds"
              @click="handleAccountClick"
            />
          </div>
        </div>
      </div>
    </template>

    <AccountDrawer
      v-if="drawerVisible && selectedAccount"
      :account="selectedAccount"
      @close="onDrawerClosed"
      @updated="onAccountUpdated"
      @deleted="onAccountDeleted"
    />
  </div>
</template>

<style scoped>
.accounts {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.card-area {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  padding: 1rem 0.5rem;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.type-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.type-label {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-muted);
  font-weight: 500;
}

.card-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: flex-start;
}
</style>
