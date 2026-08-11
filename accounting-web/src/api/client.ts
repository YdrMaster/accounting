import { i18n } from '../i18n'

const BASE_URL = '/api'

function apiUrl(path: string): string {
  const sep = path.includes('?') ? '&' : '?'
  return `${BASE_URL}${path}${sep}lang=${encodeURIComponent(i18n.global.locale.value)}`
}

/** 带 HTTP 状态码的 API 错误，message 为服务端响应原文 */
export class ApiError extends Error {
  readonly status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

/** 业务请求 401（会话过期）时的回调，由 App 注册（避免 client 与 auth store 循环依赖） */
let unauthorizedHandler: (() => void) | null = null

export function setUnauthorizedHandler(handler: (() => void) | null): void {
  unauthorizedHandler = handler
}

async function rawFetch(path: string, init?: RequestInit): Promise<Response> {
  const res = await fetch(apiUrl(path), init)
  // /auth/* 接口自身的 401（未登录、凭证错误）不触发全局登出
  if (res.status === 401 && !path.startsWith('/auth')) {
    unauthorizedHandler?.()
  }
  return res
}

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await rawFetch(path, init)
  if (!res.ok) {
    const text = await res.text()
    throw new ApiError(res.status, text || res.statusText)
  }
  return res.json() as Promise<T>
}

/** 从 apiFetch 抛出的错误中提取服务端 `{"error": "..."}` 文案 */
export function apiErrorMessage(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e)
  try {
    const parsed: unknown = JSON.parse(raw)
    if (parsed && typeof (parsed as { error?: unknown }).error === 'string') {
      return (parsed as { error: string }).error
    }
  } catch {
    // 非 JSON 响应，原文返回
  }
  return raw
}

/** 从 apiFetch 抛出的错误中提取服务端 `{"code": "..."}` 稳定码（无 code 返回空串）。
 * 客户端按 code 做逻辑分支，不解析本地化文案。 */
export function apiErrorCode(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e)
  try {
    const parsed: unknown = JSON.parse(raw)
    if (parsed && typeof (parsed as { code?: unknown }).code === 'string') {
      return (parsed as { code: string }).code
    }
  } catch {
    // 非 JSON 响应
  }
  return ''
}

import type {
  AccountDto,
  BalanceSheetDto,
  BudgetDetailDto,
  BudgetDto,
  BudgetStatusDto,
  CashFlowDto,
  ChannelDto,
  ChartPeriod,
  CommodityDto,
  CreateAccountRequest,
  CreateBudgetRequest,
  CreateSavingPlanRequest,
  CreateTransactionData,
  DailySummaryDto,
  ImportResultDto,
  LoginResultDto,
  MappingDto,
  MeDto,
  MemberDto,
  MoveAccountRequest,
  NetWorthTrendDto,
  SavingPlanDetailDto,
  SavingPlanDto,
  SavingPlanStatusDto,
  TagDto,
  TotpEnableDto,
  TotpSetupDto,
  TransactionDto,
  UpdateSavingPlanRequest,
} from '../types/api'

export async function fetchAccounts(): Promise<AccountDto[]> {
  return apiFetch<AccountDto[]>('/accounts')
}

export async function fetchMembers(): Promise<MemberDto[]> {
  return apiFetch<MemberDto[]>('/members')
}

