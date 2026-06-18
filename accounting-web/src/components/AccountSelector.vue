<template>
  <div class="account-selector">
    <div v-if="currentLevel.length === 0" class="empty-state">
      暂无账户
    </div>
    <div v-else class="cards-grid">
      <div
        v-for="account in currentLevel"
        :key="account.id"
        class="card-line"
      >
        <AccountCard
          :account="account"
          :selected="modelValue === account.id"
          :expanded="isExpanded(account.id)"
          @click="handleSelect(account)"
        >
          <template v-if="hasChildren(account.id)" #suffix>
            <button
              class="expand-btn"
              :class="{ rotated: isExpanded(account.id) }"
              @click.stop="toggleExpand(account.id)"
            >
              ▼
            </button>
          </template>
        </AccountCard>
        <div
          v-if="isExpanded(account.id) && hasChildren(account.id)"
          class="sub-cards"
        >
          <AccountSelector
            :accounts="accounts"
            :parent-id="account.id"
            :type="type"
            :model-value="modelValue"
            v-model:expanded-set="expandedSet"
            @update:model-value="emit('update:modelValue', $event)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AccountCard from './AccountCard.vue'
import type { Account } from '@/stores/account'

const props = defineProps<{
  accounts: Account[]
  parentId: number | null
  type?: string
  modelValue?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
}>()

const currentLevel = computed(() =>
  props.accounts
    .filter((a) => a.parent_id === props.parentId && (!props.type || a.account_type === props.type))
    .sort((a, b) => a.position - b.position)
)

const expandedSet = defineModel<Set<number>>('expandedSet', { default: () => new Set() })

function hasChildren(id: number): boolean {
  return props.accounts.some((a) => a.parent_id === id)
}

function isExpanded(id: number): boolean {
  return expandedSet.value.has(id)
}

function toggleExpand(id: number) {
  const next = new Set(expandedSet.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  expandedSet.value = next
}

function handleSelect(account: Account) {
  emit('update:modelValue', account.id)
}
</script>

<style scoped>
.account-selector {
  width: 100%;
}
.empty-state {
  color: #999;
  text-align: center;
  padding: 24px 0;
  font-size: 13px;
}
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  min-height: 36px;
  align-items: flex-start;
}
.card-line {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 6px;
  flex: 1 1 140px;
  min-width: 140px;
}
.card-line :deep(.account-card) {
  flex: 1;
  min-width: 0;
}
.expand-btn {
  align-self: center;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 10px;
  color: #999;
  transition: transform 0.2s;
  padding: 4px;
  flex-shrink: 0;
}
.expand-btn.rotated {
  transform: rotate(180deg);
}
.sub-cards {
  order: 1;
  flex-basis: 100%;
  position: relative;
  border: 1px solid var(--bubble-border, #d9d9d9);
  border-radius: 8px;
  background: var(--bubble-bg, #fafafa);
  padding: 12px;
}
</style>
