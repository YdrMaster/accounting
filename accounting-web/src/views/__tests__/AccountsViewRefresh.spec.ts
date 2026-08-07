import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import { i18n } from '../../i18n'
import { panelActionKey, type PanelAction } from '../../components/layout/panelAction'
import { useAccountStore } from '../../stores/account'
import { notifyAccountsChanged } from '../../stores/refresh'
import type { AccountDto } from '../../types/api'
import AccountsView from '../AccountsView.vue'

vi.mock('../../api/client', () => ({
  fetchAccounts: vi.fn().mockResolvedValue([]),
  moveAccount: vi.fn(),
  fetchBalanceSheet: vi.fn().mockResolvedValue({ assets: [] }),
  fetchNetWorthTrend: vi.fn(),
  fetchCashFlow: vi.fn(),
}))

vi.mock('../../stores/refresh', () => ({
  dataVersion: { value: 0 },
  notifyTransactionsChanged: vi.fn().mockResolvedValue(undefined),
  notifyAccountsChanged: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('../../components/layout/AccountDrawer.vue', () => ({
  default: {
    name: 'AccountDrawer',
    props: ['account'],
    emits: ['close', 'updated', 'deleted'],
    template: '<div data-testid="drawer" />',
  },
}))

vi.mock('../../components/layout/AccountCreateDrawer.vue', () => ({
  default: {
    name: 'AccountCreateDrawer',
    props: ['parentAccount'],
    emits: ['close', 'created'],
    template: '<div data-testid="create-drawer" />',
  },
}))

vi.mock('../../components/layout/AccountGrid.vue', () => ({
  default: {
    name: 'AccountGrid',
    props: ['typeLabel', 'rows', 'selectedAccountId'],
    emits: ['click', 'columnsChange'],
    template: `
      <div data-testid="grid">
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

const rootAccount: AccountDto = {
  id: 1,
  name: 'Assets',
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: true,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
}

const childAccount: AccountDto = {
  id: 2,
  name: 'Cash',
  account_type: 'Asset',
  parent_id: 1,
  closed_at: null,
  is_system: false,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
}

async function mountWithSelectedAccount() {
  const panelAction = ref<PanelAction[]>([])
  const wrapper = mount(AccountsView, {
    global: {
      plugins: [i18n],
      provide: { [panelActionKey]: panelAction },
    },
  })
  await nextTick()
  const grid = wrapper.findComponent({ name: 'AccountGrid' })
  const button = grid
    .findAll('button')
    .find(b => b.attributes('data-account-id') === String(childAccount.id))
  await button?.trigger('click')
  await nextTick()
  return { wrapper, panelAction }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  const store = useAccountStore()
  store.accounts = [rootAccount, childAccount]
})

describe('AccountsView 账户变更后的刷新', () => {
  it('账户更新后触发 notifyAccountsChanged', async () => {
    const { wrapper } = await mountWithSelectedAccount()
    const drawer = wrapper.findComponent({ name: 'AccountDrawer' })
    drawer.vm.$emit('updated', { ...childAccount, name: 'Wallet' })
    await nextTick()

    expect(notifyAccountsChanged).toHaveBeenCalled()
  })

  it('账户删除后触发 notifyAccountsChanged', async () => {
    const { wrapper } = await mountWithSelectedAccount()
    const drawer = wrapper.findComponent({ name: 'AccountDrawer' })
    drawer.vm.$emit('deleted', childAccount.id)
    await nextTick()

    expect(notifyAccountsChanged).toHaveBeenCalled()
  })

  it('账户创建后触发 notifyAccountsChanged', async () => {
    const { wrapper, panelAction } = await mountWithSelectedAccount()
    const createAction = panelAction.value[0]
    expect(createAction.disabled).toBe(false)
    createAction.onClick()
    await nextTick()

    const createDrawer = wrapper.findComponent({ name: 'AccountCreateDrawer' })
    createDrawer.vm.$emit('created')
    await nextTick()

    expect(notifyAccountsChanged).toHaveBeenCalled()
  })
})
