import { describe, expect, it } from 'vitest'
import { buildSunburstTree } from '../sunburst'

describe('buildSunburstTree', () => {
  it('returns empty tree for no items', () => {
    expect(buildSunburstTree([])).toEqual({ children: [], total: 0 })
  })

  it('builds a two-level tree from hierarchical totals', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '800' },
      { account: 'Expenses:餐饮', amount: '800' },
      { account: 'Expenses:餐饮:外卖', amount: '500' },
      { account: 'Expenses:餐饮:堂食', amount: '300' },
    ])

    expect(tree.total).toBe(800)
    expect(tree.children).toHaveLength(1)
    const food = tree.children[0]
    expect(food.name).toBe('餐饮')
    // 中间节点不带 value（由可见子节点求和决定角度），真实总额存于 total
    expect(food.value).toBeUndefined()
    expect(food.total).toBe(800)
    expect(food.children).toEqual([
      { name: '堂食', value: 300 },
      { name: '外卖', value: 500 },
    ])
  })

  it('creates a pseudo child for direct postings on intermediate nodes', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '1000' },
      { account: 'Expenses:餐饮', amount: '800' },
      { account: 'Expenses:餐饮:外卖', amount: '500' },
      { account: 'Expenses:交通', amount: '200' },
    ])

    expect(tree.total).toBe(1000)
    const food = tree.children.find(c => c.name === '餐饮')!
    // 餐饮汇总 800，子节点外卖 500 → 伪子节点承载差额 300
    expect(food.children).toContainEqual({ name: '外卖', value: 500 })
    expect(food.children).toContainEqual({ name: '餐饮', value: 300 })
  })

  it('handles multiple top-level categories', () => {
    const tree = buildSunburstTree([
      { account: 'Income', amount: '15000' },
      { account: 'Income:工资', amount: '12000' },
      { account: 'Income:理财', amount: '3000' },
    ])

    expect(tree.total).toBe(15000)
    expect(tree.children).toEqual([
      { name: '工资', value: 12000 },
      { name: '理财', value: 3000 },
    ])
  })

  it('ignores floating point residue below epsilon', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '100.00' },
      { account: 'Expenses:A', amount: '33.33' },
      { account: 'Expenses:B', amount: '33.33' },
      { account: 'Expenses:C', amount: '33.34' },
    ])

    // 33.33 + 33.33 + 33.34 = 100.00 → 无伪子节点
    expect(tree.children).toHaveLength(3)
    expect(tree.children.every(c => c.children === undefined)).toBe(true)
  })

  it('filters out children not exceeding 1% of the level total', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '1000' },
      { account: 'Expenses:餐饮', amount: '995' },
      { account: 'Expenses:交通', amount: '5' },
    ])

    // 交通占 0.5% < 1% → 被过滤；餐饮保留真实总额
    expect(tree.children).toEqual([{ name: '餐饮', value: 995 }])
    expect(tree.total).toBe(1000)
  })

  it('keeps children exceeding exactly 1% of the level total', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '1000' },
      { account: 'Expenses:餐饮', amount: '989' },
      { account: 'Expenses:交通', amount: '11' },
    ])

    // 交通占 1.1% > 1% → 保留
    expect(tree.children.map(c => c.name)).toEqual(['交通', '餐饮'])
  })

  it('applies the 1% filter at every level', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '1000' },
      { account: 'Expenses:餐饮', amount: '800' },
      { account: 'Expenses:餐饮:外卖', amount: '792' },
      { account: 'Expenses:餐饮:奶茶', amount: '8' },
      { account: 'Expenses:交通', amount: '200' },
    ])

    const food = tree.children.find(c => c.name === '餐饮')!
    // 奶茶占餐饮的 1%（未超过）→ 被过滤；餐饮真实总额 800 仍保留在 total 字段
    expect(food.value).toBeUndefined()
    expect(food.total).toBe(800)
    expect(food.children).toEqual([{ name: '外卖', value: 792 }])
  })

  it('drops a pseudo child below the 1% threshold', () => {
    const tree = buildSunburstTree([
      { account: 'Expenses', amount: '1000' },
      { account: 'Expenses:餐饮', amount: '999' },
      { account: 'Expenses:餐饮:外卖', amount: '999' },
    ])

    // 根级伪子节点差额 1（占 0.1%）→ 被过滤
    expect(tree.children).toHaveLength(1)
    expect(tree.children[0].name).toBe('餐饮')
  })
})
