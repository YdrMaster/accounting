import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import type { AccountDto } from '../../../types/api'
import type { GridRow } from '../../../utils/accountGrid'
import AccountGrid from '../AccountGrid.vue'

const account = (id: number, name: string): AccountDto => ({
  id,
  name,
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: true,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
})

const root = account(1, 'Assets')

function mountGrid(rows: GridRow[], extraProps: Record<string, unknown> = {}) {
  return mount(AccountGrid, {
    props: {
      typeLabel: '资产',
      rootAccount: root,
      rows,
      selectedAccountId: null,
      ...extraProps,
    },
  })
}

const rowOf = (...accounts: (AccountDto | null)[]) => ({
  items: accounts.map(account => ({
    account,
    isPlaceholder: account === null,
    hasChildren: false,
  })),
  depth: 0,
  expandedIndex: null,
  expandedAccountId: null,
  parentRowIndex: null,
})

describe('AccountGrid', () => {
  it('renders type label and rows', () => {
    const wrapper = mountGrid([rowOf(account(2, 'A'), null)])
    expect(wrapper.text()).toContain('资产')
    expect(wrapper.text()).toContain('A')
  })

  it('renders flat grid with fixed columns and placeholder padding', () => {
    const wrapper = mountGrid([
      rowOf(account(2, 'A'), account(3, 'B'), null),
      rowOf(account(4, 'C'), null, null),
    ])
    const cards = wrapper.findAll('.account-card')
    expect(cards).toHaveLength(6)
    const placeholders = wrapper.findAll('.account-card.placeholder')
    expect(placeholders).toHaveLength(3)
    expect(wrapper.text()).toContain('A')
    expect(wrapper.text()).toContain('B')
    expect(wrapper.text()).toContain('C')
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

    const wrapper = mountGrid([])
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('columnsChange')?.[0]).toEqual([3])

    vi.restoreAllMocks()
  })

  it('分区网格标注根账户 id 作为容器标识', () => {
    const wrapper = mountGrid([])
    const grid = wrapper.find('.account-grid')
    expect(grid.attributes('data-container-account-id')).toBe('1')
  })

  it('空容纳框渲染占位区', () => {
    const wrapper = mountGrid([])
    expect(wrapper.find('.grid-empty-placeholder').exists()).toBe(true)
  })

  it('非空容纳框不渲染占位区', () => {
    const wrapper = mountGrid([rowOf(account(2, 'A'), null)])
    expect(wrapper.find('.grid-empty-placeholder').exists()).toBe(false)
  })

  it('选中容纳框时渲染选中标识，冒泡 containerClick', async () => {
    const wrapper = mountGrid([], { selectedContainerId: 1 })
    expect(wrapper.find('.account-grid.container-selected').exists()).toBe(true)

    await wrapper.find('.account-grid').trigger('click')
    expect(wrapper.emitted('containerClick')?.[0]?.[0]).toBe(1)
  })

  it('拖放目标容器渲染虚线高亮标识', () => {
    const wrapper = mountGrid([], { dropContainerId: 1 })
    expect(wrapper.find('.account-grid.container-drop-target').exists()).toBe(true)
  })
})
