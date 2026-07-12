<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGridColumns } from '../../composables/useGridColumns'
import type { AccountDto } from '../../types/api'
import type { GridRow } from '../../utils/accountGrid'
import { buildRowTree } from '../../utils/accountGrid'
import AccountRowGroup from './AccountRowGroup.vue'

const props = defineProps<{
  typeLabel: string
  rows: GridRow[]
  selectedAccountId: number | null
}>()

const emit = defineEmits<{
  click: [account: AccountDto]
  columnsChange: [columns: number]
}>()

const tree = computed(() => buildRowTree(props.rows))
const gridRef = ref<HTMLElement | undefined>(undefined)
const { columns, isReady } = useGridColumns(gridRef)

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
      <div ref="gridRef" class="account-grid">
        <AccountRowGroup
          v-for="(node, index) in tree"
          :key="index"
          :node="node"
          :selected-account-id="selectedAccountId"
          @click="emit('click', $event)"
        />
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
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), minmax(0, 1fr));
  gap: 0.5rem;
}

@container (min-width: 600px) {
  .account-grid {
    --grid-columns: 3;
  }
}

@container (min-width: 900px) {
  .account-grid {
    --grid-columns: 4;
  }
}
</style>
