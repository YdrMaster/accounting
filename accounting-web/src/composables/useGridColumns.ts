import { useResizeObserver } from '@vueuse/core'
import { ref, watch, type Ref } from 'vue'

export function useGridColumns(gridRef: Ref<HTMLElement | undefined>) {
  const columns = ref(2)

  function update() {
    const el = gridRef.value
    if (!el) return
    const raw = getComputedStyle(el).getPropertyValue('--grid-columns') || el.style.getPropertyValue('--grid-columns')
    const value = parseInt(raw.trim(), 10)
    columns.value = Number.isNaN(value) ? 2 : value
  }

  useResizeObserver(gridRef, update)
  watch(() => gridRef.value, update, { immediate: true })

  return { columns }
}
