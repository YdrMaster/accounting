<template>
  <div
    class="account-card"
    :class="cardClass"
    @click="handleClick"
  >
    <div class="card-header">
      <slot name="prefix" />
      <span class="card-name" :title="account.full_name">
        {{ shortName(account.full_name) }}
      </span>
      <slot name="suffix" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Account } from '@/stores/account'

const props = defineProps<{
  account: Account
  selected?: boolean
  expanded?: boolean
}>()

const emit = defineEmits<{
  click: []
}>()

const cardClass = computed(() => ({
  selected: props.selected,
  expanded: props.expanded,
  closed: props.account.closed_at,
  system: props.account.is_system,
}))

function shortName(fullName: string): string {
  return fullName.split(':').pop() || fullName
}

function handleClick() {
  emit('click')
}
</script>

<style scoped>
.account-card {
  width: 140px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
  background: #fff;
  min-height: 50px;
  display: flex;
  align-items: center;
  transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
  user-select: none;
  box-sizing: border-box;
}
.account-card:hover {
  border-color: #91d5ff;
}
.account-card.selected {
  border-color: #1890ff;
  background: #e6f7ff;
}
.account-card.expanded {
  border-color: #1890ff;
}
.account-card.closed {
  opacity: 0.55;
}
.account-card.system {
  border-style: dashed;
}
.card-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
}
.card-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}
</style>
