import { describe, expect, it } from 'vitest'
import type { CashFlowItemDto } from '../../types/api'
import { buildSunburstTree } from '../sunburst'

function item(account_id: number, parent_id: number | null, name: string, amount: string): CashFlowItemDto {
  return { account_id, parent_id, name, amount }
}

describe('buildSunburstTree', () => {
  it('returns empty tree for no items', () => {
    expect(buildSunburstTree([])).toEqual({ children: [], total: 0 })
  })

  it('builds a two-level tree from hierarchical totals', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '800'),
      item(2, 1, '餐饮', '800'),
      item(3, 2, '外卖', '500'),
      item(4, 2, '堂食', '300'),
    ])

    expect(tree.total).toBe(800)
    expect(tree.children).toHaveLength(1)
    const food = tree.children[0]
    expect(food.name).toBe('餐饮')
    expect(food.accountId).toBe(2)
    // 中间节点不带 value（由可见子节点求和决定角度），真实总额存于 total
    expect(food.value).toBeUndefined()
    expect(food.total).toBe(800)
    // 兄弟节点按金额降序
    expect(food.children).toEqual([
      { name: '外卖', value: 500, accountId: 3 },
      { name: '堂食', value: 300, accountId: 4 },
    ])
  })

  it('creates a pseudo child for direct postings on intermediate nodes', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '1000'),
      item(2, 1, '餐饮', '800'),
      item(3, 2, '外卖', '500'),
      item(4, 1, '交通', '200'),
    ])

    expect(tree.total).toBe(1000)
    const food = tree.children.find(c => c.name === '餐饮')!
    // 餐饮汇总 800，子节点外卖 500 → 伪子节点承载差额 300，无 accountId 且不响应点击
    expect(food.children).toContainEqual({ name: '外卖', value: 500, accountId: 3 })
    expect(food.children).toContainEqual({ name: '餐饮', value: 300, nodeClick: false })
  })

  it('handles multiple top-level categories', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Income', '15000'),
      item(2, 1, '工资', '12000'),
      item(3, 1, '理财', '3000'),
    ])

    expect(tree.total).toBe(15000)
    expect(tree.children).toEqual([
      { name: '工资', value: 12000, accountId: 2 },
      { name: '理财', value: 3000, accountId: 3 },
    ])
  })

  it('ignores floating point residue below epsilon', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '100.00'),
      item(2, 1, 'A', '33.33'),
      item(3, 1, 'B', '33.33'),
      item(4, 1, 'C', '33.34'),
    ])

    // 33.33 + 33.33 + 33.34 = 100.00 → 无伪子节点
    expect(tree.children).toHaveLength(3)
    expect(tree.children.every(c => c.children === undefined)).toBe(true)
  })

  it('filters out children not exceeding 1% of the level total', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '1000'),
      item(2, 1, '餐饮', '995'),
      item(3, 1, '交通', '5'),
    ])

    // 交通占 0.5% < 1% → 被过滤；餐饮保留真实总额
    expect(tree.children).toEqual([{ name: '餐饮', value: 995, accountId: 2 }])
    expect(tree.total).toBe(1000)
  })

  it('keeps children exceeding exactly 1% of the level total', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '1000'),
      item(2, 1, '餐饮', '989'),
      item(3, 1, '交通', '11'),
    ])

    // 交通占 1.1% > 1% → 保留（按金额降序：餐饮在前）
    expect(tree.children.map(c => c.name)).toEqual(['餐饮', '交通'])
  })

  it('applies the 1% filter at every level', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '1000'),
      item(2, 1, '餐饮', '800'),
      item(3, 2, '外卖', '792'),
      item(4, 2, '奶茶', '8'),
      item(5, 1, '交通', '200'),
    ])

    const food = tree.children.find(c => c.name === '餐饮')!
    // 奶茶占餐饮的 1%（未超过）→ 被过滤；餐饮真实总额 800 仍保留在 total 字段
    expect(food.value).toBeUndefined()
    expect(food.total).toBe(800)
    expect(food.children).toEqual([{ name: '外卖', value: 792, accountId: 3 }])
  })

  it('drops a pseudo child below the 1% threshold', () => {
    const tree = buildSunburstTree([
      item(1, null, 'Expenses', '1000'),
      item(2, 1, '餐饮', '999'),
      item(3, 2, '外卖', '999'),
    ])

    // 根级伪子节点差额 1（占 0.1%）→ 被过滤
    expect(tree.children).toHaveLength(1)
    expect(tree.children[0].name).toBe('餐饮')
  })
})
