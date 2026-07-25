import { describe, expect, it } from 'vitest'
import type { CashFlowItemDto } from '../../types/api'
import { buildDetailRows } from '../cashFlowList'

function item(account_id: number, parent_id: number | null, name: string, amount: string): CashFlowItemDto {
  return { account_id, parent_id, name, amount }
}

const fixture = [
  item(1, null, 'Expenses', '1000'),
  item(2, 1, '餐饮', '800'),
  item(3, 2, '外卖', '500'),
  item(4, 2, '奶茶', '8'),
  item(5, 2, '聚餐', '292'),
  item(6, 1, '交通', '200'),
]

describe('buildDetailRows', () => {
  it('returns empty for no items', () => {
    expect(buildDetailRows([], null)).toEqual([])
  })

  it('shows top-level categories sorted by amount desc when not drilled', () => {
    const rows = buildDetailRows(fixture, null)
    expect(rows.map(r => r.name)).toEqual(['餐饮', '外卖', '聚餐', '奶茶', '交通'])
    expect(rows.map(r => r.depth)).toEqual([0, 1, 1, 1, 0])
    // ratio 相对根总额 1000
    expect(rows[0].ratio).toBeCloseTo(0.8)
    expect(rows[4].ratio).toBeCloseTo(0.2)
  })

  it('shows drilled account itself plus descendants, ratio relative to drilled amount', () => {
    const rows = buildDetailRows(fixture, 2)
    // 奶茶 8 元（占餐饮 1%）仍显示；子级按金额降序
    expect(rows.map(r => r.name)).toEqual(['餐饮', '外卖', '聚餐', '奶茶'])
    expect(rows.map(r => r.depth)).toEqual([0, 1, 1, 1])
    expect(rows[0].ratio).toBeCloseTo(1)
    expect(rows[1].ratio).toBeCloseTo(500 / 800)
    expect(rows[3].ratio).toBeCloseTo(8 / 800)
  })

  it('falls back to root view for unknown drill id', () => {
    const rows = buildDetailRows(fixture, 999)
    expect(rows.map(r => r.name)).toEqual(['餐饮', '外卖', '聚餐', '奶茶', '交通'])
  })

  it('returns empty when base amount is zero', () => {
    const rows = buildDetailRows([item(1, null, 'Income', '0')], null)
    expect(rows).toEqual([])
  })
})
