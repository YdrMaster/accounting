import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  createTransaction,
  deleteTransaction,
  fetchTransaction,
  fetchTransactions,
  updateTransaction,
} from '../api/client'
import { notifyTransactionsChanged } from '../stores/refresh'
import { useTransactionStore } from '../stores/transaction'
import type { CreateTransactionData, TransactionDto } from '../types/api'

vi.mock('../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
}))

vi.mock('../stores/refresh', () => ({
  dataVersion: { value: 0 },
  notifyTransactionsChanged: vi.fn().mockResolvedValue(undefined),
  notifyAccountsChanged: vi.fn().mockResolvedValue(undefined),
}))

function makeTx(id: number, date: string): TransactionDto {
  return {
    id,
    date_time: `${date}T12:00:00`,
    description: '',
    kind: 'expense',
    member_id: 1,
    member_name: 'm',
    tags: [],
    channel_paths: [],
    postings: [],
  }
}

function makeCreateData(date: string): CreateTransactionData {
  return {
    date_time: `${date}T12:00:00`,
    description: '',
    kind: 'expense',
    member_id: 1,
    channel_paths: [],
    postings: [],
    tags: [],
  }
}

async function preloadCalendarDay(date: string) {
  const txStore = useTransactionStore()
  vi.mocked(fetchTransactions).mockResolvedValue([makeTx(99, date)])
  await txStore.loadDay(date)
  expect(txStore.calendarDays.size).toBe(1)
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
})

describe('transaction store 变更后的缓存一致性', () => {
  it('create clears calendarDays and notifies derived data', async () => {
    await preloadCalendarDay('2026-08-01')
    vi.mocked(createTransaction).mockResolvedValue(1)
    vi.mocked(fetchTransaction).mockResolvedValue(makeTx(1, '2026-08-01'))
    const txStore = useTransactionStore()

    await txStore.create(makeCreateData('2026-08-01'))

    expect(txStore.calendarDays.size).toBe(0)
    expect(notifyTransactionsChanged).toHaveBeenCalledTimes(1)
  })

  it('update clears calendarDays and notifies derived data', async () => {
    await preloadCalendarDay('2026-08-01')
    vi.mocked(updateTransaction).mockResolvedValue(undefined)
    vi.mocked(fetchTransaction).mockResolvedValue(makeTx(1, '2026-08-02'))
    const txStore = useTransactionStore()

    await txStore.update(1, makeCreateData('2026-08-02'))

    expect(txStore.calendarDays.size).toBe(0)
    expect(notifyTransactionsChanged).toHaveBeenCalledTimes(1)
  })

  it('remove clears calendarDays and notifies derived data', async () => {
    await preloadCalendarDay('2026-08-01')
    vi.mocked(deleteTransaction).mockResolvedValue(undefined)
    const txStore = useTransactionStore()

    await txStore.remove(99)

    expect(txStore.calendarDays.size).toBe(0)
    expect(notifyTransactionsChanged).toHaveBeenCalledTimes(1)
  })
})
