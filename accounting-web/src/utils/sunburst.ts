import type { CategoryAmountItemDto } from '../types/api'

export interface SunburstNode {
  name: string
  value?: number
  /** 中间节点的真实汇总（含被过滤子节点）；tooltip 展示用，不参与角度计算 */
  total?: number
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

function segments(path: string): string[] {
  return path.split(':')
}

function directChildren(parentPath: string, totals: Map<string, number>): string[] {
  const depth = segments(parentPath).length
  const prefix = parentPath + ':'
  const children: string[] = []
  for (const path of totals.keys()) {
    if (path.startsWith(prefix) && segments(path).length === depth + 1) {
      children.push(path)
    }
  }
  return children.sort()
}

function buildNode(path: string, totals: Map<string, number>): SunburstNode {
  const name = segments(path).at(-1) ?? path
  const total = totals.get(path) ?? 0
  const childPaths = directChildren(path, totals)
  if (childPaths.length === 0) {
    return { name, value: total }
  }
  const children = childPaths.map(cp => buildNode(cp, totals))
  const childrenSum = childPaths.reduce((sum, cp) => sum + (totals.get(cp) ?? 0), 0)
  const own = total - childrenSum
  // 直接记入中间层的分录以同名伪子节点承载，保证父环 = 子环之和
  if (own > EPSILON) {
    children.push({ name, value: own })
  }
  const visible = filterVisible(children, total)
  if (visible.length === 0) {
    return { name, value: total }
  }
  // 中间节点不带 value：ECharts 按可见子节点求和分配角度，被过滤项不占比例；
  // 真实总额存于 total 字段供 tooltip 展示
  return { name, total, children: visible }
}

/**
 * 从扁平的「路径 → 各层汇总」列表构建太阳图树。
 * 返回根节点的子节点列表与根总额；中间层直接分录生成同名伪子节点；
 * 每一级仅保留超过该级总额 1% 的子节点（含伪子节点），被过滤项不参与比例，
 * 可见子节点归一化填满整环；中间节点真实总额存于 total 字段供 tooltip 展示。
 */
export function buildSunburstTree(items: CategoryAmountItemDto[]): SunburstTree {
  if (items.length === 0) return { children: [], total: 0 }

  const totals = new Map<string, number>()
  for (const item of items) {
    totals.set(item.account, Number(item.amount))
  }

  // 根 = 层级最浅的路径（通常为单段的 Income / Expenses 根）
  let rootPath = ''
  for (const path of totals.keys()) {
    if (rootPath === '' || segments(path).length < segments(rootPath).length) {
      rootPath = path
    }
  }

  const total = totals.get(rootPath) ?? 0
  const childPaths = directChildren(rootPath, totals)
  const children = childPaths.map(cp => buildNode(cp, totals))
  const childrenSum = childPaths.reduce((sum, cp) => sum + (totals.get(cp) ?? 0), 0)
  const own = total - childrenSum
  if (own > EPSILON) {
    children.push({ name: segments(rootPath).at(-1) ?? rootPath, value: own })
  }

  return { children: filterVisible(children, total), total }
}
