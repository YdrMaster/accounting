<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGridColumns } from '../../composables/useGridColumns'
import type { AccountDto } from '../../types/api'
import type { GridRow } from '../../utils/accountGrid'
import { buildRowTree } from '../../utils/accountGrid'
import AccountRowGroup from './AccountRowGroup.vue'

const props = defineProps<{
  typeLabel: string
  /** 分区绑定的根账户；缺省时网格为纯展示（无容器选中/拖放语义），如账户选择器 */
  rootAccount?: AccountDto | null
  rows: GridRow[]
  selectedAccountId: number | null
  draggingId?: number | null
  dropTargetId?: number | null
  dropContainerId?: number | null
}>()

const emit = defineEmits<{
  click: [account: AccountDto]
  containerClick: [accountId: number, event: MouseEvent]
  dragStart: [account: AccountDto, event: PointerEvent]
  columnsChange: [columns: number]
}>()

const tree = computed(() => buildRowTree(props.rows))
const gridRef = ref<HTMLElement | undefined>(undefined)
const { columns, isReady } = useGridColumns(gridRef)

// 容器视觉仅存在于账户页（rootAccount 存在）；选中态单源：selectedAccountId === owner
const isSelectedContainer = computed(
  () => props.rootAccount != null && props.selectedAccountId === props.rootAccount.id
)
const isDropContainer = computed(
  () => props.rootAccount != null && props.dropContainerId === props.rootAccount.id
)

function onGridClick(event: MouseEvent) {
  if (props.rootAccount) {
    emit('containerClick', props.rootAccount.id, event)
  }
}

watch(
  [columns, isReady],
  () => {
    if (isReady.value) {
      emit('columnsChange', columns.value)
    }
  },
  { immediate: true }
)
</script>

<template>
  <div class="type-section">
    <h3 class="type-label">{{ typeLabel }}</h3>
    <div class="highlight-container depth-0" :style="{ '--paper-bg': 'hsla(237, 76%, 65%, 0.08)' }">
      <div
        ref="gridRef"
        class="account-grid"
        :class="{
          'container-selected': isSelectedContainer,
          'container-drop-target': isDropContainer,
        }"
        :data-container-account-id="rootAccount?.id"
        @click="onGridClick"
      >
        <AccountRowGroup
          v-for="(node, index) in tree"
          :key="index"
          :node="node"
          :selected-account-id="selectedAccountId"
          :dragging-id="draggingId"
          :drop-target-id="dropTargetId"
          :drop-container-id="dropContainerId"
          @click="emit('click', $event)"
          @container-click="(accountId, event) => emit('containerClick', accountId, event)"
          @drag-start="(account, event) => emit('dragStart', account, event)"
        />
        <div v-if="rows.length === 0" class="grid-empty-placeholder" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.type-section {
  container-type: inline-size;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.type-label {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-muted);
  font-weight: 500;
}

.highlight-container {
  position: relative;
  z-index: 1;
  padding: 0.5rem 0;
  margin: -0.25rem 0;
  background: transparent;
  box-shadow: none;
}

.highlight-container::before {
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

.account-grid {
  position: relative;
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), minmax(0, 1fr));
  gap: 0.5rem;
}

@container (min-width: 450px) {
  .account-grid {
    --grid-columns: 3;
  }
}

@container (min-width: 675px) {
  .account-grid {
    --grid-columns: 4;
  }
}

.account-grid.container-selected::after {
  content: '';
  position: absolute;
  inset: -0.15rem -0.35rem;
  z-index: -1;
  background: hsla(237, 76%, 65%, 0.17);
  border-radius: 0.75rem;
  pointer-events: none;
}

.account-grid.container-drop-target {
  outline: 2px dashed var(--accent);
  outline-offset: -2px;
  border-radius: 0.75rem;
}

.grid-empty-placeholder {
  grid-column: 1 / -1;
  min-height: 3.25rem;
  border: 1.5px dashed var(--border);
  border-radius: 0.75rem;
}

.account-grid.container-selected .grid-empty-placeholder {
  border-color: transparent;
  background: hsla(237, 76%, 65%, 0.17);
}
</style>
