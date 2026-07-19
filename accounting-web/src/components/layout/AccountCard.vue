<script setup lang="ts">
import type { GridItem } from '../../utils/accountGrid'

const props = defineProps<{
  item: GridItem
  isExpanded?: boolean
  isSelected?: boolean
  isDragSource?: boolean
  isDropTarget?: boolean
}>()

const emit = defineEmits<{
  click: [account: NonNullable<GridItem['account']>]
  dragStart: [account: NonNullable<GridItem['account']>, event: PointerEvent]
}>()

function onClick() {
  if (!props.item.isPlaceholder && props.item.account) {
    emit('click', props.item.account)
  }
}

function onPointerDown(event: PointerEvent) {
  if (!props.item.isPlaceholder && props.item.account) {
    emit('dragStart', props.item.account, event)
  }
}

function isClosed(item: GridItem): boolean {
  return !item.isPlaceholder && item.account !== null && item.account.closed_at !== null
}
</script>

<template>
  <div
    class="account-card"
    :class="{
      placeholder: item.isPlaceholder,
      selected: !item.isPlaceholder && isSelected,
      closed: isClosed(item),
      'drag-source': isDragSource,
      'drop-target': isDropTarget,
    }"
    :data-account-id="item.account?.id"
    @click="onClick"
    @pointerdown="onPointerDown"
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
  user-select: none;
  -webkit-user-select: none;
  touch-action: pan-y;
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

.account-card.drag-source {
  opacity: 0.4;
}

.account-card.drop-target {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
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
