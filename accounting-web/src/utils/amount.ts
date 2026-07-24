import Decimal from 'decimal.js'

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
 * 日历格子金额缩写：适应格子宽度，按 locale 选择单位。
 * - 中文：绝对值超过 99999 时以"万"为单位
 * - 英文：绝对值超过 9999 时以"k"为单位
 * 缩写保留 1 位小数并去掉末尾的 .0；未达阈值原样显示。
 */
export function formatCalendarAmount(value: string, locale: string): string {
  const num = Number(value)
  if (!Number.isFinite(num)) return value
  const abs = Math.abs(num)

  if (locale.startsWith('zh')) {
    if (abs > 99999) return `${trimZero(num / 10000)}万`
  } else if (abs > 9999) {
    return `${trimZero(num / 1000)}k`
  }
  return value
}

function trimZero(n: number): string {
  const s = n.toFixed(1)
  return s.endsWith('.0') ? s.slice(0, -2) : s
}
