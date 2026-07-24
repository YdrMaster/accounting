export function formatDate(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

export function todayStr(): string {
  return formatDate(new Date())
}

export function dateOf(tx: { date_time: string }): string {
  return tx.date_time.slice(0, 10)
}

export function monthOf(tx: { date_time: string }): string {
  return tx.date_time.slice(0, 7)
}

export type ShiftPeriod = 'weekly' | 'monthly' | 'yearly'

function parseDate(dateStr: string): Date {
  const [y, m, d] = dateStr.split('-').map(Number)
  return new Date(y, m - 1, d)
}

/** 按周期粒度偏移日期：weekly ±7 天，monthly ±1 月，yearly ±1 年 */
export function shiftDate(dateStr: string, period: ShiftPeriod, delta: number): string {
  const date = parseDate(dateStr)
  switch (period) {
    case 'weekly':
      date.setDate(date.getDate() + delta * 7)
      break
    case 'monthly':
      date.setMonth(date.getMonth() + delta)
      break
    case 'yearly':
      date.setFullYear(date.getFullYear() + delta)
      break
  }
  return formatDate(date)
}

/** 返回该日期所在周的周一（ISO 日期字符串） */
export function startOfWeekMonday(dateStr: string): string {
  const date = parseDate(dateStr)
  const daysFromMonday = (date.getDay() + 6) % 7
  date.setDate(date.getDate() - daysFromMonday)
  return formatDate(date)
}
