import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../../i18n'
import { useAccountStore } from '../../../stores/account'
import type { AccountDto } from '../../../types/api'
import AccountGrid from '../AccountGrid.vue'
import AccountPickerOverlay from '../AccountPickerOverlay.vue'
import type { GridRow } from '../../../utils/accountGrid'

vi.mock('../../../api/client', () => ({
  fetchAccounts: vi.fn().mockResolvedValue([]),
}))

function makeAccount(
  id: number,
  name: string,
  parentId: number | null,
  accountType = 'Asset'
): AccountDto {
  return {
    id,
    name,
    account_type: accountType,
    parent_id: parentId,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

// 资产(1) -> 支付宝(2) -> 余额宝(3)
const accounts = [
  makeAccount(1, '资产', null),
  makeAccount(2, '支付宝', 1),
  makeAccount(3, '余额宝', 2),
]

// 多类型：资产(1)->支付宝(2)，支出(4)->餐饮(5)
const multiTypeAccounts = [
  makeAccount(1, '资产', null),
  makeAccount(2, '支付宝', 1),
  makeAccount(4, '支出', null, 'Expense'),
  makeAccount(5, '餐饮', 4, 'Expense'),
]

function mountOverlay(currentId: number | null, accountType?: 'asset' | 'expense') {
  return mount(AccountPickerOverlay, {
    props: { currentId, accountType },
    global: { plugins: [i18n] },
  })
}

function rowAccountIds(rows: GridRow[]): number[] {
  return rows.flatMap(r => r.items.filter(i => i.account !== null).map(i => i.account!.id))
}

describe('AccountPickerOverlay', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
    const accountStore = useAccountStore()
    accountStore.accounts = accounts
  })

  it('无 currentId 时不选中任何账户，深层账户不可见', async () => {
    const wrapper = mountOverlay(null)
    await flushPromises()

    const grid = wrapper.findAllComponents(AccountGrid)[0]
    expect(grid.props('selectedAccountId')).toBeNull()
    const rowIds = rowAccountIds(grid.props('rows'))
    expect(rowIds).not.toContain(3)
  })

  it('有 currentId 时选中当前账户并展开祖先链', async () => {
    const wrapper = mountOverlay(3)
    await flushPromises()

    const grid = wrapper.findAllComponents(AccountGrid)[0]
    expect(grid.props('selectedAccountId')).toBe(3)
    // 祖先展开后，当前账户（叶子）出现在行数据中
    const rowIds = rowAccountIds(grid.props('rows'))
    expect(rowIds).toContain(3)
    expect(rowIds).toContain(2)
  })

  it('不传 accountType 时渲染全部四个分组', async () => {
    const wrapper = mountOverlay(null)
    await flushPromises()

    expect(wrapper.findAllComponents(AccountGrid)).toHaveLength(4)
  })

  it('accountType 为 asset 时只渲染资产分组', async () => {
    const accountStore = useAccountStore()
    accountStore.accounts = multiTypeAccounts

    const wrapper = mountOverlay(null, 'asset')
    await flushPromises()

    const grids = wrapper.findAllComponents(AccountGrid)
    expect(grids).toHaveLength(1)
    const rowIds = rowAccountIds(grids[0].props('rows'))
    expect(rowIds).toContain(2)
    expect(rowIds).not.toContain(5)
  })

  it('accountType 为 expense 时只渲染支出分组', async () => {
    const accountStore = useAccountStore()
    accountStore.accounts = multiTypeAccounts

    const wrapper = mountOverlay(null, 'expense')
    await flushPromises()

    const grids = wrapper.findAllComponents(AccountGrid)
    expect(grids).toHaveLength(1)
    const rowIds = rowAccountIds(grids[0].props('rows'))
    expect(rowIds).toContain(5)
    expect(rowIds).not.toContain(2)
  })
})
