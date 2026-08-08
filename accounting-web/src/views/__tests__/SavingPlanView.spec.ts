import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import type { Ref } from 'vue'
import type { PanelAction } from '../../components/layout/panelAction'
import { panelActionKey } from '../../components/layout/panelAction'
import { i18n, setLocale } from '../../i18n'
import { useAccountStore } from '../../stores/account'
import { useSavingPlanStore } from '../../stores/savingPlan'
import type { SavingPlanStatusDto } from '../../types/api'
import { dialogState, resolveDialog } from '../../utils/dialog'
import SavingPlanView from '../SavingPlanView.vue'

vi.mock('../../components/layout/AccountPicker.vue', () => ({
  default: {
    name: 'AccountPicker',
    props: ['modelValue', 'placeholder', 'accountType'],
    emits: ['update:modelValue'],
    template: `
      <button
        class="picker-stub"
        :data-account-type="accountType"
        @click="$emit('update:modelValue', 11)"
      >
        {{ modelValue ?? 'none' }}
      </button>
    `,
  },
}))

function makePlan(overrides: Partial<SavingPlanStatusDto['plan']> = {}): SavingPlanStatusDto['plan'] {
  return {
    id: 1,
    name: '买相机',
    period: null,
    deadline: null,
    commodity_id: 1,
    target_amount: '3000',
    account_ids: [11, 12],
    ...overrides,
  }
}

function makeStatus(overrides: Partial<SavingPlanStatusDto> = {}): SavingPlanStatusDto {
  return {
    plan: makePlan(),
    expired: false,
    period_start: null,
    period_end: null,
    target_amount: '3000',
    current_balance: '4000',
    gap: '-1000',
    met: true,
    allocated: '3000',
    satisfaction: '100',
    accounts: [
      { account_id: 11, balance: '3000', occupied_by_earlier: '0', allocated: '2000' },
      { account_id: 12, balance: '1000', occupied_by_earlier: '0', allocated: '1000' },
    ],
    ...overrides,
  }
}

