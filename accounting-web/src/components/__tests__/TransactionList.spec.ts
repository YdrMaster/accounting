import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { i18n } from '../../i18n'
import type { PostingDto, TransactionDto } from '../../types/api'
import TransactionList from '../TransactionList.vue'

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
    is_transfer: false,
    ...overrides,
  }
}

function mountList(transactions: TransactionDto[]) {
  return mount(TransactionList, { props: { transactions }, global: { plugins: [i18n] } })
}

describe('TransactionList 按日汇总口径', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    i18n.global.locale.value = 'zh-CN'
  })

  it('零净额非转账交易不计入当日收/支', () => {
    const tx = makeTx({}, [
      posting({
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-100.00',
      }),
      posting({
        account: 'Assets:Bank',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '100.00',
      }),
      posting({
        account: 'Expenses:杂项',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '0.00',
      }),
    ])
    const wrapper = mountList([tx])
    expect(wrapper.find('.day-summary').text()).toBe('收：¥0.00 支：¥0.00')
  })

  it('纯转账不计入当日收/支', () => {
    const tx = makeTx({ is_transfer: true }, [
      posting({
        account: 'Assets:工商',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-1000.00',
      }),
      posting({
        account: 'Assets:招行',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '1000.00',
      }),
    ])
    const wrapper = mountList([tx])
    expect(wrapper.find('.day-summary').text()).toBe('收：¥0.00 支：¥0.00')
  })

  it('含手续费转账按资产净额只计支出（与后端 daily-summary 一致）', () => {
    const tx = makeTx({ is_transfer: true }, [
      posting({
        account: 'Assets:工商',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-1010.00',
      }),
      posting({
        account: 'Assets:招行',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '1000.00',
      }),
      posting({
        account: 'Expenses:Fees',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '10.00',
      }),
    ])
    const wrapper = mountList([tx])
    // 资产净额 = -1010 + 1000 = -10 → 支出 10，收入 0
    expect(wrapper.find('.day-summary').text()).toBe('收：¥0.00 支：¥10.00')
  })

  it('普通支出仍计入当日支出', () => {
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
    const wrapper = mountList([tx])
    expect(wrapper.find('.day-summary').text()).toBe('收：¥0.00 支：¥35.00')
  })
})
