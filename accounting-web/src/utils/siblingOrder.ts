import type { AccountDto } from '../types/api'

const STORAGE_KEY = 'account-sibling-order'

export type SiblingOrderMap = Record<string, number[]>

export function loadSiblingOrder(): SiblingOrderMap {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    const parsed: unknown = JSON.parse(raw)
    if (typeof parsed !== 'object' || parsed === null) return {}
    const map: SiblingOrderMap = {}
    for (const [key, value] of Object.entries(parsed)) {
      if (Array.isArray(value)) {
        map[key] = value.filter((v): v is number => typeof v === 'number')
      }
    }
    return map
  } catch {
    return {}
  }
}

export function saveSiblingOrder(map: SiblingOrderMap): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map))
  } catch {
    // ignore storage failures (private mode etc.)
  }
}

/**
 * Sort a group of sibling accounts by the stored order for their parent.
 * Ids in the stored order that no longer exist are ignored; accounts missing
 * from the stored order keep their original order (id ascending) at the tail.
 */
export function sortSiblings(accounts: AccountDto[], order: number[] | undefined): AccountDto[] {
  if (!order || order.length === 0) return accounts
  const rank = new Map<number, number>()
  order.forEach((id, index) => {
    if (!rank.has(id)) rank.set(id, index)
  })
  const known: AccountDto[] = []
  const missing: AccountDto[] = []
  for (const account of accounts) {
    if (rank.has(account.id)) {
      known.push(account)
    } else {
      missing.push(account)
    }
  }
  known.sort((a, b) => rank.get(a.id)! - rank.get(b.id)!)
  missing.sort((a, b) => a.id - b.id)
  return [...known, ...missing]
}
