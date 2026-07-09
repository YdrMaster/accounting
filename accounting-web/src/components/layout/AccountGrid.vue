<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { AccountDto } from '../../types/api'
import type { GridRow } from '../../utils/accountGrid'
import { buildRowTree } from '../../utils/accountGrid'
import { useGridColumns } from '../../composables/useGridColumns'
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
const gridRef = ref<HTMLElement | null>(null)
const { columns } = useGridColumns(gridRef)

watch(columns, value => {
  emit('columnsChange', value)
}, { immediate: true })
</script>

<template>
  <div class="type-section">
    <h3 class="type-label">{{ typeLabel }}</h3>
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

.account-grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), 1fr);
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
