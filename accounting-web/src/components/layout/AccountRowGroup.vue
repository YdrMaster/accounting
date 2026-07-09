<script setup lang="ts">
import type { RowNode } from '../../utils/accountGrid'
import AccountCard from './AccountCard.vue'

const props = defineProps<{
  node: RowNode
  selectedAccountId: number | null
}>()

const emit = defineEmits<{
  click: [account: NonNullable<RowNode['row']['items'][number]['account']>]
}>()
</script>

<template>
  <div class="row" :class="{ 'child-row': node.row.depth > 0 }">
    <AccountCard
      v-for="(item, index) in node.row.items"
      :key="item.account?.id ?? `placeholder-${node.row.depth}-${index}`"
      :item="item"
      :is-selected="!!item.account && item.account.id === selectedAccountId"
      :is-expanded="node.row.expandedAccountId === item.account?.id"
      @click="emit('click', $event)"
    />
  </div>
  <div v-if="node.children.length" class="children-container">
    <AccountRowGroup
      v-for="(child, index) in node.children"
      :key="index"
      :node="child"
      :selected-account-id="selectedAccountId"
      @click="emit('click', $event)"
    />
  </div>
</template>

<style scoped>
.row {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), 1fr);
  gap: 0.5rem;
}

.children-container {
  box-shadow: -2px 0 0 var(--accent);
  background: rgba(100, 108, 255, 0.05);
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;
}
</style>
