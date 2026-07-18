import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../i18n'
import { apiFetch, deleteAccount, fetchBudgetStatus } from '../client'

function mockFetchOk(body: unknown = []) {
  return vi.fn().mockResolvedValue({
    ok: true,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(''),
  })
}

describe('api client lang parameter', () => {
  beforeEach(() => {
    setLocale('zh-CN')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('appends the current locale as lang query param on apiFetch', async () => {
    const fetchMock = mockFetchOk()
    vi.stubGlobal('fetch', fetchMock)

    await apiFetch('/accounts')

    expect(fetchMock).toHaveBeenCalledWith('/api/accounts?lang=zh-CN', undefined)
  })

  it('appends lang to raw fetch helpers too', async () => {
    const fetchMock = mockFetchOk()
    vi.stubGlobal('fetch', fetchMock)

    await deleteAccount(7)

    expect(fetchMock).toHaveBeenCalledWith('/api/accounts/7?lang=zh-CN', { method: 'DELETE' })
  })

  it('uses & when the path already has a query string', async () => {
    const fetchMock = mockFetchOk({})
    vi.stubGlobal('fetch', fetchMock)

    await fetchBudgetStatus(3, '2026-07-01')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/budgets/3/status?date=2026-07-01&lang=zh-CN',
      undefined
    )
  })

  it('uses the new locale for subsequent requests after a language switch', async () => {
    const fetchMock = mockFetchOk()
    vi.stubGlobal('fetch', fetchMock)

    setLocale('en')
    await apiFetch('/tags')

    expect(fetchMock).toHaveBeenCalledWith('/api/tags?lang=en', undefined)
    expect(i18n.global.locale.value).toBe('en')
  })
})
