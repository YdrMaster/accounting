import { i18n } from '../i18n'

const BASE_URL = '/api'

function apiUrl(path: string): string {
  const sep = path.includes('?') ? '&' : '?'
  return `${BASE_URL}${path}${sep}lang=${encodeURIComponent(i18n.global.locale.value)}`
}

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(apiUrl(path), init)
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<T>
}

import type {
  AccountDto,
  BalanceSheetDto,
  BudgetDetailDto,
  BudgetDto,
  BudgetStatusDto,
  ChannelDto,
  CommodityDto,
  CreateAccountRequest,
  CreateBudgetRequest,
  CreateTransactionData,
  MemberDto,
  MoveAccountRequest,
  TagDto,
  TransactionDto,
} from '../types/api'

export async function fetchAccounts(): Promise<AccountDto[]> {
  return apiFetch<AccountDto[]>('/accounts')
}

export async function fetchMembers(): Promise<MemberDto[]> {
  return apiFetch<MemberDto[]>('/members')
}

export async function renameAccount(id: number, name: string): Promise<void> {
  const res = await fetch(apiUrl(`/accounts/${id}/rename`), {
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
  const res = await fetch(apiUrl(`/accounts/${id}/owner`), {
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
  const res = await fetch(apiUrl(`/accounts/${id}/close`), { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function reopenAccount(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/accounts/${id}/open`), { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteAccount(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/accounts/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function updateAccountFields(
  id: number,
  billingDay: number | null,
  repaymentDay: number | null
): Promise<void> {
  const res = await fetch(apiUrl(`/accounts/${id}/fields`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ billing_day: billingDay, repayment_day: repaymentDay }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function createAccount(data: CreateAccountRequest): Promise<number> {
  const res = await fetch(apiUrl(`/accounts`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<number>
}

export async function moveAccount(id: number, parentId: number): Promise<AccountDto> {
  const res = await fetch(apiUrl(`/accounts/${id}/parent`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ parent_id: parentId } satisfies MoveAccountRequest),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<AccountDto>
}

// ─── 交易 CRUD ───

export async function fetchTransactions(
  params?: Record<string, string>
): Promise<TransactionDto[]> {
  const qs = params ? '?' + new URLSearchParams(params).toString() : ''
  return apiFetch<TransactionDto[]>(`/transactions${qs}`)
}

export async function fetchTransaction(id: number): Promise<TransactionDto> {
  return apiFetch<TransactionDto>(`/transactions/${id}`)
}

export async function createTransaction(data: CreateTransactionData): Promise<number> {
  const res = await fetch(apiUrl(`/transactions`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<number>
}

export async function updateTransaction(id: number, data: CreateTransactionData): Promise<void> {
  const res = await fetch(apiUrl(`/transactions/${id}`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteTransaction(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/transactions/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

// ─── 报表 ───

export async function fetchBalanceSheet(): Promise<BalanceSheetDto> {
  return apiFetch<BalanceSheetDto>('/reports/balance-sheet')
}

// ─── 预算 CRUD ───

export async function fetchBudgets(): Promise<BudgetDto[]> {
  return apiFetch<BudgetDto[]>('/budgets')
}

export async function fetchBudgetDetail(id: number): Promise<BudgetDetailDto> {
  return apiFetch<BudgetDetailDto>(`/budgets/${id}`)
}

export async function fetchBudgetStatus(id: number, date?: string): Promise<BudgetStatusDto> {
  const qs = date ? `?date=${date}` : ''
  return apiFetch<BudgetStatusDto>(`/budgets/${id}/status${qs}`)
}

export async function createBudget(data: CreateBudgetRequest): Promise<BudgetDto> {
  const res = await fetch(apiUrl(`/budgets`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<BudgetDto>
}

export async function updateBudget(id: number, data: CreateBudgetRequest): Promise<void> {
  const res = await fetch(apiUrl(`/budgets/${id}`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteBudget(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/budgets/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

// ─── 辅助数据 ───

export async function fetchCommodities(): Promise<CommodityDto[]> {
  return apiFetch<CommodityDto[]>('/commodities')
}

export async function fetchChannels(): Promise<ChannelDto[]> {
  return apiFetch<ChannelDto[]>('/channels')
}

export async function createChannel(data: {
  name: string
  description?: string
  account_id?: number
}): Promise<number> {
  const res = await fetch(apiUrl(`/channels`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<number>
}

export async function updateChannel(
  id: number,
  data: { name?: string; description?: string; account_id?: number }
): Promise<void> {
  const res = await fetch(apiUrl(`/channels/${id}`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteChannel(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/channels/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function fetchTags(): Promise<TagDto[]> {
  return apiFetch<TagDto[]>('/tags')
}

export async function createTag(data: { name: string; description?: string }): Promise<TagDto> {
  const res = await fetch(apiUrl(`/tags`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<TagDto>
}

export async function updateTag(
  id: number,
  data: { name?: string; description?: string }
): Promise<TagDto> {
  const res = await fetch(apiUrl(`/tags/${id}`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<TagDto>
}

export async function deleteTag(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/tags/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function createMember(name: string): Promise<MemberDto> {
  const res = await fetch(apiUrl(`/members`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<MemberDto>
}

export async function renameMember(id: number, name: string): Promise<MemberDto> {
  const res = await fetch(apiUrl(`/members/${id}`), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<MemberDto>
}

export async function deleteMember(id: number): Promise<void> {
  const res = await fetch(apiUrl(`/members/${id}`), { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}
