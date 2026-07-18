import { useWindowSize } from '@vueuse/core'
import { computed, ref, watch } from 'vue'
import { i18n } from '../i18n'

export const paneNames = ['transaction', 'assets', 'accounts', 'calendar', 'budget'] as const

export type PaneName = (typeof paneNames)[number]

export const paneLabels: Record<PaneName, string> = {
  get transaction() {
    return i18n.global.t('nav.transaction')
  },
  get assets() {
    return i18n.global.t('nav.assets')
  },
  get accounts() {
    return i18n.global.t('nav.accounts')
  },
  get calendar() {
    return i18n.global.t('nav.calendar')
  },
  get budget() {
    return i18n.global.t('nav.budget')
  },
}

export function useResponsiveLayout() {
  const { width, height } = useWindowSize()
  const ratio = computed(() => width.value / Math.max(1, height.value))

  const columns = computed(() => {
    const count = Math.floor(ratio.value / 0.8) + 1
    return Math.max(1, Math.min(paneNames.length, count))
  })

  const isMobile = computed(() => columns.value === 1)

  const startIndex = ref(0)
  const maxStart = computed(() => Math.max(0, paneNames.length - columns.value))

  watch(columns, () => {
    startIndex.value = Math.min(startIndex.value, maxStart.value)
  })

  const activeIndex = computed(() => startIndex.value)

  function shiftLeft() {
    startIndex.value = startIndex.value <= 0 ? maxStart.value : startIndex.value - 1
  }

  function shiftRight() {
    startIndex.value = startIndex.value >= maxStart.value ? 0 : startIndex.value + 1
  }

  function goTo(index: number) {
    startIndex.value = Math.max(0, Math.min(maxStart.value, index))
  }

  return {
    width,
    height,
    ratio,
    columns,
    isMobile,
    startIndex,
    activeIndex,
    maxStart,
    shiftLeft,
    shiftRight,
    goTo,
    paneNames,
    paneLabels,
  }
}
