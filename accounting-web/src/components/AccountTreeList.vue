<template>
  <div class="account-tree-list">
    <div class="cards-grid">
      <template v-for="item in visibleItems" :key="item.id">
        <AccountCard
          v-if="item.type === 'account'"
          :account="item.account"
          :selected="isSelected(item.account.id)"
          :expanded="isExpanded(item.account.id)"
          @click="handleSelect(item.account)"
        >
          <template v-if="allowDrag" #prefix>
            <span class="drag-handle" @click.stop>⠿</span>
          </template>
          <template #suffix>
            <span v-if="item.account.closed_at" class="closed-tag">
              <a-tag color="default" style="margin: 0; font-size: 11px">已关闭</a-tag>
            </span>
            <span class="card-actions">
              <template v-if="hasChildren(item.account.id)">
                <span class="child-count">{{ childrenCount(item.account.id) }}</span>
                <button
                  type="button"
                  class="expand-btn"
                  :class="{ rotated: isExpanded(item.account.id) }"
                  @click.stop="handleToggleExpand(item.account)"
                >▼</button>
              </template>
              <button
                v-else-if="allowAdd"
                type="button"
                class="add-btn"
                @click.stop="handleAdd(item.account.id)"
              >+</button>
            </span>
          </template>
        </AccountCard>

        <div
          v-else-if="item.type === 'sub-cards'"
          class="sub-cards"
        >
          <div class="sub-cards-arrow"></div>
          <AccountTreeList
            :accounts="accounts"
            :parent-id="item.parentId"
            :type="type"
            :model-value="modelValue"
            :active-id="activeId"
            :mode="mode"
            :allow-add="allowAdd"
            :allow-drag="allowDrag"
            v-model:expanded-stack="expandedStack"
            @update:model-value="emit('update:modelValue', $event)"
            @update:active-id="emit('update:activeId', $event)"
            @add="emit('add', $event)"
            @reorder="emit('reorder', $event)"
          />
        </div>

        <div
          v-else-if="item.type === 'add-card'"
          class="account-card add-card-box"
          @click="handleAdd(item.parentId)"
        >
          <span class="add-card-text">+ 添加</span>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AccountCard from './AccountCard.vue'
import type { Account } from '@/stores/account'

interface AccountItem {
  id: string
  type: 'account'
  account: Account
  parentId: null
}

interface SubCardsItem {
  id: string
  type: 'sub-cards'
  account: null
  parentId: number
}

interface AddCardItem {
  id: string
  type: 'add-card'
  account: null
  parentId: number | null
}

type VisibleItem = AccountItem | SubCardsItem | AddCardItem

const props = defineProps<{
  accounts: Account[]
  parentId: number | null
  type?: string
  modelValue?: number
  activeId?: number | null
  mode: 'select' | 'manage'
  allowAdd: boolean
  allowDrag: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
  'update:activeId': [id: number | null]
  'add': [parentId: number | null]
  'reorder': [ids: number[]]
}>()

const currentLevel = computed(() =>
  props.accounts
    .filter((a) => (a.parent_id ?? null) === props.parentId && (!props.type || a.account_type === props.type))
    .sort((a, b) => a.position - b.position)
)

const expandedStack = defineModel<number[]>('expandedStack', { default: () => [] })

const accountById = computed(() => {
  const map = new Map<number, Account>()
  props.accounts.forEach((a) => map.set(a.id, a))
  return map
})

function hasChildren(id: number): boolean {
  return props.accounts.some((a) => a.parent_id === id)
}

function childrenCount(id: number): number {
  return props.accounts.filter((a) => a.parent_id === id).length
}

function isSelected(id: number): boolean {
  if (props.mode === 'select') return props.modelValue === id
  return props.activeId === id
}

function isExpanded(id: number): boolean {
  return expandedStack.value.includes(id)
}

function ancestorPath(id: number): number[] {
  const path: number[] = []
  let current = accountById.value.get(id)
  while (current && current.parent_id != null) {
    path.unshift(current.parent_id)
    current = accountById.value.get(current.parent_id)
  }
  return path
}

function handleSelect(account: Account) {
  if (props.mode === 'select') {
    emit('update:modelValue', account.id)
  } else {
    emit('update:activeId', account.id)
  }
  expandedStack.value = [...ancestorPath(account.id), account.id]
}

function handleToggleExpand(account: Account) {
  if (isExpanded(account.id)) {
    const index = expandedStack.value.indexOf(account.id)
    expandedStack.value = expandedStack.value.slice(0, index)
  } else {
    expandedStack.value = [...ancestorPath(account.id), account.id]
  }
}

function handleAdd(parentId: number | null) {
  emit('add', parentId)
}

const visibleItems = computed<VisibleItem[]>(() => {
  const items: VisibleItem[] = []
  currentLevel.value.forEach((account) => {
    items.push({ id: `acc-${account.id}`, type: 'account', account, parentId: null })
    if (isExpanded(account.id) && hasChildren(account.id)) {
      items.push({ id: `sub-${account.id}`, type: 'sub-cards', account: null, parentId: account.id })
    }
  })
  if (props.allowAdd) {
    items.push({ id: `add-${props.parentId ?? 'root'}`, type: 'add-card', account: null, parentId: props.parentId })
  }
  return items
})
</script>

<style scoped>
.account-tree-list {
  width: 100%;
}
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: flex-start;
}
.sub-cards {
  --bubble-border: #d9d9d9;
  --bubble-bg: #fafafa;
  order: 1;
  flex-basis: 100%;
  position: relative;
  border: 1px solid var(--bubble-border, #d9d9d9);
  border-radius: 8px;
  background: var(--bubble-bg, #fafafa);
  padding: 12px;
}
.sub-cards-arrow {
  position: absolute;
  top: -8px;
  left: 70px;
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid var(--bubble-border, #d9d9d9);
}
.sub-cards-arrow::after {
  content: '';
  position: absolute;
  top: 1px;
  left: -8px;
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid var(--bubble-bg, #fafafa);
}
.add-card-box {
  width: 140px;
  border: 1px dashed #73d13d;
  color: #73d13d;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  min-height: 50px;
  border-radius: 6px;
  box-sizing: border-box;
}
.add-card-box:hover {
  background: rgba(115, 209, 61, 0.08);
}
.drag-handle {
  cursor: grab;
  color: #999;
  margin-right: 4px;
}
.expand-btn {
  margin-left: 4px;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 10px;
  color: #999;
  transition: transform 0.2s;
}
.expand-btn.rotated {
  transform: rotate(180deg);
}
.child-count {
  font-size: 11px;
  color: #888;
  background: #f0f0f0;
  padding: 1px 6px;
  border-radius: 10px;
  margin-right: 4px;
}
.add-btn {
  background: transparent;
  border: none;
  color: #73d13d;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
}
.closed-tag {
  margin-right: 6px;
}
.card-actions {
  display: flex;
  align-items: center;
  margin-left: auto;
}
</style>
