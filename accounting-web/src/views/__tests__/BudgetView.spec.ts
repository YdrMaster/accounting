import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import type { Ref } from 'vue'
import type { PanelAction } from '../../components/layout/panelAction'
import { panelActionKey } from '../../components/layout/panelAction'
import { i18n, setLocale } from '../../i18n'
import { useAccountStore } from '../../stores/account'
import { useBudgetStore } from '../../stores/budget'
import type { BudgetStatusDto } from '../../types/api'
import { dialogState, resolveDialog } from '../../utils/dialog'
import BudgetView from '../BudgetView.vue'

vi.mock('../../components/layout/AccountPicker.vue', () => ({
  default: {
    name: 'AccountPicker',
    props: ['modelValue', 'placeholder', 'accountType'],
    emits: ['update:modelValue'],
    template: `
      <button
        class="picker-stub"
        :data-account-type="accountType"
        @click="$emit('update:modelValue', 21)"
      >
        {{ modelValue ?? 'none' }}
      </button>
    `,
  },
}))

function makeBudget(
  overrides: Partial<BudgetStatusDto['budget']> = {}
): BudgetStatusDto['budget'] {
  return {
    id: 1,
    name: '生活开销',
    period: 'monthly',
    deadline: null,
    commodity_id: 1,
    ...overrides,
  }
}

function makeStatus(overrides: Partial<BudgetStatusDto> = {}): BudgetStatusDto {
  return {
    budget: makeBudget(),
    expired: false,
    period_start: '2026-08-01',
    period_end: '2026-08-31',
    items: [
      {
        account_id: 21,
        limit_amount: '1000',
        actual_amount: '400',
        remaining: '600',
        percentage: '40',
      },
    ],
    ...overrides,
  }
}

