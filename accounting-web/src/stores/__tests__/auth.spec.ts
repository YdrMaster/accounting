import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../api/client', () => ({
  fetchMe: vi.fn(),
  login: vi.fn(),
  loginTotp: vi.fn(),
  logout: vi.fn(),
}))

import { fetchMe, login, loginTotp, logout } from '../../api/client'
import { useAuthStore } from '../auth'

const mockedFetchMe = vi.mocked(fetchMe)
const mockedLogin = vi.mocked(login)
const mockedLoginTotp = vi.mocked(loginTotp)
const mockedLogout = vi.mocked(logout)

const me = { username: 'alice', display_name: 'Alice', totp_enabled: false }

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('init: /auth/me 200 时进入 authed 并保存用户信息', async () => {
    mockedFetchMe.mockResolvedValue(me)
    const store = useAuthStore()

    await store.init()

    expect(store.status).toBe('authed')
    expect(store.user).toEqual(me)
    expect(store.displayName).toBe('Alice')
  })

  it('init: /auth/me 401 时进入 unauthed', async () => {
    mockedFetchMe.mockRejectedValue(new Error('{"error":"未登录"}'))
    const store = useAuthStore()

    await store.init()

    expect(store.status).toBe('unauthed')
    expect(store.user).toBeNull()
  })

  it('login: 无 TOTP 用户直接登录成功', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: false,
      display_name: 'Alice',
      totp_enabled: false,
    })
    const store = useAuthStore()

    const result = await store.login('alice', 'secret')

    expect(result.require_totp).toBe(false)
    expect(store.status).toBe('authed')
    expect(store.user).toEqual(me)
  })

  it('login: require_totp 时保持 unauthed，等待第二步', async () => {
    mockedLogin.mockResolvedValue({
      require_totp: true,
      pending_token: 'pending-1',
      display_name: 'Alice',
      totp_enabled: true,
    })
    const store = useAuthStore()

    const result = await store.login('alice', 'secret')

    expect(result.pending_token).toBe('pending-1')
    expect(store.status).toBe('unknown')
    expect(store.user).toBeNull()
  })

  it('loginTotp: 第二步成功后进入 authed', async () => {
    mockedLoginTotp.mockResolvedValue({
      require_totp: false,
      display_name: 'Alice',
      totp_enabled: true,
    })
    const store = useAuthStore()

    await store.loginTotp('alice', 'pending-1', '123456')

    expect(mockedLoginTotp).toHaveBeenCalledWith('pending-1', '123456')
    expect(store.status).toBe('authed')
    expect(store.user).toEqual({ username: 'alice', display_name: 'Alice', totp_enabled: true })
  })

  it('logout: 调用登出接口并清除本地状态', async () => {
    mockedFetchMe.mockResolvedValue(me)
    mockedLogout.mockResolvedValue(undefined)
    const store = useAuthStore()
    await store.init()
    expect(store.status).toBe('authed')

    await store.logout()

    expect(mockedLogout).toHaveBeenCalledTimes(1)
    expect(store.status).toBe('unauthed')
    expect(store.user).toBeNull()
  })

  it('logout: 接口失败也清除本地状态', async () => {
    mockedFetchMe.mockResolvedValue(me)
    mockedLogout.mockRejectedValue(new Error('network'))
    const store = useAuthStore()
    await store.init()

    await expect(store.logout()).rejects.toThrow('network')

    expect(store.status).toBe('unauthed')
    expect(store.user).toBeNull()
  })

  it('markUnauthed: 会话过期时清除状态', async () => {
    mockedFetchMe.mockResolvedValue(me)
    const store = useAuthStore()
    await store.init()

    store.markUnauthed()

    expect(store.status).toBe('unauthed')
    expect(store.user).toBeNull()
  })

  it('markTotpEnabled: 绑定成功后同步 totp_enabled', async () => {
    mockedFetchMe.mockResolvedValue(me)
    const store = useAuthStore()
    await store.init()

    store.markTotpEnabled()

    expect(store.user?.totp_enabled).toBe(true)
  })
})
