import { useWindowSize } from '@vueuse/core'
import { computed } from 'vue'
import { i18n } from '../i18n'

export const paneNames = ['transaction', 'assets', 'accounts', 'calendar', 'plan'] as const

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
  get plan() {
    return i18n.global.t('nav.plan')
  },
}

const { width, height } = useWindowSize()
const ratio = computed(() => width.value / Math.max(1, height.value))

const columns = computed(() => {
  const count = Math.floor(ratio.value / 0.8) + 1
  return Math.max(1, Math.min(paneNames.length, count))
})

const isMobile = computed(() => columns.value === 1)

export function useResponsiveLayout() {
  return {
    width,
    height,
    ratio,
    columns,
    isMobile,
    paneNames,
    paneLabels,
  }
}
