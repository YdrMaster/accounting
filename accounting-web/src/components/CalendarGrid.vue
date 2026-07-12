<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{
  transactionDates?: Set<string>
  selectedDate?: string
}>()

const emit = defineEmits<{
  (e: 'selectDate', date: string): void
  (e: 'update:selectedDate', date: string): void
}>()

const currentDate = ref(new Date())

const currentYear = computed(() => currentDate.value.getFullYear())
const currentMonth = computed(() => currentDate.value.getMonth() + 1)

const monthLabel = computed(() => {
  return `${currentYear.value}年${currentMonth.value}月`
})

const weekdays = ['一', '二', '三', '四', '五', '六', '日']

interface CalendarDay {
  date: string
  day: number
  isCurrentMonth: boolean
  isToday: boolean
  hasTransaction: boolean
  isSelected: boolean
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
      hasTransaction: props.transactionDates?.has(dateStr) ?? false,
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
      hasTransaction: props.transactionDates?.has(dateStr) ?? false,
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
        hasTransaction: props.transactionDates?.has(dateStr) ?? false,
        isSelected: props.selectedDate === dateStr,
      })
    }
  }

  return days
})

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
          'has-transaction': day.hasTransaction,
          selected: day.isSelected,
        }"
        @click="onDayClick(day)"
      >
        <span class="day-number">{{ day.day }}</span>
        <span v-if="day.hasTransaction" class="transaction-dot"></span>
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
  position: relative;
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: background 0.15s;
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

.transaction-dot {
  position: absolute;
  bottom: 0.25rem;
  width: 0.25rem;
  height: 0.25rem;
  border-radius: 50%;
  background: #4ade80;
}

.day-cell.today .transaction-dot {
  background: var(--accent, #646cff);
}

.day-cell.selected .transaction-dot {
  background: #fff;
}
</style>
