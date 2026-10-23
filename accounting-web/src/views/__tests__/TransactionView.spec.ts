import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import { fetchTransactions } from '../../api/client'
import { panelActionKey, type PanelAction } from '../../components/layout/panelAction'
import { i18n } from '../../i18n'
import type { PostingDto, TransactionDto } from '../../types/api'
import TransactionView from '../TransactionView.vue'

vi.mock('../../api/client', () => ({
  fetchTransactions: vi.fn(),
  fetchTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
}))

vi.mock('../../components/TransactionList.vue', () => ({
  default: { name: 'TransactionList', props: ['transactions'], template: '<div />' },
}))

vi.mock('../../components/TransactionFilterDrawer.vue', () => ({
  default: { name: 'TransactionFilterDrawer', template: '<div />' },
}))

vi.mock('../../components/layout/TransactionFormOverlay.vue', () => ({
  default: { name: 'TransactionFormOverlay', template: '<div />' },
}))

function posting(
  partial: Pick<PostingDto, 'account' | 'account_type' | 'commodity' | 'amount'>
): PostingDto {
  return {
    id: 0,
    transaction_id: 0,
    account_id: 0,
    is_reimbursable: false,
    linked_posting_id: null,
    reversal_total: '0',
    ...partial,
  }
}

function makeTx(
  overrides: Partial<TransactionDto> = {},
  postings: PostingDto[] = []
): TransactionDto {
  return {
    id: 1,
    date_time: '2026-01-01T12:00:00',
    description: '测试',
    kind: 'normal',
    member_id: 1,
    member_name: '',
    tags: [],
    pending: false,
    channel_paths: [],
    postings,
    is_transfer: false,
    ...overrides,
  }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  i18n.global.locale.value = 'zh-CN'
})

describe('TransactionView 月收支汇总口径', () => {
  it('与交易列表日分组同口径：月支出含支出、纯转账不计入', async () => {
    // 支出：资产 -100 / 支出 +100 → 资产净额 -100 → 月支出 100
    const expense = makeTx({ id: 1, date_time: '2026-01-05T12:00:00' }, [
      posting({
        account: 'Assets:Bank',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-100.00',
      }),
      posting({
        account: 'Expenses:Food',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '100.00',
      }),
    ])
    // 纯转账：资产 -50 / +50 → 净额 0 → 不计入
    const transfer = makeTx({ id: 2, date_time: '2026-01-06T12:00:00', is_transfer: true }, [
      posting({
        account: 'Assets:Bank',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-50.00',
      }),
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '50.00',
      }),
    ])

    vi.mocked(fetchTransactions).mockResolvedValue([expense, transfer])

    const wrapper = mount(TransactionView, { global: { plugins: [i18n] } })
    await vi.waitFor(() => expect(wrapper.find('.amount').text()).toContain('100.00'))

    expect(wrapper.find('.amount').text()).toBe('¥100.00')
    const sub = wrapper.find('.sub').text()
    expect(sub).toContain('¥0.00')
    expect(sub).toContain('-100.00')
  })
})


function isInert(el: Element): boolean {
  return el.hasAttribute('inert') || (el as unknown as { inert?: boolean }).inert === true
}

describe('TransactionView 详情覆盖层往返', () => {
  it('打开覆盖层时保留底层列表，关闭后滚动位置不变（编辑路径）', async () => {
    vi.mocked(fetchTransactions).mockResolvedValue([])
    const wrapper = mount(TransactionView, { global: { plugins: [i18n] } })
    await nextTick()

    const container = wrapper.find('.transaction')
    ;(container.element as HTMLElement).scrollTop = 320

    wrapper.findComponent({ name: 'TransactionList' }).vm.$emit('edit', 1)
    await nextTick()

    // 覆盖层打开时底层列表仍挂载（未被 v-if 卸载），只是被屏蔽交互
    expect(wrapper.findComponent({ name: 'TransactionFormOverlay' }).exists()).toBe(true)
    expect(wrapper.find('.transaction .hero').exists()).toBe(true)
    expect(wrapper.findComponent({ name: 'TransactionList' }).exists()).toBe(true)
    expect(isInert(container.element)).toBe(true)

    wrapper.findComponent({ name: 'TransactionFormOverlay' }).vm.$emit('close')
    await nextTick()

    expect(isInert(container.element)).toBe(false)
    expect((wrapper.find('.transaction').element as HTMLElement).scrollTop).toBe(320)
  })

  it('通过新建动作打开覆盖层时同样保留底层列表与滚动位置', async () => {
    vi.mocked(fetchTransactions).mockResolvedValue([])
    const actions = ref<PanelAction[]>([])
    const wrapper = mount(TransactionView, {
      global: { plugins: [i18n], provide: { [panelActionKey]: actions } },
    })
    await nextTick()
    await vi.waitFor(() => expect(actions.value.length).toBeGreaterThan(0))

    const container = wrapper.find('.transaction')
    ;(container.element as HTMLElement).scrollTop = 180

    actions.value[0].onClick()
    await nextTick()

    expect(wrapper.findComponent({ name: 'TransactionFormOverlay' }).exists()).toBe(true)
    expect(wrapper.find('.transaction .hero').exists()).toBe(true)
    expect(isInert(container.element)).toBe(true)

    wrapper.findComponent({ name: 'TransactionFormOverlay' }).vm.$emit('close')
    await nextTick()

    expect(isInert(container.element)).toBe(false)
    expect((wrapper.find('.transaction').element as HTMLElement).scrollTop).toBe(180)
  })
})
