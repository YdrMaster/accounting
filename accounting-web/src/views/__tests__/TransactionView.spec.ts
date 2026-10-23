import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchTransactions } from '../../api/client'
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
