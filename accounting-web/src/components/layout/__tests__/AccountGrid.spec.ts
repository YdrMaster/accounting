import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import AccountGrid from '../AccountGrid.vue'

const account = (id: number, name: string) => ({
  id,
  name,
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: false,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
})

describe('AccountGrid', () => {
  it('renders type label and rows', () => {
    const wrapper = mount(AccountGrid, {
      props: {
        typeLabel: '资产',
        rows: [
          {
            items: [
              { account: account(1, 'A'), isPlaceholder: false, hasChildren: false },
              { account: null, isPlaceholder: true, hasChildren: false },
            ],
            depth: 0,
            expandedIndex: null,
            expandedAccountId: null,
            parentRowIndex: null,
          },
        ],
        selectedAccountId: null,
      },
    })
    expect(wrapper.text()).toContain('资产')
    expect(wrapper.text()).toContain('A')
  })

  it('emits columnsChange with the real column count once ready', async () => {
    const originalGetComputedStyle = window.getComputedStyle
    vi.spyOn(window, 'getComputedStyle').mockImplementation((el: Element) => {
      const style = originalGetComputedStyle(el)
      return {
        ...style,
        getPropertyValue: (prop: string) =>
          prop === '--grid-columns' ? '3' : style.getPropertyValue(prop),
      } as CSSStyleDeclaration
    })

    const wrapper = mount(AccountGrid, {
      props: {
        typeLabel: '资产',
        rows: [],
        selectedAccountId: null,
      },
    })
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('columnsChange')?.[0]).toEqual([3])

    vi.restoreAllMocks()
  })
})
