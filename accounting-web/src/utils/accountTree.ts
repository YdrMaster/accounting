/**
 * 账户树工具：将指定账户展开为「自身 + 全部后代」ID 列表。
 * 用于现金流量表点击跳转交易筛选时，与报表的聚合口径（父含子）对齐。
 */

export interface AccountTreeNode {
  id: number
  parent_id: number | null
}

/**
 * 返回 rootId 自身及其全部后代账户 ID。
 * 防御性兜底：accounts 中找不到 rootId 时至少返回 [rootId]（退化为精确匹配）。
 */
export function expandSubtree(accounts: AccountTreeNode[], rootId: number): number[] {
  const childMap = new Map<number, number[]>()
  let found = false
  for (const acc of accounts) {
    if (acc.id === rootId) found = true
    if (acc.parent_id != null) {
      const siblings = childMap.get(acc.parent_id) ?? []
      siblings.push(acc.id)
      childMap.set(acc.parent_id, siblings)
    }
  }

  const result: number[] = []
  const walk = (id: number) => {
    result.push(id)
    for (const child of childMap.get(id) ?? []) {
      walk(child)
    }
  }
  walk(rootId)

  if (!found && !result.includes(rootId)) result.push(rootId)
  return result
}
