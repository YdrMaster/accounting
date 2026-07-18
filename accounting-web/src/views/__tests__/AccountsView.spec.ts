import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { i18n } from '../../i18n'
import { useAccountStore } from '../../stores/account'
import type { GridRow } from '../../utils/accountGrid'
import AccountsView from '../AccountsView.vue'

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
    emits: ['click', 'columnsChange'],
    template: `
      <div data-testid="grid">
        <span class="type-label">{{ typeLabel }}</span>
        <template v-for="(row, rowIndex) in rows" :key="rowIndex">
          <button
            v-for="(item, itemIndex) in row.items"
            :key="itemIndex"
            :data-account-id="item.account?.id"
            @click="item.account && $emit('click', item.account)"
          >
            {{ item.account?.name ?? '.' }}
          </button>
        </template>
      </div>
    `,
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
    const wrapper = mount(AccountsView, { global: { plugins: [i18n] } })
    await nextTick()
    const grids = wrapper.findAll('[data-testid="grid"]')
    expect(grids.length).toBeGreaterThan(0)
  })

  it('does not collapse subtrees when clicking an account on the expanded path', async () => {
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
        name: 'Bank',
        account_type: 'Asset',
        parent_id: 1,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 3,
        name: 'Savings',
        account_type: 'Asset',
        parent_id: 2,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 4,
        name: 'Checking',
        account_type: 'Asset',
        parent_id: 2,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 5,
        name: 'Emergency Fund',
        account_type: 'Asset',
        parent_id: 3,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
    ]

    const wrapper = mount(AccountsView, { global: { plugins: [i18n] } })
    await nextTick()

    const grid = wrapper.findComponent({ name: 'AccountGrid' })
    const findButton = (id: number) =>
      grid.findAll('button').find(b => b.attributes('data-account-id') === String(id))

    await findButton(2)?.trigger('click')
    await nextTick()
    expect(grid.props('rows').some((r: GridRow) => r.expandedAccountId === 2)).toBe(true)

    await findButton(3)?.trigger('click')
    await nextTick()
    expect(grid.props('rows').some((r: GridRow) => r.depth === 2)).toBe(true)

    // Click Bank (id=2) again — it's on the path [2, 3], so no collapse
    await findButton(2)?.trigger('click')
    await nextTick()
    expect(grid.props('rows').some((r: GridRow) => r.depth === 2)).toBe(true)
    expect(grid.props('rows').some((r: GridRow) => r.expandedAccountId === 2)).toBe(true)
  })

  it('collapses subtrees when clicking an account not on the expanded path', async () => {
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
        name: 'Bank',
        account_type: 'Asset',
        parent_id: 1,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 3,
        name: 'Savings',
        account_type: 'Asset',
        parent_id: 2,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 4,
        name: 'Checking',
        account_type: 'Asset',
        parent_id: 2,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
      {
        id: 5,
        name: 'Emergency Fund',
        account_type: 'Asset',
        parent_id: 3,
        closed_at: null,
        is_system: false,
        billing_day: null,
        repayment_day: null,
        owner_ids: [],
      },
    ]

    const wrapper = mount(AccountsView, { global: { plugins: [i18n] } })
    await nextTick()

    const grid = wrapper.findComponent({ name: 'AccountGrid' })
    const findButton = (id: number) =>
      grid.findAll('button').find(b => b.attributes('data-account-id') === String(id))

    await findButton(2)?.trigger('click')
    await nextTick()
    await findButton(3)?.trigger('click')
    await nextTick()
    expect(grid.props('rows').some((r: GridRow) => r.depth === 2)).toBe(true)

    // Click Checking (id=4) — not on path [2, 3], so Savings collapses
    await findButton(4)?.trigger('click')
    await nextTick()
    expect(grid.props('rows').some((r: GridRow) => r.depth === 2)).toBe(false)
    expect(grid.props('rows').some((r: GridRow) => r.expandedAccountId === 2)).toBe(true)
  })
})
