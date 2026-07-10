<script setup lang="ts">
import { computed } from 'vue'
import type { RowNode } from '../../utils/accountGrid'
import AccountCard from './AccountCard.vue'

const props = defineProps<{
  node: RowNode
  selectedAccountId: number | null
}>()

const emit = defineEmits<{
  click: [account: NonNullable<RowNode['row']['items'][number]['account']>]
}>()

const paperBgStyle = computed(() => {
  const d = props.node.row.depth
  const h = 237 + 5 * d
  const s = 76 + 2 * d
  const l = 65 + 3 * d
  const a = +(0.08 + 0.02 * d).toFixed(2)
  return { '--paper-bg': `hsla(${h}, ${s}%, ${l}%, ${a})` }
})
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
  <div v-if="node.children.length" class="children-container stacked-children" :style="paperBgStyle">
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
  grid-column: 1 / -1;
  grid-template-columns: subgrid;
  gap: 0.5rem;
}

@supports not (grid-template-columns: subgrid) {
  .row {
    grid-template-columns: repeat(var(--grid-columns, 2), minmax(0, 1fr));
  }
}

.children-container {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: subgrid;
  gap: 0.5rem;
}

@supports not (grid-template-columns: subgrid) {
  .children-container {
    grid-template-columns: repeat(var(--grid-columns, 2), minmax(0, 1fr));
  }
}

.children-container.stacked-children {
  position: relative;
  z-index: 1;
  padding: 0.5rem 0;
  margin: -0.25rem 0;
  background: transparent;
  box-shadow: none;
}

.children-container.stacked-children::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: -0.5rem;
  right: -0.5rem;
  background: var(--paper-bg);
  border-radius: 0.75rem;
  z-index: -1;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.25), 0 -1px 0 rgba(255, 255, 255, 0.04) inset;
}
</style>
