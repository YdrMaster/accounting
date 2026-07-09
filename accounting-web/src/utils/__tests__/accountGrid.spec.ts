import { describe, expect, it } from 'vitest'
import type { AccountDto } from '../../types/api'
import { buildRowTree, compileRows } from '../accountGrid'

function makeAccount(id: number, parentId: number | null = null, name = `acc-${id}`): AccountDto {
  return {
    id,
    name,
    account_type: 'Asset',
    parent_id: parentId,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

describe('compileRows', () => {
  it('renders roots left-aligned with trailing placeholders when no expansion', () => {
    const roots = [makeAccount(1), makeAccount(2)]
    const rows = compileRows(roots, [], 3, () => [])
    expect(rows).toHaveLength(1)
    expect(rows[0].items).toHaveLength(3)
    expect(rows[0].items[0].account?.id).toBe(1)
    expect(rows[0].items[1].account?.id).toBe(2)
    expect(rows[0].items[2].isPlaceholder).toBe(true)
    expect(rows[0].items[2].hasChildren).toBe(false)
  })

  it('expands a root and renders its children on the next row', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(4, 2), makeAccount(5, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    expect(rows).toHaveLength(2)
    expect(rows[0].expandedAccountId).toBe(2)
    expect(rows[1].items[0].account?.id).toBe(4)
    expect(rows[1].items[1].account?.id).toBe(5)
    expect(rows[1].items[2].isPlaceholder).toBe(true)
  })

  it('renders deeper paths and respects depth-first order', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3)]
    const children: Record<number, AccountDto[]> = {
      3: [makeAccount(4, 3), makeAccount(5, 3)],
      5: [makeAccount(6, 5)],
    }
    const rows = compileRows(roots, [3, 5], 3, id => children[id] ?? [])
    expect(rows.map(r => r.items.map(i => i.account?.id ?? null))).toEqual([
      [1, 2, 3],
      [4, 5, null],
      [6, null, null],
    ])
  })

  it('finishes a subtree before rendering remaining siblings', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3), makeAccount(4)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(5, 2), makeAccount(6, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    expect(rows.map(r => r.items.map(i => i.account?.id ?? null))).toEqual([
      [1, 2, 3],
      [5, 6, null],
      [4, null, null],
    ])
  })
})

describe('buildRowTree', () => {
  it('groups child rows under their parent row', () => {
    const roots = [makeAccount(1), makeAccount(2)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(3, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    const tree = buildRowTree(rows)
    expect(tree).toHaveLength(1)
    expect(tree[0].row.items[0].account?.id).toBe(1)
    expect(tree[0].children).toHaveLength(1)
    expect(tree[0].children[0].row.items[1].account?.id).toBe(2)
    expect(tree[0].children[0].children).toHaveLength(1)
    expect(tree[0].children[0].children[0].row.items[0].account?.id).toBe(3)
  })
})
