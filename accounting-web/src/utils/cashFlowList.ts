import type { CashFlowItemDto } from '../types/api'

export interface DetailRow {
  accountId: number
  name: string
  amount: number
  /** 占基准金额（未下钻为该侧总额，下钻为被钻账户金额）的比例，0~1 */
  ratio: number
  /** 树状缩进深度，从 0 开始 */
  depth: number
}

const EPSILON = 1e-6

/**
 * 构建收支变动详情列表行：按 account_id / parent_id 链接成树，
 * 未下钻（drillId 为 null）时以根账户的子账户为起点，下钻时以被钻账户自身为起点，
 * 展示起点账户及其各级后代；每级兄弟按金额降序；ratio 相对基准金额
 * （未下钻 = 根账户金额，下钻 = 被钻账户金额）。
 */
export function buildDetailRows(items: CashFlowItemDto[], drillId: number | null): DetailRow[] {
  if (items.length === 0) return []

  const byId = new Map(items.map(i => [i.account_id, i]))
  const childMap = new Map<number, CashFlowItemDto[]>()
  let root: CashFlowItemDto | undefined
  for (const item of items) {
    if (item.parent_id != null && byId.has(item.parent_id)) {
      const siblings = childMap.get(item.parent_id) ?? []
      siblings.push(item)
      childMap.set(item.parent_id, siblings)
    } else {
      root = item
    }
  }
  if (!root) return []

  const baseItem = drillId != null ? byId.get(drillId) : undefined
  const baseAmount = Number((baseItem ?? root).amount)
  if (baseAmount <= EPSILON) return []

  const rows: DetailRow[] = []
  const walk = (item: CashFlowItemDto, depth: number) => {
    const amount = Number(item.amount)
    rows.push({ accountId: item.account_id, name: item.name, amount, ratio: amount / baseAmount, depth })
    const children = (childMap.get(item.account_id) ?? [])
      .slice()
      .sort((a, b) => Number(b.amount) - Number(a.amount))
    for (const child of children) {
      walk(child, depth + 1)
    }
  }

  if (baseItem) {
    walk(baseItem, 0)
  } else {
    const top = (childMap.get(root.account_id) ?? [])
      .slice()
      .sort((a, b) => Number(b.amount) - Number(a.amount))
    for (const item of top) {
      walk(item, 0)
    }
  }
  return rows
}