const ACCOUNTS = [
  {
    id: 2,
    name: 'Expenses',
    account_type: 'Expense',
    parent_id: null,
    closed_at: null,
    is_system: true,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
  {
    id: 21,
    name: '餐饮',
    account_type: 'Expense',
    parent_id: 2,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
  {
    id: 22,
    name: '交通',
    account_type: 'Expense',
    parent_id: 2,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
]

interface Harness {
  store: ReturnType<typeof useBudgetStore>
  panelAction: Ref<PanelAction[]>
  mountView: () => ReturnType<typeof mount>
}

function setup(statuses: BudgetStatusDto[]): Harness {
  setActivePinia(createPinia())
  setLocale('zh-CN')

  const store = useBudgetStore()
  store.statuses = statuses
  store.loadStatuses = vi.fn().mockResolvedValue(undefined)
  store.create = vi.fn().mockResolvedValue(makeBudget({ id: 99 }))
  store.update = vi.fn().mockResolvedValue(undefined)
  store.remove = vi.fn().mockResolvedValue(undefined)

  const accountStore = useAccountStore()
  accountStore.accounts = ACCOUNTS
  accountStore.loadAccounts = vi.fn().mockResolvedValue(undefined)

  const panelAction = ref<PanelAction[]>([])

  const mountView = () =>
    mount(BudgetView, {
      global: {
        plugins: [i18n],
        provide: { [panelActionKey as symbol]: panelAction },
      },
    })

  return { store, panelAction, mountView }
}

describe('BudgetView', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  it('renders budget cards with ring color class, center amounts and badges', async () => {
    const { mountView } = setup([
      makeStatus(), // 400/1000 → 未超支绿环，剩余 600.00
      makeStatus({
        budget: makeBudget({ id: 2, name: '交通预算' }),
        items: [
          {
            account_id: 22,
            limit_amount: '600',
            actual_amount: '1008.10',
            remaining: '-408.10',
            percentage: '168',
          },
        ],
      }), // 超支 → 红环 + 超支 408.10
      makeStatus({
        budget: makeBudget({ id: 3, name: '过期预算', deadline: '2020-01-01' }),
        expired: true,
      }), // 已失效 → 灰环 + 徽标
    ])
    const wrapper = mountView()
    await nextTick()

    const cards = wrapper.findAll('.budget-card')
    expect(cards).toHaveLength(3)
    expect(cards[0].text()).toContain('生活开销')
    expect(cards[0].text()).toContain('每月')

    const rings = wrapper.findAll('.budget-ring')
    expect(rings[0].classes()).toContain('ring-green')
    expect(rings[0].text()).toContain('剩余')
    expect(rings[0].text()).toContain('600.00')
    expect(rings[1].classes()).toContain('ring-red')
    expect(rings[1].text()).toContain('超支')
    expect(rings[1].text()).toContain('408.10')
    expect(rings[2].classes()).toContain('ring-gray')

    expect(cards[2].find('.badge-expired').exists()).toBe(true)
    expect(cards[2].text()).toContain('已失效')
    expect(cards[0].find('.badge-expired').exists()).toBe(false)
  })

  it('shows one-off label with deadline for period-null budgets', async () => {
    const { mountView } = setup([
      makeStatus({
        budget: makeBudget({ period: null, deadline: '2026-12-31' }),
        period_start: null,
        period_end: null,
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    const card = wrapper.find('.budget-card')
    expect(card.text()).toContain('一次性')
    expect(card.text()).toContain('2026-12-31')
  })

  it('expands status detail inline on card click and collapses on second click', async () => {
    const { mountView } = setup([
      makeStatus({
        items: [
          {
            account_id: 21,
            limit_amount: '1000',
            actual_amount: '1200',
            remaining: '-200',
            percentage: '120',
          },
          {
            account_id: 22,
            limit_amount: '500',
            actual_amount: '100',
            remaining: '400',
            percentage: '20',
          },
        ],
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    expect(wrapper.find('.budget-detail').exists()).toBe(false)

    await wrapper.find('.budget-card').trigger('click')
    const detail = wrapper.find('.budget-detail')
    expect(detail.exists()).toBe(true)

    // 周期预算显示当前周期起止
    const range = detail.find('.period-range')
    expect(range.exists()).toBe(true)
    expect(range.text()).toContain('2026-08-01')
    expect(range.text()).toContain('2026-08-31')

    // 各账户明细行：限额/实际/剩余/百分比
    const rows = detail.findAll('.status-row')
    expect(rows).toHaveLength(2)
    expect(rows[0].text()).toContain('Expenses:餐饮')
    expect(rows[0].find('.item-limit').text()).toContain('1000')
    expect(rows[0].find('.item-actual').text()).toContain('1200')
    expect(rows[0].find('.item-remaining').text()).toContain('-200')
    expect(rows[0].find('.item-percentage').text()).toContain('120')
    // 超支账户红色标记
    expect(rows[0].classes()).toContain('overspent')
    expect(rows[1].classes()).not.toContain('overspent')
    expect(rows[1].text()).toContain('Expenses:交通')
    expect(rows[1].find('.item-remaining').text()).toContain('400')

    await wrapper.find('.budget-card').trigger('click')
    expect(wrapper.find('.budget-detail').exists()).toBe(false)
  })

  it('keeps ring green when a single account overspends but the aggregate does not', async () => {
    // 口径钉住：合计 1300/1500 未超支 → 绿环；仅超支账户的明细行标红
    const { mountView } = setup([
      makeStatus({
        items: [
          {
            account_id: 21,
            limit_amount: '1000',
            actual_amount: '1200',
            remaining: '-200',
            percentage: '120',
          },
          {
            account_id: 22,
            limit_amount: '500',
            actual_amount: '100',
            remaining: '400',
            percentage: '20',
          },
        ],
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    const ring = wrapper.find('.budget-ring')
    expect(ring.classes()).toContain('ring-green')
    expect(ring.classes()).not.toContain('ring-red')

    await wrapper.find('.budget-card').trigger('click')
    const rows = wrapper.findAll('.status-row')
    expect(rows[0].classes()).toContain('overspent')
    expect(rows[1].classes()).not.toContain('overspent')
  })

  it('shows no period range for one-off budgets in expanded detail', async () => {
    const { mountView } = setup([
      makeStatus({
        budget: makeBudget({ period: null }),
        period_start: null,
        period_end: null,
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.budget-card').trigger('click')
    const detail = wrapper.find('.budget-detail')
    expect(detail.exists()).toBe(true)
    expect(detail.find('.period-range').exists()).toBe(false)
    expect(detail.findAll('.status-row')).toHaveLength(1)
  })

  it('shows empty state when there are no budgets', async () => {
    const { mountView } = setup([])
    const wrapper = mountView()
    await nextTick()
    expect(wrapper.find('.empty').exists()).toBe(true)
    expect(wrapper.find('.empty').text()).toBe('暂无预算表')
  })

  it('submits one-off create form with deadline through the store and refreshes statuses', async () => {
    const { store, panelAction, mountView } = setup([])
    const wrapper = mountView()
    await nextTick()

    expect(panelAction.value).toHaveLength(1)
    panelAction.value[0].onClick()
    await nextTick()
    expect(wrapper.find('.drawer').exists()).toBe(true)

    await wrapper.find('input[type="text"]').setValue('旅行预算')
    // 周期选「一次性」（空值选项）
    await wrapper.find('select').setValue('')
    await wrapper.find('input[type="date"]').setValue('2026-12-31')

    // 添加一行限额并选择账户（stub 固定 emit 21）
    await wrapper.find('.add-limit-btn').trigger('click')
    const picker = wrapper.find('.picker-stub')
    expect(picker.attributes('data-account-type')).toBe('expense')
    await picker.trigger('click')
    await wrapper.find('.limit-row input[type="number"]').setValue('500')

    await wrapper.find('.submit-btn').trigger('click')
    await nextTick()

    expect(store.create).toHaveBeenCalledWith({
      name: '旅行预算',
      period: null,
      deadline: '2026-12-31',
      commodity_id: 1,
      limits: [{ account_id: 21, amount: '500' }],
    })
    // 挂载时一次 + 创建成功后刷新一次
    expect(store.loadStatuses).toHaveBeenCalledTimes(2)
    expect(wrapper.find('.drawer').exists()).toBe(false)
  })

  it('submits periodic create form with null deadline (regression)', async () => {
    const { store, panelAction, mountView } = setup([])
    const wrapper = mountView()
    await nextTick()
    panelAction.value[0].onClick()
    await nextTick()

    await wrapper.find('input[type="text"]').setValue('月度预算')
    // 默认周期 monthly，deadline 留空
    await wrapper.find('.add-limit-btn').trigger('click')
    await wrapper.find('.picker-stub').trigger('click')
    await wrapper.find('.limit-row input[type="number"]').setValue('1000')

    await wrapper.find('.submit-btn').trigger('click')
    await nextTick()

    expect(store.create).toHaveBeenCalledWith({
      name: '月度预算',
      period: 'monthly',
      deadline: null,
      commodity_id: 1,
      limits: [{ account_id: 21, amount: '1000' }],
    })
  })

  it('opens edit drawer prefilled and submits update through the store', async () => {
    const { store, mountView } = setup([
      makeStatus({
        budget: makeBudget({ id: 7, name: '房租', period: null, deadline: '2026-12-31' }),
        period_start: null,
        period_end: null,
        items: [
          {
            account_id: 21,
            limit_amount: '1000',
            actual_amount: '400',
            remaining: '600',
            percentage: '40',
          },
          {
            account_id: 22,
            limit_amount: '500',
            actual_amount: '100',
            remaining: '400',
            percentage: '20',
          },
        ],
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.edit-btn').trigger('click')
    await nextTick()
    expect(wrapper.find('.drawer').exists()).toBe(true)

    // 预填：名称/一次性（空值选项）/deadline/限额列表
    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe('房租')
    expect((wrapper.find('select').element as HTMLSelectElement).value).toBe('')
    expect((wrapper.find('input[type="date"]').element as HTMLInputElement).value).toBe('2026-12-31')
    expect(wrapper.findAll('.picker-stub')).toHaveLength(2)
    const amounts = wrapper.findAll('.limit-row input[type="number"]')
    expect((amounts[0].element as HTMLInputElement).value).toBe('1000')
    expect((amounts[1].element as HTMLInputElement).value).toBe('500')

    await wrapper.find('input[type="text"]').setValue('房租水电')
    await wrapper.find('.submit-btn').trigger('click')
    await nextTick()

    expect(store.update).toHaveBeenCalledWith(7, {
      name: '房租水电',
      period: null,
      deadline: '2026-12-31',
      commodity_id: 1,
      limits: [
        { account_id: 21, amount: '1000' },
        { account_id: 22, amount: '500' },
      ],
    })
    expect(store.loadStatuses).toHaveBeenCalledTimes(2)
    expect(wrapper.find('.drawer').exists()).toBe(false)
  })

  it('asks for confirmation before deleting and removes the budget', async () => {
    const { store, mountView } = setup([makeStatus()])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.delete-btn').trigger('click')
    expect(dialogState.visible).toBe(true)
    resolveDialog(true)
    await vi.waitFor(() => expect(store.remove).toHaveBeenCalledWith(1))
  })

  it('does not delete when confirmation is cancelled', async () => {
    const { store, mountView } = setup([makeStatus()])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.delete-btn').trigger('click')
    expect(dialogState.visible).toBe(true)
    resolveDialog(false)
    await nextTick()
    expect(store.remove).not.toHaveBeenCalled()
  })
})
