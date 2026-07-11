<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AccountGrid from './AccountGrid.vue'
import { useAccountStore } from '../../stores/account'
import type { AccountDto } from '../../types/api'
import { compileRows, type GridRow } from '../../utils/accountGrid'

const emit = defineEmits<{
  close: []
  select: [account: AccountDto]
}>()

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

const selectedAccountId = ref<number | null>(null)
const expandedPath = ref<number[]>([])
const columnsByType = ref<Record<string, number>>({})

function getChildrenOfType(type: string): AccountDto[] {
  const roots = store.groupedAccounts.get(type) ?? []
  const children: AccountDto[] = []
  for (const root of roots) {
    children.push(...store.getChildren(root.id))
  }
  return children
}

function isDescendantOf(accountId: number, ancestorId: number): boolean {
  return store.isDescendant(accountId, ancestorId)
}

function handleAccountClick(account: AccountDto) {
  const clickedId = account.id

  if (selectedAccountId.value === clickedId) {
    selectedAccountId.value = null
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
}

function onColumnsChange(type: string, columns: number) {
  columnsByType.value[type] = columns
}

function rowsForType(type: string): GridRow[] {
  const roots = getChildrenOfType(type)
  const columns = columnsByType.value[type] ?? 2
  return compileRows(roots, expandedPath.value, columns, (id) => store.getChildren(id))
}

const selectedAccount = computed(() => {
  if (selectedAccountId.value === null) return null
  return store.accounts.find((a) => a.id === selectedAccountId.value) ?? null
})

function onConfirm() {
  if (selectedAccount.value) {
    emit('select', selectedAccount.value)
    emit('close')
  }
}
</script>

<template>
  <div class="picker-overlay">
    <div class="picker-header">
      <button class="back-btn" @click="emit('close')">← 返回</button>
      <span class="picker-title">选择账户</span>
    </div>

    <div class="picker-body">
      <AccountGrid
        v-for="type in typeOrder"
        :key="type"
        :type-label="typeLabels[type]"
        :rows="rowsForType(type)"
        :selected-account-id="selectedAccountId"
        @click="handleAccountClick"
        @columns-change="(columns) => onColumnsChange(type, columns)"
      />
    </div>

    <div class="picker-footer">
      <button
        class="confirm-btn"
        :disabled="!selectedAccount"
        @click="onConfirm"
      >
        确认选择
      </button>
    </div>
  </div>
</template>

<style scoped>
.picker-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  background: var(--card-bg, #1e1e1e);
  z-index: 200;
  display: flex;
  flex-direction: column;
  animation: slideIn 0.2s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.picker-header {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  gap: 0.75rem;
}

.back-btn {
  background: none;
  border: none;
  color: var(--accent, #646cff);
  font-size: 0.875rem;
  cursor: pointer;
  padding: 0.25rem 0.5rem;
}

.picker-title {
  font-weight: 600;
  color: var(--text-heading);
}

.picker-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.picker-footer {
  padding: 1rem;
  border-top: 1px solid var(--border);
}

.confirm-btn {
  width: 100%;
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.75rem;
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
}

.confirm-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
