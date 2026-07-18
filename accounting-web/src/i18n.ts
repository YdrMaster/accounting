import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zhCN from './locales/zh-CN'

export const LANG_STORAGE_KEY = 'accounting-lang'

export type AppLocale = 'en' | 'zh-CN'

export const SUPPORTED_LOCALES: AppLocale[] = ['en', 'zh-CN']

function normalizeLocale(raw: string | null | undefined): AppLocale | null {
  if (!raw) return null
  if (raw.toLowerCase().startsWith('zh')) return 'zh-CN'
  if (SUPPORTED_LOCALES.includes(raw as AppLocale)) return raw as AppLocale
  return null
}

export function getStoredLocale(): AppLocale | null {
  return normalizeLocale(localStorage.getItem(LANG_STORAGE_KEY))
}

export function detectLocale(): AppLocale {
  return normalizeLocale(navigator.language) ?? 'en'
}

export function getInitialLocale(): AppLocale {
  return getStoredLocale() ?? detectLocale()
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: 'en',
  messages: {
    en,
    'zh-CN': zhCN,
  },
})

export function getCurrentLocale(): AppLocale {
  return i18n.global.locale.value as AppLocale
}

export function setLocale(locale: AppLocale): void {
  i18n.global.locale.value = locale
  localStorage.setItem(LANG_STORAGE_KEY, locale)
}
