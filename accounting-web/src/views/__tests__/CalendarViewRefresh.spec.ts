import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { fetchDailySummary, fetchTransactions } from '../../api/client'
import { i18n } from '../../i18n'
import { dataVersion } from '../../stores/refresh'
import CalendarView from '../CalendarView.vue'

vi.mock('../../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
  fetchDailySummary: vi.fn(),
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

vi.mock('../../components/CalendarGrid.vue', () => ({
  default: {
    name: 'CalendarGrid',
    props: ['dailyStats', 'selectedDate'],
    emits: ['selectDate', 'visibleRangeChange'],
    template: '<div data-testid="calendar-grid" />',
  },
}))

vi.mock('../../components/TransactionList.vue', () => ({
  default: { name: 'TransactionList', props: ['transactions'], template: '<div />' },
}))

vi.mock('../../components/layout/TransactionFormOverlay.vue', () => ({
  default: { name: 'TransactionFormOverlay', template: '<div />' },
}))

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  vi.mocked(fetchTransactions).mockResolvedValue([])
  vi.mocked(fetchDailySummary).mockResolvedValue([])
})

describe('CalendarView 数据刷新', () => {
  it('数据版本变化后重新加载日历统计', async () => {
    const wrapper = mount(CalendarView, { global: { plugins: [i18n] } })
    await nextTick()

    const grid = wrapper.findComponent({ name: 'CalendarGrid' })
    grid.vm.$emit('visibleRangeChange', '2026-08-01', '2026-08-31')
    await vi.waitFor(() => expect(fetchDailySummary).toHaveBeenCalledTimes(1))

    vi.mocked(fetchDailySummary).mockClear()
    // 模拟导入/编辑交易后账目数据版本号递增
    dataVersion.value++

    await vi.waitFor(() => expect(fetchDailySummary).toHaveBeenCalledTimes(1))
    expect(fetchDailySummary).toHaveBeenCalledWith('2026-08-01', '2026-08-31')
  })
})
