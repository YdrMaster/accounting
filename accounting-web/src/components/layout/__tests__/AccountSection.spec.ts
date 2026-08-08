import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../../i18n'
import { useAuthStore } from '../../../stores/auth'
import AccountSection from '../AccountSection.vue'

vi.mock('../../../api/client', async importOriginal => {
  const actual = await importOriginal<typeof import('../../../api/client')>()
  return {
    ...actual,
    fetchMe: vi.fn(),
    login: vi.fn(),
    loginTotp: vi.fn(),
    logout: vi.fn(),
    totpSetup: vi.fn(),
    totpEnable: vi.fn(),
  }
})

vi.mock('qrcode', () => ({
  default: { toDataURL: vi.fn().mockResolvedValue('data:image/png;base64,qr') },
}))

import QRCode from 'qrcode'
import { logout, totpEnable, totpSetup } from '../../../api/client'

const mockedLogout = vi.mocked(logout)
const mockedTotpSetup = vi.mocked(totpSetup)
const mockedTotpEnable = vi.mocked(totpEnable)
const mockedQr = vi.mocked(QRCode.toDataURL)

const me = { username: 'alice', display_name: 'Alice', totp_enabled: false }

async function mountSection() {
  const auth = useAuthStore()
  auth.status = 'authed'
  auth.user = { ...me }
  const wrapper = mount(AccountSection, { global: { plugins: [i18n] } })
  await flushPromises()
  return wrapper
}

describe('AccountSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
  })

  it('显示当前用户 display_name 与用户名', async () => {
    const wrapper = await mountSection()

    expect(wrapper.text()).toContain('Alice')
    expect(wrapper.text()).toContain('alice')
  })

  it('登出：调用登出接口并清除登录状态', async () => {
    mockedLogout.mockResolvedValue(undefined)
    const auth = useAuthStore()
    const wrapper = await mountSection()

    await wrapper.find('.logout-btn').trigger('click')
    await flushPromises()

    expect(mockedLogout).toHaveBeenCalledTimes(1)
    expect(auth.status).toBe('unauthed')
    expect(auth.user).toBeNull()
  })

  it('TOTP 绑定：setup 渲染二维码，enable 后展示 8 个恢复码', async () => {
    mockedTotpSetup.mockResolvedValue({ otpauth_uri: 'otpauth://totp/Accounting:alice?secret=ABC' })
    mockedTotpEnable.mockResolvedValue({
      recovery_codes: ['c1', 'c2', 'c3', 'c4', 'c5', 'c6', 'c7', 'c8'],
    })
    const auth = useAuthStore()
    const wrapper = await mountSection()

    await wrapper.find('.totp-start').trigger('click')
    await flushPromises()

    expect(mockedTotpSetup).toHaveBeenCalledTimes(1)
    expect(mockedQr).toHaveBeenCalledWith('otpauth://totp/Accounting:alice?secret=ABC', {
      width: 200,
      margin: 1,
    })
    expect((wrapper.find('.totp-qr').element as HTMLImageElement).src).toContain(
      'data:image/png;base64,qr'
    )

    await wrapper.find('.totp-card .field-input').setValue('123456')
    await wrapper.find('.totp-card .add-row .add-btn').trigger('click')
    await flushPromises()

    expect(mockedTotpEnable).toHaveBeenCalledWith('123456')
    const codes = wrapper.findAll('.recovery-code')
    expect(codes).toHaveLength(8)
    expect(codes[0].text()).toBe('c1')
    expect(wrapper.text()).toContain('恢复码仅此一次显示')
    expect(auth.user?.totp_enabled).toBe(true)
  })

  it('TOTP enable 失败：展示服务端错误文案', async () => {
    mockedTotpSetup.mockResolvedValue({ otpauth_uri: 'otpauth://totp/Accounting:alice?secret=ABC' })
    mockedTotpEnable.mockRejectedValue(new Error('{"error":"验证码错误"}'))
    const wrapper = await mountSection()

    await wrapper.find('.totp-start').trigger('click')
    await flushPromises()
    await wrapper.find('.totp-card .field-input').setValue('000000')
    await wrapper.find('.totp-card .add-row .add-btn').trigger('click')
    await flushPromises()

    expect(wrapper.find('.store-error').text()).toBe('验证码错误')
    expect(wrapper.findAll('.recovery-code')).toHaveLength(0)
  })
})
