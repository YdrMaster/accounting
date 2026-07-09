import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import type { RowNode } from '../../../utils/accountGrid'
import AccountRowGroup from '../AccountRowGroup.vue'

function account(id: number, name: string) {
  return {
    id,
    name,
    account_type: 'Asset',
    parent_id: null,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

describe('AccountRowGroup', () => {
  it('renders a row of cards', () => {
    const node: RowNode = {
      row: {
        items: [
          { account: account(1, 'A'), isPlaceholder: false, hasChildren: false },
          { account: account(2, 'B'), isPlaceholder: false, hasChildren: false },
          { account: null, isPlaceholder: true, hasChildren: false },
        ],
        depth: 0,
        expandedIndex: null,
        expandedAccountId: null,
        parentRowIndex: null,
      },
      children: [],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    expect(wrapper.text()).toContain('A')
    expect(wrapper.text()).toContain('B')
  })

  it('wraps child rows in a children container', () => {
    const node: RowNode = {
      row: {
        items: [{ account: account(1, 'A'), isPlaceholder: false, hasChildren: false }],
        depth: 0,
        expandedIndex: 0,
        expandedAccountId: 1,
        parentRowIndex: null,
      },
      children: [
        {
          row: {
            items: [{ account: account(2, 'B'), isPlaceholder: false, hasChildren: false }],
            depth: 1,
            expandedIndex: null,
            expandedAccountId: null,
            parentRowIndex: 0,
          },
          children: [],
        },
      ],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    expect(wrapper.find('.children-container').exists()).toBe(true)
    expect(wrapper.text()).toContain('B')
  })

  it('bubbles click events from nested cards', async () => {
    const acc = account(1, 'A')
    const node: RowNode = {
      row: {
        items: [{ account: acc, isPlaceholder: false, hasChildren: false }],
        depth: 0,
        expandedIndex: null,
        expandedAccountId: null,
        parentRowIndex: null,
      },
      children: [],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    await wrapper.find('.account-card').trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.emitted('click')![0]).toEqual([acc])
  })
})
