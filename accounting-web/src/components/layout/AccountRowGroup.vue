<script setup lang="ts">
import { computed } from 'vue'
import type { RowNode } from '../../utils/accountGrid'
import AccountCard from './AccountCard.vue'

const props = defineProps<{
  node: RowNode
  selectedAccountId: number | null
  draggingId?: number | null
  dropTargetId?: number | null
  dropContainerId?: number | null
}>()

const emit = defineEmits<{
  click: [account: NonNullable<RowNode['row']['items'][number]['account']>]
  containerClick: [accountId: number, event: MouseEvent]
  dragStart: [account: NonNullable<RowNode['row']['items'][number]['account']>, event: PointerEvent]
}>()

// 选中态单源：owner === selectedAccountId 时子容纳框随卡片联动高亮
const childContainerSelected = computed(
  () =>
    props.selectedAccountId != null && props.selectedAccountId === props.node.row.expandedAccountId
)
const childContainerDropTarget = computed(
  () => props.dropContainerId === props.node.row.expandedAccountId
)

const paperBgStyle = computed(() => {
  const d = props.node.row.depth
  const h = 237 + 5 * d
  const s = 76 + 2 * d
  const l = 65 + 3 * d
  const a = +(0.08 + 0.02 * d).toFixed(2)
  return { '--paper-bg': `hsla(${h}, ${s}%, ${l}%, ${a})` }
})

function onContainerClick(event: MouseEvent) {
  const id = props.node.row.expandedAccountId
  if (id !== null) emit('containerClick', id, event)
}
</script>

<template>
  <div class="row" :class="{ 'child-row': node.row.depth > 0 }">
    <AccountCard
      v-for="(item, index) in node.row.items"
      :key="item.account?.id ?? `placeholder-${node.row.depth}-${index}`"
      :item="item"
      :is-selected="!!item.account && item.account.id === selectedAccountId"
      :is-expanded="node.row.expandedAccountId === item.account?.id"
      :is-drag-source="!!item.account && item.account.id === draggingId"
      :is-drop-target="!!item.account && item.account.id === dropTargetId"
      @click="emit('click', $event)"
      @drag-start="(account, event) => emit('dragStart', account, event)"
    />
  </div>
  <div
    v-if="node.children.length"
    class="children-container stacked-children"
    :class="{
      'container-selected': childContainerSelected,
      'container-drop-target': childContainerDropTarget,
    }"
    :style="paperBgStyle"
    :data-container-account-id="node.row.expandedAccountId"
    @click="onContainerClick"
  >
    <AccountRowGroup
      v-for="(child, index) in node.children"
      :key="index"
      :node="child"
      :selected-account-id="selectedAccountId"
      :dragging-id="draggingId"
      :drop-target-id="dropTargetId"
      :drop-container-id="dropContainerId"
      @click="emit('click', $event)"
      @container-click="(accountId, event) => emit('containerClick', accountId, event)"
      @drag-start="(account, event) => emit('dragStart', account, event)"
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
  box-shadow:
    0 -2px 8px rgba(0, 0, 0, 0.25),
    0 -1px 0 rgba(255, 255, 255, 0.04) inset;
}

.children-container.container-selected::after {
  content: '';
  position: absolute;
  inset: -0.15rem -0.35rem;
  z-index: -1;
  background: hsla(237, 76%, 65%, 0.17);
  border-radius: 0.75rem;
  pointer-events: none;
}

.children-container.container-drop-target {
  outline: 2px dashed var(--accent);
  outline-offset: -2px;
  border-radius: 0.75rem;
}
</style>
