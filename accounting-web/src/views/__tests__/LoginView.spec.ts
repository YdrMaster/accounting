import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiError } from '../../api/client'
import { i18n, setLocale } from '../../i18n'
import { useAuthStore } from '../../stores/auth'
import LoginView from '../LoginView.vue'

vi.mock('../../api/client', async importOriginal => {
  const actual = await importOriginal<typeof import('../../api/client')>()
  return {
    ...actual,
    fetchMe: vi.fn(),
    login: vi.fn(),
    loginTotp: vi.fn(),
    logout: vi.fn(),
  }
})

import { login, loginTotp } from '../../api/client'

const mockedLogin = vi.mocked(login)
const mockedLoginTotp = vi.mocked(loginTotp)

function mountLogin() {
  return mount(LoginView, { global: { plugins: [i18n] } })
}

async function submitPasswordForm(wrapper: ReturnType<typeof mountLogin>) {
  await wrapper.find('form').trigger('submit')
  await flushPromises()
}

describe('LoginView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
  })

  it('登录成功：保存登录状态并进入应用', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: false,
      display_name: 'Alice',
      totp_enabled: false,
    })
    const auth = useAuthStore()
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)

    expect(mockedLogin).toHaveBeenCalledWith('alice', 'secret')
    expect(auth.status).toBe('authed')
    expect(auth.user?.display_name).toBe('Alice')
    expect(wrapper.find('.login-error').exists()).toBe(false)
  })

  it('登录失败：展示服务端统一文案，保留用户名、清空密码', async () => {
    mockedLogin.mockRejectedValue(new ApiError(401, '{"error":"用户名或密码错误"}'))
    const auth = useAuthStore()
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('wrong')
    await submitPasswordForm(wrapper)

    expect(auth.status).not.toBe('authed')
    expect(wrapper.find('.login-error').text()).toBe('用户名或密码错误')
    expect((wrapper.find('#login-username').element as HTMLInputElement).value).toBe('alice')
    expect((wrapper.find('#login-password').element as HTMLInputElement).value).toBe('')
  })

  it('429 限流：提示稍后再试', async () => {
    mockedLogin.mockRejectedValue(new ApiError(429, '{"error":"尝试过于频繁，请稍后再试"}'))
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)

    expect(wrapper.find('.login-error').text()).toBe('尝试过于频繁，请稍后再试')
  })

  it('require_totp：隐藏密码表单，显示动态码输入框', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: true,
      pending_token: 'pending-1',
      display_name: 'Alice',
      totp_enabled: true,
    })
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)

    expect(wrapper.find('#login-password').exists()).toBe(false)
    expect(wrapper.find('#login-code').exists()).toBe(true)
  })

  it('两步登录成功：进入 authed', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: true,
      pending_token: 'pending-1',
      display_name: 'Alice',
      totp_enabled: true,
    })
    mockedLoginTotp.mockResolvedValue({
      require_totp: false,
      display_name: 'Alice',
      totp_enabled: true,
    })
    const auth = useAuthStore()
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)
    await wrapper.find('#login-code').setValue('123456')
    await submitPasswordForm(wrapper)

    expect(mockedLoginTotp).toHaveBeenCalledWith('pending-1', '123456')
    expect(auth.status).toBe('authed')
  })

  it('动态码错误：提示验证码错误并可重试', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: true,
      pending_token: 'pending-1',
      display_name: 'Alice',
      totp_enabled: true,
    })
    mockedLoginTotp.mockRejectedValue(new ApiError(401, '{"error":"验证码错误"}'))
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)
    await wrapper.find('#login-code').setValue('000000')
    await submitPasswordForm(wrapper)

    expect(wrapper.find('.login-error').text()).toBe('验证码错误')
    // 停留在动态码界面，可重试
    expect(wrapper.find('#login-code').exists()).toBe(true)
    expect(wrapper.find('#login-password').exists()).toBe(false)
    expect((wrapper.find('#login-code').element as HTMLInputElement).value).toBe('')
  })

  it('pending 过期：提示重新登录并退回密码表单', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: true,
      pending_token: 'pending-1',
      display_name: 'Alice',
      totp_enabled: true,
    })
    mockedLoginTotp.mockRejectedValue(new ApiError(401, '{"error":"用户名或密码错误"}'))
    const wrapper = mountLogin()

    await wrapper.find('#login-username').setValue('alice')
    await wrapper.find('#login-password').setValue('secret')
    await submitPasswordForm(wrapper)
    await wrapper.find('#login-code').setValue('123456')
    await submitPasswordForm(wrapper)

    expect(wrapper.find('.login-error').text()).toBe('登录已过期，请重新输入密码')
    expect(wrapper.find('#login-password').exists()).toBe(true)
    expect(wrapper.find('#login-code').exists()).toBe(false)
    // 用户名保留
    expect((wrapper.find('#login-username').element as HTMLInputElement).value).toBe('alice')
  })
})
