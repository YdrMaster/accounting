const BASE_URL = '/api'

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, init)
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<T>
}

import type { AccountDto, MemberDto } from '../types/api'

export async function fetchAccounts(): Promise<AccountDto[]> {
  return apiFetch<AccountDto[]>('/accounts')
}

export async function fetchMembers(): Promise<MemberDto[]> {
  return apiFetch<MemberDto[]>('/members')
}

export async function renameAccount(id: number, name: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}/rename`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function setAccountOwners(id: number, ownerIds: number[]): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}/owner`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ owner_ids: ownerIds }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function closeAccount(id: number): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}/close`, { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function reopenAccount(id: number): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}/open`, { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteAccount(id: number): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function updateAccountFields(
  id: number,
  billingDay: number | null,
  repaymentDay: number | null,
): Promise<void> {
  const res = await fetch(`${BASE_URL}/accounts/${id}/fields`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ billing_day: billingDay, repayment_day: repaymentDay }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}
