import { beforeEach, describe, expect, it } from 'vitest'
import type { AccountDto } from '../../types/api'
import { loadSiblingOrder, saveSiblingOrder, sortSiblings } from '../siblingOrder'

function makeAccount(id: number, parentId: number | null = 10): AccountDto {
  return {
    id,
    name: `acc-${id}`,
    account_type: 'Asset',
    parent_id: parentId,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

beforeEach(() => {
  localStorage.clear()
})

describe('sortSiblings', () => {
  it('returns accounts untouched when no stored order', () => {
    const accounts = [makeAccount(2), makeAccount(1)]
    expect(sortSiblings(accounts, undefined)).toBe(accounts)
  })

  it('orders by stored ids and ignores unknown ids', () => {
    const accounts = [makeAccount(1), makeAccount(2), makeAccount(3)]
    const sorted = sortSiblings(accounts, [3, 99, 1])
    expect(sorted.map(a => a.id)).toEqual([3, 1, 2])
  })

  it('appends missing accounts by id ascending at the tail', () => {
    const accounts = [makeAccount(4), makeAccount(2), makeAccount(3), makeAccount(1)]
    const sorted = sortSiblings(accounts, [2])
    expect(sorted.map(a => a.id)).toEqual([2, 1, 3, 4])
  })
})

describe('loadSiblingOrder / saveSiblingOrder', () => {
  it('round-trips through localStorage', () => {
    const map = { '10': [3, 1, 2] }
    saveSiblingOrder(map)
    expect(loadSiblingOrder()).toEqual(map)
  })

  it('returns empty map on invalid json', () => {
    localStorage.setItem('account-sibling-order', 'not-json')
    expect(loadSiblingOrder()).toEqual({})
  })

  it('filters out non-number entries', () => {
    localStorage.setItem('account-sibling-order', JSON.stringify({ '10': [1, 'x', 2] }))
    expect(loadSiblingOrder()).toEqual({ '10': [1, 2] })
  })
})
