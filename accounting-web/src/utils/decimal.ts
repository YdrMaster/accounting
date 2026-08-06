/**
 * 后端 Decimal 字符串的显示格式化：最多保留 dp 位小数并去掉尾随零。
 * 例："75.00" → "75"，"33.33333333333333333333333333" → "33.33"。
 * 非数值输入原样返回。
 */
export function formatDecimal(value: string, dp = 2): string {
  const n = Number(value)
  if (!Number.isFinite(n)) return value
  return String(Number(n.toFixed(dp)))
}
