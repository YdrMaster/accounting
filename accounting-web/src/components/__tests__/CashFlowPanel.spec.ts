import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n } from '../../i18n'
import { useAccountStore } from '../../stores/account'
import { useReportStore } from '../../stores/report'
import { useTransactionStore } from '../../stores/transaction'
import CashFlowPanel from '../CashFlowPanel.vue'

const spinTo = vi.fn()
vi.mock('../../composables/useWheelScroll', () => ({
  useWheelScroll: () => ({ spinTo }),
}))

const cashFlow = {
  period_start: '2026-08-01',
  period_end: '2026-08-31',
  income: [],
  expense: [
    { account_id: 1, parent_id: null, name: 'Expenses', amount: '500' },
    { account_id: 2, parent_id: 1, name: '餐饮', amount: '500' },
    { account_id: 3, parent_id: 2, name: '外卖', amount: '300' },
    { account_id: 4, parent_id: 2, name: '聚餐', amount: '200' },
  ],
}

function mountPanel() {
  return mount(CashFlowPanel, {
    global: {
      plugins: [i18n],
      stubs: {
        PeriodNav: true,
        PeriodSelect: true,
        CategorySunburst: true,
      },
    },
  })
}

describe('CashFlowPanel 点击明细行跳转交易筛选', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    spinTo.mockClear()

    const reportStore = useReportStore()
    reportStore.cashFlow = cashFlow
    reportStore.loadCashFlowTab = vi.fn()

    const accountStore = useAccountStore()
    accountStore.accounts = [
      { id: 1, parent_id: null },
      { id: 2, parent_id: 1 },
      { id: 3, parent_id: 2 },
      { id: 4, parent_id: 2 },
    ] as never
  })

  it('点击父账户：整体替换筛选为周期 + 账户子树，并转动至交易面板', async () => {
    const txStore = useTransactionStore()
    const setFilter = vi.spyOn(txStore, 'setFilter')

    const wrapper = mountPanel()
    await wrapper.findAll('.row')[0].trigger('click') // 餐饮

    expect(setFilter).toHaveBeenCalledWith({
      from: '2026-08-01',
      to: '2026-08-31',
      accounts: [2, 3, 4],
      members: [],
      tags: [],
      channels: [],
    })
    expect(spinTo).toHaveBeenCalledWith(0)
  })

  it('点击叶子账户：筛选仅含自身', async () => {
    const txStore = useTransactionStore()
    const setFilter = vi.spyOn(txStore, 'setFilter')

    const wrapper = mountPanel()
    await wrapper.findAll('.row')[1].trigger('click') // 外卖

    expect(setFilter).toHaveBeenCalledWith(
      expect.objectContaining({ accounts: [3] })
    )
  })
})
