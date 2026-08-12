import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { i18n } from '../../i18n'
import type { PostingDto, TransactionDto } from '../../types/api'
import TransactionCard from '../TransactionCard.vue'

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
    description: '午饭',
    kind: 'normal',
    member_id: 1,
    member_name: '',
    tags: [],
    pending: false,
    channel_paths: [],
    postings,
    ...overrides,
  }
}

function mountCard(tx: TransactionDto) {
  return mount(TransactionCard, { props: { tx }, global: { plugins: [i18n] } })
}

describe('TransactionCard 折叠态', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('普通支出两行展示：标题 + 金额，第二行含摘要', () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-35.00',
      }),
      posting({
        account: 'Expenses:餐饮',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '35.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.row-main').exists()).toBe(true)
    expect(wrapper.find('.row-sub').exists()).toBe(true)
    expect(wrapper.find('.title').text()).toBe('午饭')
    expect(wrapper.find('.amount').text()).toBe('-35.00')
    expect(wrapper.find('.summary').text()).toBe('Cash → 餐饮')
  })

  it('描述为空的折叠态合并为单行，摘要充当主标题', () => {
    const tx = makeTx({ description: '' }, [
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-35.00',
      }),
      posting({
        account: 'Expenses:餐饮',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '35.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.row-main.merged').exists()).toBe(true)
    expect(wrapper.find('.row-sub').exists()).toBe(false)
    expect(wrapper.find('.title').text()).toBe('Cash → 餐饮')
    expect(wrapper.find('.amount').text()).toBe('-35.00')
  })

  it('转账摘要按资金流方向对位，金额为正值之和', () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:工商',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-8000.00',
      }),
      posting({
        account: 'Assets:招行',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '8000.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.summary').text()).toBe('工商 → 招行')
    expect(wrapper.find('.amount').text()).toBe('+8,000.00')
  })

  it('待分类交易带 pending 样式类且金额必现', () => {
    const tx = makeTx({ pending: true }, [
      posting({
        account: 'Assets:Import:alipay:餐饮',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-128.00',
      }),
      posting({
        account: 'Expenses:Import:alipay:餐饮',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '128.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.tx-card.pending').exists()).toBe(true)
    expect(wrapper.find('.amount').exists()).toBe(true)
    expect(wrapper.find('.amount').text()).toBe('-128.00')
  })

  it('含 :Import: 分录的交易折叠态金额必现', () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:Import:pending:晚餐',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-45.00',
      }),
      posting({
        account: 'Expenses:Import:pending:晚餐',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '45.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.amount').text()).toBe('-45.00')
  })

  it('多币种金额带主币种前缀，次要币种代码标出', () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-100.00',
      }),
      posting({
        account: 'Assets:CashUSD',
        account_type: 'asset',
        commodity: 'USD',
        amount: '-120.00',
      }),
      posting({
        account: 'Expenses:海外',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '100.00',
      }),
      posting({
        account: 'Expenses:海外购',
        account_type: 'expense',
        commodity: 'USD',
        amount: '120.00',
      }),
    ])
    const wrapper = mountCard(tx)
    // CNY / USD 各两条平票 → 先出现者为主币种 CNY → ¥ 前缀
    expect(wrapper.find('.amount').text()).toBe('¥-220.00')
    expect(wrapper.find('.currency').text()).toBe('USD')
  })

  it('展开后显示完整分录，点击可折叠', async () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-35.00',
      }),
      posting({
        account: 'Expenses:餐饮',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '35.00',
      }),
    ])
    const wrapper = mountCard(tx)
    expect(wrapper.find('.tx-entries').exists()).toBe(false)
    await wrapper.trigger('click')
    expect(wrapper.find('.tx-entries').exists()).toBe(true)
    expect(wrapper.findAll('.entry-row')).toHaveLength(2)
  })
})
