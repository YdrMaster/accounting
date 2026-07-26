import type { TxFilters } from '../types/api'

export function buildTxQuery(
  filter: TxFilters | null,
  extra?: Record<string, string>
): URLSearchParams {
  const params = new URLSearchParams()
  if (extra) {
    for (const [k, v] of Object.entries(extra)) {
      params.set(k, v)
    }
  }
  if (!filter) return params
  if (filter.from) params.set('from', filter.from)
  if (filter.to) params.set('to', filter.to)
  for (const id of filter.accounts) params.append('account', String(id))
  for (const id of filter.members) params.append('member', String(id))
  for (const name of filter.tags) params.append('tag', name)
  for (const id of filter.channels) params.append('channel', String(id))
  if (filter.keyword) params.set('keyword', filter.keyword)
  if (filter.reimbursable) params.set('reimbursable', 'true')
  return params
}

export function isFilterActive(filter: TxFilters | null): boolean {
  if (!filter) return false
  return !!(
    filter.from ||
    filter.to ||
    filter.accounts.length ||
    filter.members.length ||
    filter.tags.length ||
    filter.channels.length ||
    filter.keyword ||
    filter.reimbursable
  )
}
