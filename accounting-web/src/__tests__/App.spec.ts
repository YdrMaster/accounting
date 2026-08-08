import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../i18n'
import { useAuthStore } from '../stores/auth'

vi.mock('../api/client', () => ({
  setUnauthorizedHandler: vi.fn(),
  fetchMe: vi.fn(),
  login: vi.fn(),
  loginTotp: vi.fn(),
  logout: vi.fn(),
}))

vi.mock('../components/layout/ResponsiveShell.vue', () => ({
  default: { name: 'ResponsiveShell', template: '<div class="shell-stub" />' },
}))

vi.mock('../views/LoginView.vue', () => ({
  default: { name: 'LoginView', template: '<div class="login-stub" />' },
}))

import App from '../App.vue'
import { fetchMe, setUnauthorizedHandler } from '../api/client'

const mockedFetchMe = vi.mocked(fetchMe)
const mockedSetHandler = vi.mocked(setUnauthorizedHandler)

const me = { username: 'alice', display_name: 'Alice', totp_enabled: false }

function mountApp() {
  return mount(App, { global: { plugins: [i18n] } })
}

describe('App 认证门控', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
  })

  it('启动时 status 为 unknown，显示加载态', () => {
    mockedFetchMe.mockReturnValue(new Promise(() => {}))
    const wrapper = mountApp()

    expect(wrapper.find('.boot-loading').exists()).toBe(true)
    expect(wrapper.find('.shell-stub').exists()).toBe(false)
    expect(wrapper.find('.login-stub').exists()).toBe(false)
  })

  it('me 200：显示应用主界面', async () => {
    mockedFetchMe.mockResolvedValue(me)
    const wrapper = mountApp()
    await flushPromises()

    expect(wrapper.find('.shell-stub').exists()).toBe(true)
    expect(wrapper.find('.login-stub').exists()).toBe(false)
  })

  it('me 401：显示登录页', async () => {
    mockedFetchMe.mockRejectedValue(new Error('{"error":"未登录"}'))
    const wrapper = mountApp()
    await flushPromises()

    expect(wrapper.find('.login-stub').exists()).toBe(true)
    expect(wrapper.find('.shell-stub').exists()).toBe(false)
  })

  it('注册 401 回调：会话过期时切换到登录页', async () => {
    mockedFetchMe.mockResolvedValue(me)
    const auth = useAuthStore()
    const wrapper = mountApp()
    await flushPromises()
    expect(wrapper.find('.shell-stub').exists()).toBe(true)

    expect(mockedSetHandler).toHaveBeenCalledTimes(1)
    const handler = mockedSetHandler.mock.calls[0][0]
    handler?.()
    await flushPromises()

    expect(auth.status).toBe('unauthed')
    expect(wrapper.find('.login-stub').exists()).toBe(true)
    expect(wrapper.find('.shell-stub').exists()).toBe(false)
  })
})
