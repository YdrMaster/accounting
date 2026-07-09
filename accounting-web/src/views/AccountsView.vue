<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AccountDrawer from '../components/layout/AccountDrawer.vue'
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

  selectedAccountId.value = clickedId

  const hasChildren = store.getChildren(clickedId).length > 0
  if (hasChildren) {
    const existingIndex = expandedPath.value.indexOf(clickedId)
    if (existingIndex !== -1) {
      expandedPath.value = expandedPath.value.slice(0, existingIndex + 1)
    } else {
      const newPath: number[] = []
      for (const id of expandedPath.value) {
        if (isDescendantOf(clickedId, id)) {
          newPath.push(id)
        }
      }
      newPath.push(clickedId)
      expandedPath.value = newPath
    }
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
</script>

<template>
  <div class="accounts">
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
</style>
