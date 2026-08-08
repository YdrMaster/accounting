import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { i18n } from '../../i18n'
import type { CashFlowItemDto } from '../../types/api'
import CashFlowDetailList from '../CashFlowDetailList.vue'

const items: CashFlowItemDto[] = [
  { account_id: 1, parent_id: null, name: 'Expenses', amount: '500' },
  { account_id: 2, parent_id: 1, name: '餐饮', amount: '500' },
  { account_id: 3, parent_id: 2, name: '外卖', amount: '300' },
  { account_id: 4, parent_id: 2, name: '聚餐', amount: '200' },
]

function mountList() {
  return mount(CashFlowDetailList, {
    props: { items, drillId: null, side: 'expense' as const },
    global: { plugins: [i18n] },
  })
}

describe('CashFlowDetailList 点击行', () => {
  it('点击明细行 emit select 及正确 accountId', async () => {
    const wrapper = mountList()
    const rows = wrapper.findAll('.row')
    // 未下钻时从一级分类开始：餐饮、外卖、聚餐
    await rows[0].trigger('click')
    await rows[1].trigger('click')
    const events = wrapper.emitted('select')
    expect(events).toEqual([[2], [3]])
  })

  it('行带有可点击样式', () => {
    const wrapper = mountList()
    expect(wrapper.find('.row.clickable').exists()).toBe(true)
  })
})
