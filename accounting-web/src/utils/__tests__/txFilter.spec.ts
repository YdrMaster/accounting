import { describe, expect, it } from 'vitest'
import { buildTxQuery, isFilterActive } from '../txFilter'
import type { TxFilters } from '../../types/api'

describe('buildTxQuery', () => {
  it('returns empty params for null filter', () => {
    const params = buildTxQuery(null)
    expect(params.toString()).toBe('')
  })

  it('includes extra params', () => {
    const params = buildTxQuery(null, { to: '2026-07-25', limit: '100' })
    expect(params.get('to')).toBe('2026-07-25')
    expect(params.get('limit')).toBe('100')
  })

  it('serializes date range', () => {
    const filter: TxFilters = {
      from: '2026-07-01',
      to: '2026-07-25',
      accounts: [],
      members: [],
      tags: [],
      channels: [],
    }
    const params = buildTxQuery(filter)
    expect(params.get('from')).toBe('2026-07-01')
    expect(params.get('to')).toBe('2026-07-25')
  })

  it('serializes multi-value accounts with repeated keys', () => {
    const filter: TxFilters = {
      accounts: [1, 2, 3],
      members: [],
      tags: [],
      channels: [],
    }
    const params = buildTxQuery(filter)
    expect(params.getAll('account')).toEqual(['1', '2', '3'])
  })

  it('serializes tags by name', () => {
    const filter: TxFilters = {
      accounts: [],
      members: [],
      tags: ['餐饮', '交通'],
      channels: [],
    }
    const params = buildTxQuery(filter)
    expect(params.getAll('tag')).toEqual(['餐饮', '交通'])
  })

  it('serializes keyword and reimbursable', () => {
    const filter: TxFilters = {
      accounts: [],
      members: [],
      tags: [],
      channels: [],
      keyword: '咖啡',
      reimbursable: true,
    }
    const params = buildTxQuery(filter)
    expect(params.get('keyword')).toBe('咖啡')
    expect(params.get('reimbursable')).toBe('true')
  })

  it('omits reimbursable when false', () => {
    const filter: TxFilters = {
      accounts: [],
      members: [],
      tags: [],
      channels: [],
      reimbursable: false,
    }
    const params = buildTxQuery(filter)
    expect(params.has('reimbursable')).toBe(false)
  })

  it('extra params coexist with filter params', () => {
    const filter: TxFilters = {
      accounts: [5],
      members: [],
      tags: [],
      channels: [],
    }
    const params = buildTxQuery(filter, { limit: '100' })
    expect(params.get('limit')).toBe('100')
    expect(params.getAll('account')).toEqual(['5'])
  })
})

describe('isFilterActive', () => {
  it('returns false for null', () => {
    expect(isFilterActive(null)).toBe(false)
  })

  it('returns false for empty filter', () => {
    expect(
      isFilterActive({ accounts: [], members: [], tags: [], channels: [] })
    ).toBe(false)
  })

  it('returns true when any condition is set', () => {
    expect(
      isFilterActive({ accounts: [1], members: [], tags: [], channels: [] })
    ).toBe(true)
    expect(
      isFilterActive({ accounts: [], members: [], tags: ['x'], channels: [] })
    ).toBe(true)
    expect(
      isFilterActive({ from: '2026-01-01', accounts: [], members: [], tags: [], channels: [] })
    ).toBe(true)
    expect(
      isFilterActive({ accounts: [], members: [], tags: [], channels: [], keyword: 'a' })
    ).toBe(true)
  })
})
