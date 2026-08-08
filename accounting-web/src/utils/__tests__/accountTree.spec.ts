import { describe, expect, it } from 'vitest'
import { expandSubtree, type AccountTreeNode } from '../accountTree'

const tree: AccountTreeNode[] = [
  { id: 1, parent_id: null }, // Expenses
  { id: 2, parent_id: 1 }, // 餐饮
  { id: 3, parent_id: 2 }, // 餐饮:外卖
  { id: 4, parent_id: 2 }, // 餐饮:聚餐
  { id: 5, parent_id: 1 }, // 交通
  { id: 6, parent_id: null }, // Income
]

describe('expandSubtree', () => {
  it('展开多级子树：自身 + 全部后代', () => {
    expect(expandSubtree(tree, 1).sort()).toEqual([1, 2, 3, 4, 5])
  })

  it('展开中间层：不含祖先与兄弟', () => {
    expect(expandSubtree(tree, 2).sort()).toEqual([2, 3, 4])
  })

  it('叶子账户仅含自身', () => {
    expect(expandSubtree(tree, 3)).toEqual([3])
  })

  it('未知 ID 兜底返回自身', () => {
    expect(expandSubtree(tree, 99)).toEqual([99])
  })

  it('空账户列表兜底返回自身', () => {
    expect(expandSubtree([], 1)).toEqual([1])
  })
})
