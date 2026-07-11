<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AccountDrawer from '../components/layout/AccountDrawer.vue'
import AccountCreateDrawer from '../components/layout/AccountCreateDrawer.vue'
import AccountGrid from '../components/layout/AccountGrid.vue'
import { useAccountStore } from '../stores/account'
import type { AccountDto } from '../types/api'
import { compileRows, type GridRow } from '../utils/accountGrid'

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

const expandedPath = ref<number[]>([])
const selectedAccountId = ref<number | null>(null)
const drawerVisible = ref(false)
const columnsByType = ref<Record<string, number>>({})

function onColumnsChange(type: string, columns: number) {
  columnsByType.value[type] = columns
}

function isRootAccount(account: AccountDto): boolean {
  return account.parent_id === null
}

function isDescendantOf(accountId: number, ancestorId: number): boolean {
  return store.isDescendant(accountId, ancestorId)
}

function handleAccountClick(account: AccountDto) {
  const clickedId = account.id

  if (selectedAccountId.value === clickedId) {
    if (!account.is_system && !isRootAccount(account)) {
      drawerVisible.value = true
    }
    return
  }

  const onPath = expandedPath.value.includes(clickedId)
  selectedAccountId.value = clickedId

  if (!onPath) {
    const newPath: number[] = []
    for (const id of expandedPath.value) {
      if (isDescendantOf(clickedId, id)) {
        newPath.push(id)
      }
    }
    if (store.getChildren(clickedId).length > 0) {
      newPath.push(clickedId)
    }
    expandedPath.value = newPath
  }

  if (!account.is_system && !isRootAccount(account)) {
    drawerVisible.value = true
  }
}

function rowsForType(type: string): GridRow[] {
  const roots = getChildrenOfType(type)
  const columns = columnsByType.value[type] ?? 2
  return compileRows(roots, expandedPath.value, columns, id => store.getChildren(id))
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
  expandedPath.value = expandedPath.value.filter(x => x !== id)
  if (selectedAccountId.value === id) {
    selectedAccountId.value = null
    drawerVisible.value = false
  }
}

const createDrawerVisible = ref(false)

function onAccountCreated() {
  createDrawerVisible.value = false
  store.loadAccounts()
}
</script>

<template>
  <div class="accounts">
    <div class="header-actions">
      <button class="create-btn" @click="createDrawerVisible = true">+ 新建账户</button>
    </div>

    <div v-if="store.loading" class="loading">加载中...</div>
    <div v-else-if="store.error" class="error">{{ store.error }}</div>

    <template v-else>
      <div class="card-area">
        <AccountGrid
          v-for="type in typeOrder"
          :key="type"
          :type-label="typeLabels[type]"
          :rows="rowsForType(type)"
          :selected-account-id="selectedAccountId"
          @click="handleAccountClick"
          @columns-change="columns => onColumnsChange(type, columns)"
        />
      </div>
    </template>

    <AccountDrawer
      v-if="drawerVisible && selectedAccount"
      :account="selectedAccount"
      @close="onDrawerClosed"
      @updated="onAccountUpdated"
      @deleted="onAccountDeleted"
    />

    <AccountCreateDrawer
      v-if="createDrawerVisible"
      @close="createDrawerVisible = false"
      @created="onAccountCreated"
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

.header-actions {
  display: flex;
  justify-content: flex-end;
  padding: 0.75rem 0.5rem 0;
}

.create-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.create-btn:hover {
  opacity: 0.9;
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
</style>
