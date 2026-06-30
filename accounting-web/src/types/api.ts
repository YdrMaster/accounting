export interface ChannelPathNodeDto {
  position: number
  channel_id: number
  reconciled: boolean
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
