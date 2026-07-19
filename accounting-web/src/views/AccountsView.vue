<script setup lang="ts">
import { computed, inject, onMounted, ref, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import { moveAccount } from '../api/client'
import AccountCreateDrawer from '../components/layout/AccountCreateDrawer.vue'
import AccountDrawer from '../components/layout/AccountDrawer.vue'
import AccountGrid from '../components/layout/AccountGrid.vue'
import { panelActionKey } from '../components/layout/panelAction'
import { useAccountDrag, type AccountDropResult } from '../composables/useAccountDrag'
import { useAccountStore } from '../stores/account'
import type { AccountDto } from '../types/api'
import { compileRows, type GridRow } from '../utils/accountGrid'
import { loadSiblingOrder, saveSiblingOrder, sortSiblings } from '../utils/siblingOrder'

const store = useAccountStore()
const { t } = useI18n()

onMounted(() => {
  if (store.accounts.length === 0) {
    store.loadAccounts()
  }
})

const typeLabels = computed<Record<string, string>>(() => ({
  Asset: t('accounts.types.Asset'),
  Income: t('accounts.types.Income'),
  Expense: t('accounts.types.Expense'),
  Equity: t('accounts.types.Equity'),
}))

const typeOrder = ['Asset', 'Income', 'Expense', 'Equity'] as const

const siblingOrder = ref(loadSiblingOrder())

function sortedChildrenOf(parentId: number): AccountDto[] {
  return sortSiblings(store.getChildren(parentId), siblingOrder.value[String(parentId)])
}

function getChildrenOfType(type: string): AccountDto[] {
  const roots = store.groupedAccounts.get(type) ?? []
  const children: AccountDto[] = []
  for (const root of roots) {
    children.push(...sortedChildrenOf(root.id))
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

  // 与卡片交互时收起新建抽屉，避免两个抽屉叠加
  createDrawerVisible.value = false

  if (selectedAccountId.value === clickedId) {
    // 反复点击已选中的卡片：切换其子账户的展开/折叠状态
    if (store.getChildren(clickedId).length > 0) {
      if (expandedPath.value.includes(clickedId)) {
        expandedPath.value = expandedPath.value.filter(x => x !== clickedId)
      } else {
        expandedPath.value = [...expandedPath.value, clickedId]
      }
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
  return compileRows(roots, expandedPath.value, columns, sortedChildrenOf)
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

function onCreateClick() {
  if (!selectedAccount.value) return
  // 先折叠选中账户的编辑抽屉，再展开新建账户抽屉
  drawerVisible.value = false
  createDrawerVisible.value = true
}

const panelAction = inject(panelActionKey, null)
watchEffect(() => {
  if (!panelAction) return
  panelAction.value = {
    label: t('accounts.createAccount'),
    disabled: !selectedAccount.value,
    onClick: onCreateClick,
  }
})

function onAccountCreated() {
  createDrawerVisible.value = false
  store.loadAccounts()
}

const drag = useAccountDrag({
  getAccount: id => store.accounts.find(a => a.id === id),
  getChildren: id => store.getChildren(id),
  isDescendant: (accountId, ancestorId) => store.isDescendant(accountId, ancestorId),
  isExpanded: id => expandedPath.value.includes(id),
  expand: id => {
    if (!expandedPath.value.includes(id)) {
      expandedPath.value = [...expandedPath.value, id]
    }
  },
  onDrop: handleDrop,
})

const draggedAccount = computed(() => {
  if (drag.draggingId.value === null) return null
  return store.accounts.find(a => a.id === drag.draggingId.value) ?? null
})

async function handleDrop(result: AccountDropResult) {
  if (result.kind === 'child') {
    await handleMoveAccount(result.draggedId, result.targetId)
  } else {
    handleSiblingReorder(result.draggedId, result.parentId, result.beforeId)
  }
  // 拖放完成后如同点击了被拖放卡片一样切换选中
  const dragged = store.accounts.find(a => a.id === result.draggedId)
  if (dragged) handleAccountClick(dragged)
}

async function handleMoveAccount(draggedId: number, targetId: number) {
  const dragged = store.accounts.find(a => a.id === draggedId)
  const target = store.accounts.find(a => a.id === targetId)
  if (!dragged || !target) return
  if (dragged.parent_id === targetId) return
  if (store.isDescendant(targetId, draggedId)) return

  const fromType = store.getRootType(dragged)
  const toType = store.getRootType(target)
  if (fromType !== null && toType !== null && fromType !== toType) {
    const typeLabel = typeLabels.value[toType] ?? toType
    if (!confirm(t('accounts.moveTypeChangeConfirm', { type: typeLabel }))) return
  }

  const oldParentId = dragged.parent_id
  store.setAccountParent(draggedId, targetId)
  try {
    const updated = await moveAccount(draggedId, targetId)
    store.refreshAccount(updated)
    if (store.getChildren(targetId).length > 0 && !expandedPath.value.includes(targetId)) {
      expandedPath.value = [...expandedPath.value, targetId]
    }
  } catch (e) {
    store.setAccountParent(draggedId, oldParentId)
    alert(t('accounts.moveFailed', { message: e instanceof Error ? e.message : String(e) }))
  }
}

function handleSiblingReorder(draggedId: number, parentId: number, beforeId: number | null) {
  const key = String(parentId)
  const siblings = sortSiblings(store.getChildren(parentId), siblingOrder.value[key])
  const ids = siblings.map(a => a.id).filter(id => id !== draggedId)
  let index = beforeId === null ? ids.length : ids.indexOf(beforeId)
  if (index === -1) index = ids.length
  ids.splice(index, 0, draggedId)
  siblingOrder.value = { ...siblingOrder.value, [key]: ids }
  saveSiblingOrder(siblingOrder.value)
}
</script>

<template>
  <div class="accounts">
    <div v-if="store.loading" class="loading">{{ t('common.loading') }}</div>
    <div v-else-if="store.error" class="error">{{ store.error }}</div>

    <template v-else>
      <div class="card-area">
        <AccountGrid
          v-for="type in typeOrder"
          :key="type"
          :type-label="typeLabels[type]"
          :rows="rowsForType(type)"
          :selected-account-id="selectedAccountId"
          :dragging-id="drag.draggingId.value"
          :drop-target-id="drag.dropTargetId.value"
          @click="handleAccountClick"
          @drag-start="drag.onCardPointerDown"
          @columns-change="columns => onColumnsChange(type, columns)"
        />
      </div>
    </template>

    <Teleport to="body">
      <div
        v-if="draggedAccount"
        class="drag-ghost"
        :style="{ left: `${drag.dragPosition.value.x}px`, top: `${drag.dragPosition.value.y}px` }"
      >
        {{ draggedAccount.name }}
      </div>

      <div
        v-if="drag.insertion.value"
        class="insert-indicator"
        :style="{
          left: `${drag.insertion.value.left}px`,
          top: `${drag.insertion.value.top}px`,
          height: `${drag.insertion.value.height}px`,
        }"
      />
    </Teleport>

    <Transition name="drawer-slide" :duration="{ enter: 0, leave: 250 }">
      <AccountDrawer
        v-if="drawerVisible && selectedAccount"
        :account="selectedAccount"
        @close="onDrawerClosed"
        @updated="onAccountUpdated"
        @deleted="onAccountDeleted"
      />
    </Transition>

    <Transition name="drawer-slide" :duration="{ enter: 0, leave: 250 }">
      <AccountCreateDrawer
        v-if="createDrawerVisible && selectedAccount"
        :parent-account="selectedAccount"
        @close="createDrawerVisible = false"
        @created="onAccountCreated"
      />
    </Transition>
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

.create-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.create-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

.drag-ghost {
  position: fixed;
  transform: translate(-50%, -50%);
  padding: 0.5rem 0.75rem;
  background: var(--card-bg);
  border: 1px solid var(--accent);
  border-radius: 0.75rem;
  color: var(--text-heading);
  font-size: 0.8125rem;
  pointer-events: none;
  z-index: 200;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}

.insert-indicator {
  position: fixed;
  width: 3px;
  border-radius: 2px;
  background: var(--accent);
  pointer-events: none;
  z-index: 200;
}

:deep(.drawer-slide-leave-active .drawer) {
  animation: drawerSlideDown 0.25s ease forwards;
}

@keyframes drawerSlideDown {
  from {
    transform: translateY(0);
  }
  to {
    transform: translateY(100%);
  }
}
</style>
