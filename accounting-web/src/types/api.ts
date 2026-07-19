export interface ChannelPathNodeDto {
  position: number
  channel_id: number
  channel_name: string
  status: 'default' | 'pending' | 'verified'
}

export interface PostingDto {
  id: number
  transaction_id: number
  account: string
  account_type: string
  commodity: string
  amount: string
  is_reimbursable: boolean
  linked_posting_id: number | null
  reversal_total: string
}

export interface TransactionDto {
  id: number
  date_time: string
  description: string
  kind: string
  member_id: number
  member_name: string
  tags: string[]
  channel_paths: ChannelPathNodeDto[]
  postings: PostingDto[]
}

export interface SummaryDto {
  income: string
  expense: string
}

export interface DailySummaryDto {
  date: string
  income: string
  expense: string
}

export interface MemberDto {
  id: number
  name: string
}

export interface AccountDto {
  id: number
  name: string
  account_type: string
  parent_id: number | null
  closed_at: string | null
  is_system: boolean
  billing_day: number | null
  repayment_day: number | null
  owner_ids: number[]
}

// ─── 报表 ───

export interface BalanceEntryDto {
  commodity_id: number
  amount: string
}

export interface AccountBalanceItemDto {
  account: string
  balances: BalanceEntryDto[]
}

export interface BalanceSheetDto {
  assets: AccountBalanceItemDto[]
}

// ─── 预算 ───

export interface BudgetDto {
  id: number
  name: string
  period: string
  commodity_id: number
}

export interface BudgetLimitDto {
  account_id: number
  amount: string
}

export interface BudgetDetailDto {
  budget: BudgetDto
  limits: BudgetLimitDto[]
}

export interface BudgetItemStatusDto {
  account_id: number
  limit_amount: string
  actual_amount: string
  remaining: string
  percentage: string
}

export interface BudgetStatusDto {
  budget: BudgetDto
  period_start: string
  period_end: string
  items: BudgetItemStatusDto[]
}

export interface BudgetLimitRequest {
  account_id: number
  amount: string
}

export interface CreateBudgetRequest {
  name: string
  period: string
  commodity_id: number
  limits: BudgetLimitRequest[]
}

// ─── 交易 ───

export interface PostingInput {
  account: string
  commodity: string
  amount: string
  is_reimbursable?: boolean
  linked_posting_id?: number
}

export interface ChannelPathNodeInput {
  position: number
  channel_id: number
  status?: string
}

export interface CreateTransactionData {
  date_time: string
  description: string
  kind: string
  member_id: number
  channel_paths: ChannelPathNodeInput[]
  postings: PostingInput[]
  tags: string[]
}

// ─── 辅助数据 ───

export interface CommodityDto {
  id: number
  symbol: string
  name: string
  precision: number
}

export interface ChannelDto {
  id: number
  name: string
  description: string | null
  account_id: number | null
  is_system: boolean
}

export interface TagDto {
  id: number
  name: string
  description: string | null
  is_system: boolean
}

export interface CreateAccountRequest {
  name: string
  parent_id?: number
  billing_day?: number
  repayment_day?: number
  owner_ids: number[]
}

export interface MoveAccountRequest {
  parent_id: number
}
