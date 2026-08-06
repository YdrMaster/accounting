import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../../i18n'
import { useAccountStore } from '../../../stores/account'
import AccountPicker from '../AccountPicker.vue'
import AccountPickerOverlay from '../AccountPickerOverlay.vue'

vi.mock('../../../api/client', () => ({
  fetchAccounts: vi.fn().mockResolvedValue([]),
}))

function mountPicker(accountType?: 'asset' | 'expense') {
  return mount(AccountPicker, {
    props: { modelValue: null, accountType },
    global: { plugins: [i18n], stubs: { teleport: true } },
  })
}

describe('AccountPicker', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
    const accountStore = useAccountStore()
    accountStore.accounts = []
  })

  it('打开 Overlay 时透传 accountType prop', async () => {
    const wrapper = mountPicker('asset')
    await wrapper.find('.picker-trigger').trigger('click')
    await flushPromises()

    expect(wrapper.findComponent(AccountPickerOverlay).props('accountType')).toBe('asset')
  })

  it('不传 accountType 时 Overlay 收到 undefined（行为不变）', async () => {
    const wrapper = mountPicker()
    await wrapper.find('.picker-trigger').trigger('click')
    await flushPromises()

    expect(wrapper.findComponent(AccountPickerOverlay).props('accountType')).toBeUndefined()
  })
})
