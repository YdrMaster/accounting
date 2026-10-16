/**
 * 金额整型最小单位换算（precision=2 时单位为"分"）。
 * 全程字符串/整数运算，不经浮点：后端 Decimal 字符串（scale 可变）先转整数分
 * 参与累加、相减、比较与排序，仅在渲染边界经 fromCents 恢复展示串。
 */

const DECIMAL_PATTERN = /^[+-]?\d+(\.\d+)?$/

/** Decimal 字符串 → 整数最小单位；小数超出 precision 位时按首溢位四舍五入。 */
export function toCents(value: string, precision = 2): number {
  if (!DECIMAL_PATTERN.test(value)) {
    throw new Error(`invalid decimal string: ${JSON.stringify(value)}`)
  }
  const negative = value.startsWith('-')
  const [intPart, decPart = ''] = value.replace(/^[+-]/, '').split('.')
  const kept = decPart.slice(0, precision).padEnd(precision, '0')
  let cents = Number(intPart + kept)
  if (decPart.length > precision && decPart[precision] >= '5') {
    cents += 1
  }
  return negative && cents !== 0 ? -cents : cents
}

/** 整数最小单位 → 定长小数字符串（precision 位小数），纯位拼接不经除法。 */
export function fromCents(cents: number, precision = 2): string {
  const sign = cents < 0 ? '-' : ''
  const digits = String(Math.abs(cents))
  if (precision === 0) {
    return sign + digits
  }
  const padded = digits.padStart(precision + 1, '0')
  return `${sign}${padded.slice(0, -precision)}.${padded.slice(-precision)}`
}
