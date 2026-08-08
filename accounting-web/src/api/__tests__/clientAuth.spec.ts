import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { apiFetch, deleteAccount, setUnauthorizedHandler } from '../client'

function mockFetch(status: number, body = '') {
  return vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    statusText: 'STATUS',
    json: () => Promise.resolve({}),
    text: () => Promise.resolve(body),
  })
}

describe('api client 401 拦截', () => {
  beforeEach(() => {
    setUnauthorizedHandler(null)
  })

  afterEach(() => {
    setUnauthorizedHandler(null)
    vi.unstubAllGlobals()
  })

  it('业务请求 401 触发 unauthorized 回调', async () => {
    vi.stubGlobal('fetch', mockFetch(401, '{"error":"未登录"}'))
    const handler = vi.fn()
    setUnauthorizedHandler(handler)

    await expect(apiFetch('/accounts')).rejects.toThrow()

    expect(handler).toHaveBeenCalledTimes(1)
  })

  it('直接 rawFetch 的业务请求 401 同样触发回调', async () => {
    vi.stubGlobal('fetch', mockFetch(401, '{"error":"未登录"}'))
    const handler = vi.fn()
    setUnauthorizedHandler(handler)

    await expect(deleteAccount(7)).rejects.toThrow()

    expect(handler).toHaveBeenCalledTimes(1)
  })

  it('/auth/* 接口的 401 不触发回调', async () => {
    vi.stubGlobal('fetch', mockFetch(401, '{"error":"用户名或密码错误"}'))
    const handler = vi.fn()
    setUnauthorizedHandler(handler)

    await expect(apiFetch('/auth/login', { method: 'POST', body: '{}' })).rejects.toThrow()

    expect(handler).not.toHaveBeenCalled()
  })

  it('非 401 错误不触发回调', async () => {
    vi.stubGlobal('fetch', mockFetch(500, 'boom'))
    const handler = vi.fn()
    setUnauthorizedHandler(handler)

    await expect(apiFetch('/accounts')).rejects.toThrow()

    expect(handler).not.toHaveBeenCalled()
  })

  it('成功响应不触发回调', async () => {
    vi.stubGlobal('fetch', mockFetch(200))
    const handler = vi.fn()
    setUnauthorizedHandler(handler)

    await apiFetch('/accounts')

    expect(handler).not.toHaveBeenCalled()
  })
})
