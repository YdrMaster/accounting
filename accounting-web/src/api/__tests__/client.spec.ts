import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../i18n'
import {
  apiFetch,
  createSavingPlan,
  deleteAccount,
  fetchBudgetStatus,
  fetchBudgetStatuses,
  fetchSavingPlanStatus,
  fetchSavingPlanStatuses,
} from '../client'

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

describe('saving-plan and budget statuses endpoints', () => {
  beforeEach(() => {
    setLocale('zh-CN')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('fetchSavingPlanStatuses appends date and lang', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)

    await fetchSavingPlanStatuses('2026-06-26')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/saving-plans/statuses?date=2026-06-26&lang=zh-CN',
      undefined
    )
  })

  it('fetchSavingPlanStatuses omits date when not given', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)

    await fetchSavingPlanStatuses()

    expect(fetchMock).toHaveBeenCalledWith('/api/saving-plans/statuses?lang=zh-CN', undefined)
  })

  it('fetchSavingPlanStatus appends id, date and lang', async () => {
    const fetchMock = mockFetchOk({})
    vi.stubGlobal('fetch', fetchMock)

    await fetchSavingPlanStatus(5, '2026-06-26')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/saving-plans/5/status?date=2026-06-26&lang=zh-CN',
      undefined
    )
  })

  it('fetchBudgetStatuses appends date and lang', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)

    await fetchBudgetStatuses('2026-06-15')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/budgets/statuses?date=2026-06-15&lang=zh-CN',
      undefined
    )
  })

  it('fetchBudgetStatuses omits date when not given', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)

    await fetchBudgetStatuses()

    expect(fetchMock).toHaveBeenCalledWith('/api/budgets/statuses?lang=zh-CN', undefined)
  })

  it('createSavingPlan posts JSON body and returns the created plan', async () => {
    const created = {
      id: 9,
      name: '旅行基金',
      period: 'monthly',
      deadline: null,
      commodity_id: 1,
      target_amount: '5000',
      account_ids: [1, 2],
    }
    const fetchMock = mockFetchOk(created)
    vi.stubGlobal('fetch', fetchMock)

    const data = {
      name: '旅行基金',
      period: 'monthly',
      deadline: null,
      commodity_id: 1,
      target_amount: '5000',
      account_ids: [1, 2],
    }
    const result = await createSavingPlan(data)

    expect(fetchMock).toHaveBeenCalledWith('/api/saving-plans?lang=zh-CN', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    })
    expect(result).toEqual(created)
  })
})
