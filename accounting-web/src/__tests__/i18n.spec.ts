import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it } from 'vitest'
import { defineComponent, nextTick } from 'vue'
import { getInitialLocale, i18n, LANG_STORAGE_KEY, setLocale } from '../i18n'

describe('i18n', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('falls back to browser language when nothing is stored', () => {
    // happy-dom navigator.language is en-US
    expect(getInitialLocale()).toBe('en')
  })

  it('reads the initial locale from localStorage', () => {
    localStorage.setItem(LANG_STORAGE_KEY, 'zh-CN')
    expect(getInitialLocale()).toBe('zh-CN')
  })

  it('normalizes any zh-* browser language to zh-CN', () => {
    localStorage.setItem(LANG_STORAGE_KEY, 'zh-TW')
    expect(getInitialLocale()).toBe('zh-CN')
  })

  it('setLocale updates the active locale and persists it', () => {
    setLocale('zh-CN')
    expect(i18n.global.locale.value).toBe('zh-CN')
    expect(localStorage.getItem(LANG_STORAGE_KEY)).toBe('zh-CN')
    setLocale('en')
    expect(i18n.global.locale.value).toBe('en')
    expect(localStorage.getItem(LANG_STORAGE_KEY)).toBe('en')
  })

  it('re-renders component text immediately when the locale changes', async () => {
    const Comp = defineComponent({
      template: '<span>{{ $t("nav.accounts") }}</span>',
    })
    const wrapper = mount(Comp, { global: { plugins: [i18n] } })

    setLocale('zh-CN')
    await nextTick()
    expect(wrapper.text()).toBe('账户')

    setLocale('en')
    await nextTick()
    expect(wrapper.text()).toBe('Accounts')
  })
})