export async function renameAccount(id: number, name: string): Promise<void> {
  const res = await rawFetch(`/accounts/${id}/rename`, {
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
  const res = await rawFetch(`/accounts/${id}/owner`, {
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
  const res = await rawFetch(`/accounts/${id}/close`, { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function reopenAccount(id: number): Promise<void> {
  const res = await rawFetch(`/accounts/${id}/open`, { method: 'PUT' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteAccount(id: number): Promise<void> {
  const res = await rawFetch(`/accounts/${id}`, { method: 'DELETE' })
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
  const res = await rawFetch(`/accounts/${id}/fields`, {
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
  const res = await rawFetch(`/accounts`, {
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
  const res = await rawFetch(`/accounts/${id}/parent`, {
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

export async function fetchTransactions(params?: URLSearchParams): Promise<TransactionDto[]> {
  const qs = params && params.toString() ? '?' + params.toString() : ''
  return apiFetch<TransactionDto[]>(`/transactions${qs}`)
}

export async function fetchTransaction(id: number): Promise<TransactionDto> {
  return apiFetch<TransactionDto>(`/transactions/${id}`)
}

export async function createTransaction(data: CreateTransactionData): Promise<number> {
  const res = await rawFetch(`/transactions`, {
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
  const res = await rawFetch(`/transactions/${id}`, {
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
  const res = await rawFetch(`/transactions/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

// ─── 报表 ───

export async function fetchBalanceSheet(): Promise<BalanceSheetDto> {
  return apiFetch<BalanceSheetDto>('/reports/balance-sheet')
}

export async function fetchDailySummary(from: string, to: string): Promise<DailySummaryDto[]> {
  const qs = new URLSearchParams({ from, to }).toString()
  return apiFetch<DailySummaryDto[]>(`/reports/daily-summary?${qs}`)
}

export async function fetchNetWorthTrend(period: ChartPeriod): Promise<NetWorthTrendDto> {
  const qs = new URLSearchParams({ period }).toString()
  return apiFetch<NetWorthTrendDto>(`/reports/net-worth-trend?${qs}`)
}

export async function fetchCashFlow(date: string, period: ChartPeriod): Promise<CashFlowDto> {
  const qs = new URLSearchParams({ date, period }).toString()
  return apiFetch<CashFlowDto>(`/reports/cash-flow?${qs}`)
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

export async function fetchBudgetStatuses(date?: string): Promise<BudgetStatusDto[]> {
  const qs = date ? `?date=${date}` : ''
  return apiFetch<BudgetStatusDto[]>(`/budgets/statuses${qs}`)
}

export async function createBudget(data: CreateBudgetRequest): Promise<BudgetDto> {
  const res = await rawFetch(`/budgets`, {
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
  const res = await rawFetch(`/budgets/${id}`, {
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
  const res = await rawFetch(`/budgets/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

// ─── 攒钱计划 CRUD ───

export async function fetchSavingPlans(): Promise<SavingPlanDto[]> {
  return apiFetch<SavingPlanDto[]>('/saving-plans')
}

export async function fetchSavingPlanStatuses(date?: string): Promise<SavingPlanStatusDto[]> {
  const qs = date ? `?date=${date}` : ''
  return apiFetch<SavingPlanStatusDto[]>(`/saving-plans/statuses${qs}`)
}

export async function fetchSavingPlanDetail(id: number): Promise<SavingPlanDetailDto> {
  return apiFetch<SavingPlanDetailDto>(`/saving-plans/${id}`)
}

export async function fetchSavingPlanStatus(
  id: number,
  date?: string
): Promise<SavingPlanStatusDto> {
  const qs = date ? `?date=${date}` : ''
  return apiFetch<SavingPlanStatusDto>(`/saving-plans/${id}/status${qs}`)
}

export async function createSavingPlan(data: CreateSavingPlanRequest): Promise<SavingPlanDto> {
  const res = await rawFetch(`/saving-plans`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<SavingPlanDto>
}

export async function updateSavingPlan(id: number, data: UpdateSavingPlanRequest): Promise<void> {
  const res = await rawFetch(`/saving-plans/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function deleteSavingPlan(id: number): Promise<void> {
  const res = await rawFetch(`/saving-plans/${id}`, { method: 'DELETE' })
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
  const res = await rawFetch(`/channels`, {
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
  const res = await rawFetch(`/channels/${id}`, {
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
  const res = await rawFetch(`/channels/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function importBill(
  channelId: number,
  memberId: number,
  file: File
): Promise<ImportResultDto> {
  const res = await rawFetch(`/channels/${channelId}/import?member_id=${memberId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/octet-stream' },
    body: file,
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.json() as Promise<ImportResultDto>
}

export async function fetchTags(): Promise<TagDto[]> {
  return apiFetch<TagDto[]>('/tags')
}

// ─── 导入映射 ───

export async function fetchMappings(memberId: number, channelId: number): Promise<MappingDto[]> {
  const qs = new URLSearchParams({
    member_id: String(memberId),
    channel_id: String(channelId),
  }).toString()
  return apiFetch<MappingDto[]>(`/mappings?${qs}`)
}

export async function upsertMapping(dto: MappingDto): Promise<string> {
  const res = await rawFetch(`/mappings`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(dto),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
  return res.text()
}

export async function deleteMapping(
  memberId: number,
  channelId: number,
  category: string
): Promise<void> {
  const qs = new URLSearchParams({
    member_id: String(memberId),
    channel_id: String(channelId),
    category,
  }).toString()
  const res = await rawFetch(`/mappings?${qs}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function createTag(data: { name: string; description?: string }): Promise<TagDto> {
  const res = await rawFetch(`/tags`, {
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
  const res = await rawFetch(`/tags/${id}`, {
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
  const res = await rawFetch(`/tags/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

export async function createMember(name: string): Promise<MemberDto> {
  const res = await rawFetch(`/members`, {
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
  const res = await rawFetch(`/members/${id}`, {
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
  const res = await rawFetch(`/members/${id}`, { method: 'DELETE' })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || res.statusText)
  }
}

// ─── 认证 ───

const JSON_HEADERS = { 'Content-Type': 'application/json' }

export async function login(username: string, password: string): Promise<LoginResultDto> {
  return apiFetch<LoginResultDto>('/auth/login', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ username, password }),
  })
}

export async function loginTotp(pendingToken: string, code: string): Promise<LoginResultDto> {
  return apiFetch<LoginResultDto>('/auth/login/totp', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ pending_token: pendingToken, code }),
  })
}

export async function fetchMe(): Promise<MeDto> {
  return apiFetch<MeDto>('/auth/me')
}

export async function logout(): Promise<void> {
  const res = await rawFetch('/auth/logout', { method: 'POST' })
  if (!res.ok) {
    const text = await res.text()
    throw new ApiError(res.status, text || res.statusText)
  }
}

export async function totpSetup(): Promise<TotpSetupDto> {
  return apiFetch<TotpSetupDto>('/auth/totp/setup', { method: 'POST' })
}

export async function totpEnable(code: string): Promise<TotpEnableDto> {
  return apiFetch<TotpEnableDto>('/auth/totp/enable', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ code }),
  })
}
