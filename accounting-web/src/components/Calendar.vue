<template>
  <div class="calendar">
    <div class="calendar-header">
      <span class="arrow" @click="prevMonth">‹</span>
      <span class="title">{{ currentYearMonth }}</span>
      <span class="arrow" @click="nextMonth">›</span>
    </div>
    <div class="calendar-weekdays">
      <div v-for="d in weekdays" :key="d" class="weekday">{{ d }}</div>
    </div>
    <div class="calendar-days">
      <div
        v-for="day in days"
        :key="day.date"
        class="day-cell"
        :class="{
          'other-month': !day.isCurrentMonth,
          'today': day.isToday,
          'selected': isSelected(day.date),
          'in-range': isInRange(day.date),
        }"
        @click="handleClick(day.date)"
      >
        <div class="day-number">{{ day.dayOfMonth }}</div>
        <div v-if="data && data[day.date]" class="day-stats">
          <div v-if="data[day.date].income" class="income">
            +{{ data[day.date].income }}
          </div>
          <div v-if="data[day.date].expense" class="expense">
            -{{ data[day.date].expense }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import dayjs from 'dayjs'

defineProps<{
  data?: Record<string, { income: number; expense: number }>
}>()

const emit = defineEmits<{
  (e: 'select', date: string): void
  (e: 'selectRange', from: string, to: string): void
}>()

const currentMonth = ref(dayjs().startOf('month'))
const selectedDate = ref<string | null>(null)
const rangeStart = ref<string | null>(null)
const rangeEnd = ref<string | null>(null)

const weekdays = ['日', '一', '二', '三', '四', '五', '六']

const currentYearMonth = computed(() => currentMonth.value.format('YYYY年MM月'))

interface DayCell {
  date: string
  dayOfMonth: number
  isCurrentMonth: boolean
  isToday: boolean
}

const days = computed<DayCell[]>(() => {
  const start = currentMonth.value.startOf('month')
  const end = currentMonth.value.endOf('month')
  const startDayOfWeek = start.day()
  const daysInMonth = end.date()

  const result: DayCell[] = []

  // Previous month padding
  const prevMonthEnd = start.subtract(1, 'day')
  for (let i = startDayOfWeek - 1; i >= 0; i--) {
    const d = prevMonthEnd.subtract(i, 'day')
    result.push({
      date: d.format('YYYY-MM-DD'),
      dayOfMonth: d.date(),
      isCurrentMonth: false,
      isToday: false,
    })
  }

  // Current month
  const today = dayjs().format('YYYY-MM-DD')
  for (let i = 1; i <= daysInMonth; i++) {
    const d = start.date(i)
    const dateStr = d.format('YYYY-MM-DD')
    result.push({
      date: dateStr,
      dayOfMonth: i,
      isCurrentMonth: true,
      isToday: dateStr === today,
    })
  }

  // Next month padding to fill 6 rows (42 cells)
  const remaining = 42 - result.length
  for (let i = 1; i <= remaining; i++) {
    const d = end.add(i, 'day')
    result.push({
      date: d.format('YYYY-MM-DD'),
      dayOfMonth: d.date(),
      isCurrentMonth: false,
      isToday: false,
    })
  }

  return result
})

function prevMonth() {
  currentMonth.value = currentMonth.value.subtract(1, 'month')
}

function nextMonth() {
  currentMonth.value = currentMonth.value.add(1, 'month')
}

function isSelected(date: string): boolean {
  if (selectedDate.value) return selectedDate.value === date
  if (rangeStart.value && !rangeEnd.value) return rangeStart.value === date
  return false
}

function isInRange(date: string): boolean {
  if (!rangeStart.value || !rangeEnd.value) return false
  const d = dayjs(date)
  return (
    d.isAfter(dayjs(rangeStart.value).subtract(1, 'day')) &&
    d.isBefore(dayjs(rangeEnd.value).add(1, 'day'))
  )
}

function handleClick(date: string) {
  if (!rangeStart.value) {
    rangeStart.value = date
    selectedDate.value = date
    emit('select', date)
  } else if (!rangeEnd.value) {
    if (rangeStart.value === date) {
      emit('select', date)
    } else {
      const from = dayjs(rangeStart.value).isBefore(dayjs(date))
        ? rangeStart.value
        : date
      const to = dayjs(rangeStart.value).isBefore(dayjs(date))
        ? date
        : rangeStart.value
      rangeStart.value = from
      rangeEnd.value = to
      selectedDate.value = null
      emit('selectRange', from, to)
    }
  } else {
    rangeStart.value = date
    rangeEnd.value = null
    selectedDate.value = date
    emit('select', date)
  }
}
</script>

<style scoped>
.calendar {
  background: #fff;
  border-radius: 8px;
  padding: 16px;
  user-select: none;
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  font-size: 16px;
  font-weight: bold;
}

.arrow {
  cursor: pointer;
  padding: 0 12px;
  font-size: 20px;
  color: #666;
}

.calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  margin-bottom: 8px;
}

.weekday {
  text-align: center;
  color: #999;
  font-size: 12px;
}

.calendar-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}

.day-cell {
  min-height: 64px;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.day-cell:hover {
  background: #f5f5f5;
}

.day-cell.other-month {
  color: #ccc;
}

.day-cell.today .day-number {
  color: #1890ff;
  font-weight: bold;
}

.day-cell.selected {
  background: #e6f7ff;
  border: 1px solid #1890ff;
}

.day-cell.in-range {
  background: #e6f7ff;
}

.day-number {
  font-size: 14px;
  margin-bottom: 2px;
}

.day-stats {
  font-size: 10px;
  line-height: 1.2;
}

.income {
  color: #52c41a;
}

.expense {
  color: #f5222d;
}
</style>