const ACCOUNTS = [
  {
    id: 1,
    name: 'Assets',
    account_type: 'Asset',
    parent_id: null,
    closed_at: null,
    is_system: true,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
  {
    id: 11,
    name: 'A',
    account_type: 'Asset',
    parent_id: 1,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
  {
    id: 12,
    name: 'B',
    account_type: 'Asset',
    parent_id: 1,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  },
]

interface Harness {
  store: ReturnType<typeof useSavingPlanStore>
  panelAction: Ref<PanelAction[]>
  mountView: () => ReturnType<typeof mount>
}

function setup(statuses: SavingPlanStatusDto[]): Harness {
  setActivePinia(createPinia())
  setLocale('zh-CN')

  const store = useSavingPlanStore()
  store.statuses = statuses
  store.loadStatuses = vi.fn().mockResolvedValue(undefined)
  store.create = vi.fn().mockResolvedValue(makePlan({ id: 99 }))
  store.update = vi.fn().mockResolvedValue(undefined)
  store.remove = vi.fn().mockResolvedValue(undefined)

  const accountStore = useAccountStore()
  accountStore.accounts = ACCOUNTS
  accountStore.loadAccounts = vi.fn().mockResolvedValue(undefined)

  const panelAction = ref<PanelAction[]>([])

  const mountView = () =>
    mount(SavingPlanView, {
      global: {
        plugins: [i18n],
        provide: { [panelActionKey as symbol]: panelAction },
      },
    })

  return { store, panelAction, mountView }
}

describe('SavingPlanView', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  it('renders plan cards with ring color class and badges', async () => {
    const { mountView } = setup([
      makeStatus(), // satisfaction 100, met → green
      makeStatus({
        plan: makePlan({ id: 2, name: '旅行基金', target_amount: '2000' }),
        target_amount: '2000',
        current_balance: '1500',
        gap: '500',
        met: false,
        allocated: '1500',
        satisfaction: '75',
        accounts: [],
      }), // 75 → yellow + gap badge
      makeStatus({
        plan: makePlan({ id: 3, name: '过期计划', deadline: '2020-01-01' }),
        expired: true,
        met: false,
        satisfaction: '50',
        accounts: [],
      }), // expired → gray + expired badge
    ])
    const wrapper = mountView()
    await nextTick()

    const cards = wrapper.findAll('.plan-card')
    expect(cards).toHaveLength(3)
    expect(cards[0].text()).toContain('买相机')
    expect(cards[0].text()).toContain('3000')

    const rings = wrapper.findAll('.plan-ring')
    expect(rings[0].classes()).toContain('ring-green')
    expect(rings[0].text()).toContain('100%')
    expect(rings[1].classes()).toContain('ring-yellow')
    expect(rings[1].text()).toContain('75%')
    expect(rings[2].classes()).toContain('ring-gray')

    expect(cards[0].find('.badge-met').exists()).toBe(true)
    expect(cards[1].find('.badge-gap').exists()).toBe(true)
    expect(cards[1].find('.badge-gap').text()).toContain('500')
    expect(cards[2].find('.badge-expired').exists()).toBe(true)
    expect(cards[2].text()).toContain('已失效')
  })

  it('formats recurring-decimal satisfaction to two places in ring center and detail', async () => {
    const { mountView } = setup([
      makeStatus({
        allocated: '1000',
        satisfaction: '33.33333333333333333333333333',
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    // 环心显示两位小数而非全长小数
    expect(wrapper.find('.plan-ring').text()).toContain('33.33%')
    expect(wrapper.find('.plan-ring').text()).not.toContain('33.3333')

    await wrapper.find('.budget-card').trigger('click')
    const detail = wrapper.find('.plan-detail')
    expect(detail.text()).toContain('33.33%')
    expect(detail.text()).not.toContain('33.3333')
  })

  it('expands status detail inline on card click and collapses on second click', async () => {
    const { mountView } = setup([makeStatus()])
    const wrapper = mountView()
    await nextTick()

    expect(wrapper.find('.plan-detail').exists()).toBe(false)

    await wrapper.find('.plan-card').trigger('click')
    const detail = wrapper.find('.plan-detail')
    expect(detail.exists()).toBe(true)

    // 账面口径
    expect(detail.text()).toContain('4000') // current_balance
    expect(detail.text()).toContain('-1000') // gap
    // 分配口径
    expect(detail.text()).toContain('3000') // allocated
    expect(detail.text()).toContain('100') // satisfaction

    // 每账户分配明细
    const rows = detail.findAll('.alloc-row')
    expect(rows).toHaveLength(2)
    expect(rows[0].text()).toContain('Assets:A')
    expect(rows[0].find('.alloc-balance').text()).toContain('3000')
    expect(rows[0].find('.alloc-occupied').text()).toContain('0')
    expect(rows[0].find('.alloc-allocated').text()).toContain('2000')
    expect(rows[1].text()).toContain('Assets:B')
    expect(rows[1].find('.alloc-balance').text()).toContain('1000')
    expect(rows[1].find('.alloc-allocated').text()).toContain('1000')

    await wrapper.find('.plan-card').trigger('click')
    expect(wrapper.find('.plan-detail').exists()).toBe(false)
  })

  it('shows empty state when there are no plans', async () => {
    const { mountView } = setup([])
    const wrapper = mountView()
    await nextTick()
    expect(wrapper.find('.empty').exists()).toBe(true)
    expect(wrapper.find('.empty').text()).toBe('暂无攒钱计划')
  })

  it('submits create form through the store and refreshes statuses', async () => {
    const { store, panelAction, mountView } = setup([])
    const wrapper = mountView()
    await nextTick()

    // 标题栏新建按钮经 panelAction 注入
    expect(panelAction.value).toHaveLength(1)
    panelAction.value[0].onClick()
    await nextTick()
    expect(wrapper.find('.drawer').exists()).toBe(true)

    await wrapper.find('input[type="text"]').setValue('买相机')
    await wrapper.find('input[type="number"]').setValue('3000')

    // 添加一行账户并选择（stub 固定 emit 11）
    await wrapper.find('.add-account-btn').trigger('click')
    const picker = wrapper.find('.picker-stub')
    expect(picker.attributes('data-account-type')).toBe('asset')
    await picker.trigger('click')

    await wrapper.find('.submit-btn').trigger('click')
    await nextTick()

    expect(store.create).toHaveBeenCalledWith({
      name: '买相机',
      period: 'monthly',
      deadline: null,
      commodity_id: 1,
      target_amount: '3000',
      account_ids: [11],
    })
    // 挂载时一次 + 创建成功后刷新一次
    expect(store.loadStatuses).toHaveBeenCalledTimes(2)
    expect(wrapper.find('.drawer').exists()).toBe(false)
  })

  it('rejects submit when required fields are invalid', async () => {
    const { store, panelAction, mountView } = setup([])
    const wrapper = mountView()
    await nextTick()
    panelAction.value[0].onClick()
    await nextTick()

    // 名称为空
    await wrapper.find('.submit-btn').trigger('click')
    expect(store.create).not.toHaveBeenCalled()
    expect(dialogState.visible).toBe(true)
    resolveDialog(true)

    // 名称有了但目标金额非正
    await wrapper.find('input[type="text"]').setValue('计划')
    await wrapper.find('input[type="number"]').setValue('0')
    await wrapper.find('.submit-btn').trigger('click')
    expect(store.create).not.toHaveBeenCalled()
    resolveDialog(true)

    // 金额有效但账户集合为空
    await wrapper.find('input[type="number"]').setValue('3000')
    await wrapper.find('.submit-btn').trigger('click')
    expect(store.create).not.toHaveBeenCalled()
    resolveDialog(true)
  })

  it('asks for confirmation before deleting and removes the plan', async () => {
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

  it('opens edit drawer prefilled and submits update through the store', async () => {
    const { store, mountView } = setup([
      makeStatus({
        plan: makePlan({ id: 7, name: '房租', period: 'monthly', deadline: '2026-12-31' }),
      }),
    ])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.edit-btn').trigger('click')
    await nextTick()
    expect(wrapper.find('.drawer').exists()).toBe(true)

    // 预填：名称/周期/deadline/目标金额
    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe('房租')
    expect((wrapper.find('select').element as HTMLSelectElement).value).toBe('monthly')
    expect((wrapper.find('input[type="date"]').element as HTMLInputElement).value).toBe('2026-12-31')
    expect((wrapper.find('input[type="number"]').element as HTMLInputElement).value).toBe('3000')
    // 预填账户集合两行
    expect(wrapper.findAll('.picker-stub')).toHaveLength(2)

    await wrapper.find('input[type="text"]').setValue('房租备用金')
    await wrapper.find('.submit-btn').trigger('click')
    await nextTick()

    expect(store.update).toHaveBeenCalledWith(7, {
      name: '房租备用金',
      period: 'monthly',
      deadline: '2026-12-31',
      commodity_id: 1,
      target_amount: '3000',
      account_ids: [11, 12],
    })
    expect(store.loadStatuses).toHaveBeenCalledTimes(2)
    expect(wrapper.find('.drawer').exists()).toBe(false)
  })

  it('prefills one-off plan as period empty option in edit drawer', async () => {
    const { mountView } = setup([makeStatus()])
    const wrapper = mountView()
    await nextTick()

    await wrapper.find('.edit-btn').trigger('click')
    await nextTick()

    // period 为 null 时预填为「一次性」（空值选项）
    expect((wrapper.find('select').element as HTMLSelectElement).value).toBe('')
  })
})
