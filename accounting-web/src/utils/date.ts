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
