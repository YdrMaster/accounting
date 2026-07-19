<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { DailySummaryDto } from '../types/api'
import { formatCalendarAmount } from '../utils/amount'

const { t, tm, locale } = useI18n()

const props = defineProps<{
  dailyStats?: Record<string, DailySummaryDto>
  selectedDate?: string
}>()

const emit = defineEmits<{
  (e: 'selectDate', date: string): void
  (e: 'update:selectedDate', date: string): void
  (e: 'visibleRangeChange', from: string, to: string): void
}>()

const currentDate = ref(new Date())

const currentYear = computed(() => currentDate.value.getFullYear())
const currentMonth = computed(() => currentDate.value.getMonth() + 1)

const monthLabel = computed(() => {
  return t('calendarGrid.monthLabel', { year: currentYear.value, month: currentMonth.value })
})

const weekdays = computed(() => tm('calendarGrid.weekdays') as string[])

interface CalendarDay {
  date: string
  day: number
  isCurrentMonth: boolean
  isToday: boolean
  expense: string | null
  income: string | null
  isSelected: boolean
}

function dayAmounts(dateStr: string): { expense: string | null; income: string | null } {
  const stat = props.dailyStats?.[dateStr]
  if (!stat) return { expense: null, income: null }
  return {
    expense: Number(stat.expense) !== 0 ? formatCalendarAmount(stat.expense, locale.value) : null,
    income: Number(stat.income) !== 0 ? formatCalendarAmount(stat.income, locale.value) : null,
  }
}

const calendarDays = computed<CalendarDay[]>(() => {
  const year = currentYear.value
  const month = currentMonth.value - 1

  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)

  const startDayOfWeek = firstDay.getDay() === 0 ? 6 : firstDay.getDay() - 1
  const daysInMonth = lastDay.getDate()

  const days: CalendarDay[] = []

  const prevMonthLastDay = new Date(year, month, 0).getDate()
  for (let i = startDayOfWeek - 1; i >= 0; i--) {
    const day = prevMonthLastDay - i
    const dateStr = formatDateStr(year, month - 1, day)
    days.push({
      date: dateStr,
      day,
      isCurrentMonth: false,
      isToday: false,
      ...dayAmounts(dateStr),
      isSelected: props.selectedDate === dateStr,
    })
  }

  const today = new Date()
  const todayStr = formatDateStr(today.getFullYear(), today.getMonth(), today.getDate())

  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = formatDateStr(year, month, day)
    days.push({
      date: dateStr,
      day,
      isCurrentMonth: true,
      isToday: dateStr === todayStr,
      ...dayAmounts(dateStr),
      isSelected: props.selectedDate === dateStr,
    })
  }

  const remaining = 7 - (days.length % 7)
  if (remaining < 7) {
    for (let day = 1; day <= remaining; day++) {
      const dateStr = formatDateStr(year, month + 1, day)
      days.push({
        date: dateStr,
        day,
        isCurrentMonth: false,
        isToday: false,
        ...dayAmounts(dateStr),
        isSelected: props.selectedDate === dateStr,
      })
    }
  }

  return days
})

// 网格可见范围（含前后月补位天），挂载与翻月时通知父组件加载统计
const visibleRange = computed(() => {
  const days = calendarDays.value
  if (days.length === 0) return null
  return { from: days[0].date, to: days[days.length - 1].date }
})

watch(
  visibleRange,
  range => {
    if (range) emit('visibleRangeChange', range.from, range.to)
  },
  { immediate: true }
)

function formatDateStr(year: number, month: number, day: number): string {
  const m = String(month + 1).padStart(2, '0')
  const d = String(day).padStart(2, '0')
  return `${year}-${m}-${d}`
}

function prevMonth() {
  const d = new Date(currentDate.value)
  d.setMonth(d.getMonth() - 1)
  currentDate.value = d
}

function nextMonth() {
  const d = new Date(currentDate.value)
  d.setMonth(d.getMonth() + 1)
  currentDate.value = d
}

function onDayClick(day: CalendarDay) {
  emit('update:selectedDate', day.date)
  emit('selectDate', day.date)
}
</script>

<template>
  <div class="calendar-grid">
    <div class="calendar-header">
      <button class="nav-btn" @click="prevMonth">←</button>
      <span class="month-label">{{ monthLabel }}</span>
      <button class="nav-btn" @click="nextMonth">→</button>
    </div>

    <div class="weekday-row">
      <div v-for="wd in weekdays" :key="wd" class="weekday-cell">{{ wd }}</div>
    </div>

    <div class="days-grid">
      <div
        v-for="(day, idx) in calendarDays"
        :key="idx"
        class="day-cell"
        :class="{
          'other-month': !day.isCurrentMonth,
          today: day.isToday,
          selected: day.isSelected,
        }"
        @click="onDayClick(day)"
      >
        <span class="day-number">{{ day.day }}</span>
        <span v-if="day.expense" class="day-amount expense">{{ day.expense }}</span>
        <span v-if="day.income" class="day-amount income">{{ day.income }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-grid {
  background: var(--card-bg-alt, #252525);
  border-radius: 0.75rem;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0.25rem;
}

.nav-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1rem;
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}

.nav-btn:hover {
  background: var(--card-bg, #1e1e1e);
  color: var(--text-heading);
}

.month-label {
  font-weight: 600;
  color: var(--text-heading);
  font-size: 0.9375rem;
}

.weekday-row {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25rem;
}

.weekday-cell {
  text-align: center;
  color: var(--text-muted);
  font-size: 0.75rem;
  padding: 0.25rem 0;
}

.days-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.25rem;
}

.day-cell {
  min-height: 3.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.25rem 0.125rem;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: background 0.15s;
  overflow: hidden;
}

.day-cell:hover {
  background: var(--card-bg, #1e1e1e);
}

.day-cell.other-month {
  opacity: 0.3;
}

.day-cell.today {
  border: 2px solid var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.day-cell.today:hover {
  background: var(--card-bg, #1e1e1e);
}

.day-cell.selected {
  background: var(--accent, #646cff);
  color: #fff;
}

.day-cell.selected:hover {
  background: var(--accent, #646cff);
  opacity: 0.9;
}

.day-number {
  font-size: 0.8125rem;
  color: var(--text-heading);
}

.day-cell.today .day-number {
  color: var(--accent, #646cff);
  font-weight: 600;
}

.day-cell.selected .day-number {
  color: #fff;
}

.day-amount {
  font-size: 0.6875rem;
  line-height: 1.2;
  white-space: nowrap;
}

.day-amount.expense {
  color: #e74c3c;
}

.day-amount.income {
  color: #4ade80;
}
</style>
