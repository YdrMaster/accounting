import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import AccountsView from '../AccountsView.vue'
import { useAccountStore } from '../../stores/account'

vi.mock('../../components/layout/AccountDrawer.vue', () => ({
  default: {
    name: 'AccountDrawer',
    template: '<div data-testid="drawer">Drawer</div>',
  },
}))

vi.mock('../../components/layout/AccountGrid.vue', () => ({
  default: {
    name: 'AccountGrid',
    props: ['typeLabel', 'rows', 'selectedAccountId'],
    template: '<div data-testid="grid">{{ typeLabel }}</div>',
  },
}))

describe('AccountsView', () => {
  it('renders a grid for each account type', async () => {
    setActivePinia(createPinia())
    const store = useAccountStore()
    store.accounts = [
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
        id: 2,
        name: 'Cash',
        account_type: 'Asset',
        parent_id: 1,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
    ]
    const wrapper = mount(AccountsView)
    await nextTick()
    const grids = wrapper.findAll('[data-testid="grid"]')
    expect(grids.length).toBeGreaterThan(0)
  })
})
