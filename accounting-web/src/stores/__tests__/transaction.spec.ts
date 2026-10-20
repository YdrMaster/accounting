import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchTransactions } from '../../api/client'
import type { TransactionDto } from '../../types/api'
import { useTransactionStore } from '../transaction'

vi.mock('../../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
}))

vi.mock('../refresh', () => ({
  dataVersion: { value: 0 },
  notifyTransactionsChanged: vi.fn().mockResolvedValue(undefined),
  notifyAccountsChanged: vi.fn().mockResolvedValue(undefined),
}))

function makeTx(id: number, date: string, time = '12:00:00'): TransactionDto {
  return {
    id,
    date_time: `${date}T${time}`,
    description: '',
    kind: 'normal',
    member_id: 1,
    member_name: 'm',
    tags: [],
    pending: false,
    channel_paths: [],
    postings: [],
    is_transfer: false,
  }
}

function paramsOf(callIndex: number): URLSearchParams {
  return vi.mocked(fetchTransactions).mock.calls[callIndex][0] as URLSearchParams
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
})

describe('transaction store 整日加载', () => {
  it('初始加载按整天请求并记录已加载范围', async () => {
    vi.mocked(fetchTransactions).mockResolvedValue([
      makeTx(3, '2026-07-12', '09:00:00'),
      makeTx(2, '2026-07-05', '06:00:00'),
    ])
    const store = useTransactionStore()

    await store.loadInitial('2026-07-12', 100)

    const params = paramsOf(0)
    expect(params.get('to')).toBe('2026-07-12')
    expect(params.get('limit')).toBe('100')
    expect(params.get('full_days')).toBe('true')
    expect(store.loadedRange).toEqual({ from: '2026-07-05', to: '2026-07-12' })
    expect(store.transactions.map(t => t.id)).toEqual([3, 2])
  })

  it('翻页游标是最旧一天的前一天，并追加更早的交易', async () => {
    vi.mocked(fetchTransactions)
      .mockResolvedValueOnce([
        makeTx(3, '2026-07-12', '09:00:00'),
        makeTx(2, '2026-07-05', '06:00:00'),
      ])
      .mockResolvedValueOnce([
        makeTx(1, '2026-07-04', '08:00:00'),
        makeTx(0, '2026-07-03', '20:00:00'),
      ])
    const store = useTransactionStore()

    await store.loadInitial('2026-07-12', 100)
    await store.loadMore()

    const params = paramsOf(1)
    expect(params.get('to')).toBe('2026-07-04')
    expect(params.get('full_days')).toBe('true')
    expect(store.loadedRange).toEqual({ from: '2026-07-03', to: '2026-07-12' })
    expect(store.transactions.map(t => t.id)).toEqual([3, 2, 1, 0])
  })

  it('翻到底（返回空页）即停，范围保持不变', async () => {
    vi.mocked(fetchTransactions)
      .mockResolvedValueOnce([makeTx(1, '2026-04-01', '08:00:00')])
      .mockResolvedValueOnce([])
    const store = useTransactionStore()

    await store.loadInitial('2026-04-01', 100)
    await store.loadMore()

    expect(store.loadedRange).toEqual({ from: '2026-04-01', to: '2026-04-01' })
    expect(store.transactions.map(t => t.id)).toEqual([1])
  })

  it('日历按整天加载单日并写入缓存', async () => {
    vi.mocked(fetchTransactions).mockResolvedValue([makeTx(9, '2026-06-15', '10:00:00')])
    const store = useTransactionStore()

    await store.loadDay('2026-06-15')

    const params = paramsOf(0)
    expect(params.get('from')).toBe('2026-06-15')
    expect(params.get('to')).toBe('2026-06-15')
    expect(params.get('full_days')).toBe('true')
    expect(store.calendarDays.get('2026-06-15')?.map(t => t.id)).toEqual([9])
  })
})
