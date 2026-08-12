import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  fetchAccounts,
  fetchBalanceSheet,
  fetchBudgetStatuses,
  fetchCashFlow,
  fetchNetWorthTrend,
  fetchSavingPlanStatuses,
  fetchTransactions,
} from '../api/client'
import { useAccountStore } from '../stores/account'
import { useBudgetStore } from '../stores/budget'
import { dataVersion, notifyAccountsChanged, notifyTransactionsChanged } from '../stores/refresh'
import { useReportStore } from '../stores/report'
import { useSavingPlanStore } from '../stores/savingPlan'
import { useTransactionStore } from '../stores/transaction'
import type { TransactionDto } from '../types/api'

vi.mock('../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
  fetchAccounts: vi.fn(),
  fetchBalanceSheet: vi.fn(),
  fetchNetWorthTrend: vi.fn(),
  fetchCashFlow: vi.fn(),
  fetchBudgets: vi.fn(),
  fetchBudgetDetail: vi.fn(),
  fetchBudgetStatus: vi.fn(),
  fetchBudgetStatuses: vi.fn(),
  createBudget: vi.fn(),
  updateBudget: vi.fn(),
  deleteBudget: vi.fn(),
  fetchSavingPlans: vi.fn(),
  fetchSavingPlanStatuses: vi.fn(),
  fetchSavingPlanStatus: vi.fn(),
  createSavingPlan: vi.fn(),
  updateSavingPlan: vi.fn(),
  deleteSavingPlan: vi.fn(),
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
    pending: false,
    channel_paths: [],
    postings: [],
  }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  vi.mocked(fetchBudgetStatuses).mockResolvedValue([])
  vi.mocked(fetchSavingPlanStatuses).mockResolvedValue([])
  vi.mocked(fetchAccounts).mockResolvedValue([])
  vi.mocked(fetchBalanceSheet).mockResolvedValue({ assets: [] })
  vi.mocked(fetchNetWorthTrend).mockResolvedValue({ period: 'monthly', points: [] })
  vi.mocked(fetchCashFlow).mockResolvedValue({
    period_start: '',
    period_end: '',
    income: [],
    expense: [],
  })
  vi.mocked(fetchTransactions).mockResolvedValue([])
})

describe('notifyTransactionsChanged', () => {
  it('reloads budget statuses when they were loaded before', async () => {
    const budgetStore = useBudgetStore()
    await budgetStore.loadStatuses()
    vi.mocked(fetchBudgetStatuses).mockClear()

    await notifyTransactionsChanged()

    expect(fetchBudgetStatuses).toHaveBeenCalledTimes(1)
  })

  it('does not load budget statuses if never loaded', async () => {
    await notifyTransactionsChanged()
    expect(fetchBudgetStatuses).not.toHaveBeenCalled()
  })

  it('reloads saving plan statuses when they were loaded before', async () => {
    const savingPlanStore = useSavingPlanStore()
    await savingPlanStore.loadStatuses()
    vi.mocked(fetchSavingPlanStatuses).mockClear()

    await notifyTransactionsChanged()

    expect(fetchSavingPlanStatuses).toHaveBeenCalledTimes(1)
  })

  it('reloads accounts when they were loaded before', async () => {
    const accountStore = useAccountStore()
    await accountStore.loadAccounts()
    vi.mocked(fetchAccounts).mockClear()

    await notifyTransactionsChanged()

    expect(fetchAccounts).toHaveBeenCalledTimes(1)
  })

  it('does not load accounts if never loaded', async () => {
    await notifyTransactionsChanged()
    expect(fetchAccounts).not.toHaveBeenCalled()
  })

  it('refreshes the balance sheet when it was loaded before', async () => {
    const reportStore = useReportStore()
    await reportStore.loadBalanceSheet()
    vi.mocked(fetchBalanceSheet).mockClear()

    await notifyTransactionsChanged()

    expect(fetchBalanceSheet).toHaveBeenCalledTimes(1)
  })

  it('reloads net worth trend with the last used period', async () => {
    const reportStore = useReportStore()
    await reportStore.loadNetWorthTrend('yearly')
    vi.mocked(fetchNetWorthTrend).mockClear()

    await notifyTransactionsChanged()

    expect(fetchNetWorthTrend).toHaveBeenCalledTimes(1)
    expect(fetchNetWorthTrend).toHaveBeenCalledWith('yearly')
  })

  it('reloads cash flow with the last used query', async () => {
    const reportStore = useReportStore()
    await reportStore.loadCashFlowTab('2026-08-01', 'monthly')
    vi.mocked(fetchCashFlow).mockClear()

    await notifyTransactionsChanged()

    expect(fetchCashFlow).toHaveBeenCalledTimes(1)
    expect(fetchCashFlow).toHaveBeenCalledWith('2026-08-01', 'monthly')
  })

  it('does not touch report APIs if nothing was loaded', async () => {
    await notifyTransactionsChanged()
    expect(fetchBalanceSheet).not.toHaveBeenCalled()
    expect(fetchNetWorthTrend).not.toHaveBeenCalled()
    expect(fetchCashFlow).not.toHaveBeenCalled()
  })

  it('bumps the data version', async () => {
    const before = dataVersion.value
    await notifyTransactionsChanged()
    expect(dataVersion.value).toBeGreaterThan(before)
  })

  it('resolves even when a reload fails', async () => {
    const budgetStore = useBudgetStore()
    await budgetStore.loadStatuses()
    vi.mocked(fetchBudgetStatuses).mockRejectedValue(new Error('network down'))

    await expect(notifyTransactionsChanged()).resolves.toBeUndefined()
  })
})

describe('notifyAccountsChanged', () => {
  it('reloads accounts and refreshes the balance sheet when loaded', async () => {
    const accountStore = useAccountStore()
    const reportStore = useReportStore()
    await accountStore.loadAccounts()
    await reportStore.loadBalanceSheet()
    vi.mocked(fetchAccounts).mockClear()
    vi.mocked(fetchBalanceSheet).mockClear()

    await notifyAccountsChanged()

    expect(fetchAccounts).toHaveBeenCalledTimes(1)
    expect(fetchBalanceSheet).toHaveBeenCalledTimes(1)
  })

  it('bumps the data version', async () => {
    const before = dataVersion.value
    await notifyAccountsChanged()
    expect(dataVersion.value).toBeGreaterThan(before)
  })
})

describe('transaction store reloadAll (import path)', () => {
  it('clears caches, reloads the list and refreshes derived data', async () => {
    const txStore = useTransactionStore()
    const budgetStore = useBudgetStore()
    vi.mocked(fetchTransactions).mockResolvedValue([makeTx(1, '2026-08-01')])
    await txStore.loadDay('2026-08-01')
    expect(txStore.calendarDays.size).toBe(1)
    await budgetStore.loadStatuses()
    vi.mocked(fetchTransactions).mockClear()
    vi.mocked(fetchTransactions).mockResolvedValue([])
    vi.mocked(fetchBudgetStatuses).mockClear()

    await txStore.reloadAll()

    expect(fetchTransactions).toHaveBeenCalled()
    expect(fetchBudgetStatuses).toHaveBeenCalledTimes(1)
  })
})
