import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import AccountGrid from '../AccountGrid.vue'

vi.mock('../../composables/useGridColumns', () => ({
  useGridColumns: () => ({ columns: ref(2) }),
}))

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

  it('emits columnsChange when columns change', async () => {
    const wrapper = mount(AccountGrid, {
      props: {
        typeLabel: '资产',
        rows: [],
        selectedAccountId: null,
      },
    })
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('columnsChange')?.[0]).toEqual([2])
  })
})
