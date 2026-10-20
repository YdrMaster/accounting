import Decimal from 'decimal.js'
import type { PostingDto } from '../types/api'
import { toCents } from './money'

/** 千分位格式化，保留两位小数 */
export function formatAmount(amt: Decimal): string {
  const fixed = amt.toFixed(2)
  const [intPart, decPart] = fixed.split('.')
  const sign = intPart.startsWith('-') ? '-' : ''
  const abs = intPart.replace('-', '')
  const formatted = abs.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${formatted}.${decPart}`
}

/**
 * 资产类分录取和 —— 非转账交易金额口径。
 * 严格等于资产类分录金额之和，不含任何回退。该回退曾用于兼容 is_transfer
 * 标识出现之前的转账（转账资产分录净额恒为零），现由 DTO 的 is_transfer
 * 与转账渲染分支接管，无需在金额计算里兜底。
 */
export function sumAssetAmount(
  postings: ReadonlyArray<Pick<PostingDto, 'account_type' | 'amount'>>
): Decimal {
  let sum = new Decimal(0)
  for (const p of postings) {
    if (p.account_type === 'asset') sum = sum.plus(new Decimal(p.amount))
  }
  return sum
}

/**
 * 日历格子金额缩写：适应格子宽度，按 locale 选择单位。
 * - 中文：绝对值超过 99999 时以"万"为单位
 * - 英文：绝对值超过 9999 时以"k"为单位
 * 缩写保留 1 位小数并去掉末尾的 .0；未达阈值原样返回后端字符串（scale 可变，
 * 不经 fromCents 还原以免补零改变展示口径）。
 * 阈值与缩写换算在整数分域判定，避免浮点误差影响边界。
 */
export function formatCalendarAmount(value: string, locale: string): string {
  const cents = toCents(value)
  const abs = Math.abs(cents)

  if (locale.startsWith('zh')) {
    if (abs > 99999 * 100) return `${trimZero(cents / 1_000_000)}万`
  } else if (abs > 9999 * 100) {
    return `${trimZero(cents / 100_000)}k`
  }
  return value
}

function trimZero(n: number): string {
  const s = n.toFixed(1)
  return s.endsWith('.0') ? s.slice(0, -2) : s
}
