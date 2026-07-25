import type { CashFlowItemDto } from '../types/api'

export interface SunburstNode {
  name: string
  value?: number
  /** 中间节点的真实汇总（含被过滤子节点）；tooltip 展示用，不参与角度计算 */
  total?: number
  /** 账户 id，钻入事件载荷；伪子节点无 id（不响应钻入） */
  accountId?: number
  /** 伪子节点设为 false，点击既不下钻也不抛出事件 */
  nodeClick?: false
  itemStyle?: { color: string }
  /** 里圈节点的标签描边覆盖（白色描边保证在扇区底色上可辨） */
  label?: { textBorderColor: string; textBorderWidth: number }
  children?: SunburstNode[]
}

export interface SunburstTree {
  children: SunburstNode[]
  total: number
}

const EPSILON = 1e-6

// 仅显示超过当前级别总额 1% 的节点，避免渲染过窄的扇区；
// 被过滤项不参与比例计算，可见子节点重新归一化填满整环
const MIN_VISIBLE_RATIO = 0.01

function filterVisible(children: SunburstNode[], parentTotal: number): SunburstNode[] {
  if (parentTotal <= EPSILON) return children
  return children.filter(c => (c.value ?? c.total ?? 0) > parentTotal * MIN_VISIBLE_RATIO)
}

/**
 * 从扁平的「账户 → 各层汇总」列表按 account_id / parent_id 构建太阳图树。
 * 返回根节点的子节点列表与根总额；中间层直接分录生成同名伪子节点（无 accountId，
 * 不响应钻入）；每一级仅保留超过该级总额 1% 的子节点（含伪子节点），被过滤项不
 * 参与比例，可见子节点归一化填满整环；中间节点真实总额存于 total 字段供 tooltip
 * 展示；兄弟节点按金额降序排列。
 */
export function buildSunburstTree(items: CashFlowItemDto[]): SunburstTree {
  if (items.length === 0) return { children: [], total: 0 }

  const byId = new Map(items.map(i => [i.account_id, i]))
  const childMap = new Map<number, CashFlowItemDto[]>()
  const roots: CashFlowItemDto[] = []
  for (const item of items) {
    if (item.parent_id != null && byId.has(item.parent_id)) {
      const siblings = childMap.get(item.parent_id) ?? []
      siblings.push(item)
      childMap.set(item.parent_id, siblings)
    } else {
      roots.push(item)
    }
  }

  const byAmountDesc = (a: SunburstNode, b: SunburstNode) =>
    (b.value ?? b.total ?? 0) - (a.value ?? a.total ?? 0)

  function buildNode(item: CashFlowItemDto): SunburstNode {
    const total = Number(item.amount)
    const childItems = childMap.get(item.account_id) ?? []
    if (childItems.length === 0) {
      return { name: item.name, value: total, accountId: item.account_id }
    }
    const children = childItems.map(buildNode)
    const childrenSum = childItems.reduce((sum, c) => sum + Number(c.amount), 0)
    const own = total - childrenSum
    // 直接记入中间层的分录以同名伪子节点承载，保证父环 = 子环之和；伪子节点不响应钻入
    if (own > EPSILON) {
      children.push({ name: item.name, value: own, nodeClick: false })
    }
    children.sort(byAmountDesc)
    const visible = filterVisible(children, total)
    if (visible.length === 0) {
      return { name: item.name, value: total, accountId: item.account_id }
    }
    // 中间节点不带 value：ECharts 按可见子节点求和分配角度，被过滤项不占比例；
    // 真实总额存于 total 字段供 tooltip 展示
    return { name: item.name, total, accountId: item.account_id, children: visible }
  }

  const total = roots.reduce((sum, r) => sum + Number(r.amount), 0)
  const children: SunburstNode[] = []
  for (const root of roots) {
    const childItems = childMap.get(root.account_id) ?? []
    const nodes = childItems.map(buildNode)
    const childrenSum = childItems.reduce((sum, c) => sum + Number(c.amount), 0)
    const own = Number(root.amount) - childrenSum
    if (own > EPSILON) {
      nodes.push({ name: root.name, value: own, nodeClick: false })
    }
    children.push(...nodes)
  }
  children.sort(byAmountDesc)

  return { children: filterVisible(children, total), total }
}
