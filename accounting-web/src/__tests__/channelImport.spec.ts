import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchTransactions, importBill } from '../api/client'
import { useChannelStore } from '../stores/channel'
import { useTransactionStore } from '../stores/transaction'

vi.mock('../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
  createChannel: vi.fn(),
  deleteChannel: vi.fn(),
  fetchChannels: vi.fn(),
  importBill: vi.fn(),
  updateChannel: vi.fn(),
  fetchAccounts: vi.fn().mockResolvedValue([]),
  fetchBalanceSheet: vi.fn().mockResolvedValue({ assets: [] }),
  fetchNetWorthTrend: vi.fn(),
  fetchCashFlow: vi.fn(),
  fetchBudgets: vi.fn(),
  fetchBudgetDetail: vi.fn(),
  fetchBudgetStatus: vi.fn(),
  fetchBudgetStatuses: vi.fn().mockResolvedValue([]),
  createBudget: vi.fn(),
  updateBudget: vi.fn(),
  deleteBudget: vi.fn(),
  fetchSavingPlans: vi.fn(),
  fetchSavingPlanStatuses: vi.fn().mockResolvedValue([]),
  fetchSavingPlanStatus: vi.fn(),
  createSavingPlan: vi.fn(),
  updateSavingPlan: vi.fn(),
  deleteSavingPlan: vi.fn(),
}))

const file = new File(['csv'], 'bill.csv')

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  vi.mocked(fetchTransactions).mockResolvedValue([])
})

describe('channel store importFile', () => {
  it('reloads transaction data after a successful import', async () => {
    vi.mocked(importBill).mockResolvedValue({
      imported: 3,
      skipped: 1,
      pending_tag_name: null,
      errors: [],
    })
    const channelStore = useChannelStore()

    const result = await channelStore.importFile(1, file, 1)

    expect(result?.imported).toBe(3)
    expect(fetchTransactions).toHaveBeenCalled()
  })

  it('does not reload transaction data when the import fails', async () => {
    vi.mocked(importBill).mockRejectedValue(new Error('bad file'))
    const channelStore = useChannelStore()
    const txStore = useTransactionStore()
    const reloadSpy = vi.spyOn(txStore, 'reloadAll')

    const result = await channelStore.importFile(1, file, 1)

    expect(result).toBeUndefined()
    expect(reloadSpy).not.toHaveBeenCalled()
  })
})
