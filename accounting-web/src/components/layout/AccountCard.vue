<script setup lang="ts">
import type { GridItem } from '../../utils/accountGrid'

const props = defineProps<{
  item: GridItem
  isExpanded?: boolean
  isSelected?: boolean
}>()

const emit = defineEmits<{
  click: [account: NonNullable<GridItem['account']>]
}>()

function onClick() {
  if (!props.item.isPlaceholder && props.item.account) {
    emit('click', props.item.account)
  }
}
</script>

<template>
  <div
    class="account-card"
    :class="{
      placeholder: item.isPlaceholder,
      selected: !item.isPlaceholder && isSelected,
      closed: !item.isPlaceholder && item.account.closed_at !== null,
    }"
    @click="onClick"
  >
    <template v-if="!item.isPlaceholder && item.account">
      <span v-if="item.hasChildren" class="expand-indicator">
        {{ isExpanded ? '▾' : '▸' }}
      </span>
      <span class="card-name">{{ item.account.name }}</span>
    </template>
  </div>
</template>

<style scoped>
.account-card {
  min-height: 3rem;
  padding: 0.75rem 0.5rem;
  background: var(--card-bg-alt);
  border-radius: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  overflow: hidden;
  transition: background 0.15s;
}

.account-card:hover {
  background: var(--card-bg);
}

.account-card.placeholder {
  background: transparent;
  cursor: default;
  pointer-events: none;
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
}

.account-card.selected .card-name {
  color: #fff;
}
</style>
