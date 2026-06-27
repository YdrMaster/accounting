<script setup lang="ts">
import type { AccountDto } from '../../types/api'
import { useAccountStore } from '../../stores/account'

const props = defineProps<{
  account: AccountDto
  selectedAccountId: number | null
  expandedAccountIds: Set<number>
}>()

const emit = defineEmits<{
  click: [account: AccountDto]
}>()

const store = useAccountStore()

function getChildren(parentId: number): AccountDto[] {
  return store.getChildren(parentId)
}

function isSelected(accountId: number): boolean {
  return props.selectedAccountId === accountId
}

function isExpanded(accountId: number): boolean {
  return props.expandedAccountIds.has(accountId)
}

function hasChildren(accountId: number): boolean {
  return getChildren(accountId).length > 0
}

function onClick() {
  emit('click', props.account)
}

function onChildClick(child: AccountDto) {
  emit('click', child)
}
</script>

<template>
  <div class="account-node">
    <div
      class="account-card"
      :class="{
        selected: isSelected(account.id),
        closed: account.closed_at !== null,
      }"
      @click="onClick"
    >
      <span v-if="hasChildren(account.id)" class="expand-indicator">
        {{ isExpanded(account.id) ? '▾' : '▸' }}
      </span>
      <span class="card-name">{{ account.name }}</span>
    </div>
    <div v-if="isExpanded(account.id)" class="children-container">
      <AccountNode
        v-for="child in getChildren(account.id)"
        :key="child.id"
        :account="child"
        :selected-account-id="selectedAccountId"
        :expanded-account-ids="expandedAccountIds"
        @click="onChildClick"
      />
    </div>
  </div>
</template>

<style scoped>
.account-node {
  display: flex;
  flex-direction: column;
}

.account-card {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.75rem 0.5rem;
  background: var(--card-bg-alt);
  border-radius: 0.75rem;
  cursor: pointer;
  transition: background 0.15s;
  min-height: 3rem;
  width: 5rem;
  flex-shrink: 0;
}

.account-card:hover {
  background: var(--card-bg);
}

.account-card.selected {
  background: var(--accent);
  color: #fff;
}

.account-card.closed {
  opacity: 0.5;
  text-decoration: line-through;
}

.expand-indicator {
  font-size: 0.625rem;
  color: var(--text-muted);
  flex-shrink: 0;
  width: 0.75rem;
  text-align: center;
}

.account-card.selected .expand-indicator {
  color: #fff;
}

.card-name {
  font-size: 0.8125rem;
  color: var(--text-heading);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.account-card.selected .card-name {
  color: #fff;
}

.children-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--card-bg);
  border-radius: 0.75rem;
  margin-top: 0.25rem;
  margin-left: 1rem;
  width: fit-content;
}
</style>
