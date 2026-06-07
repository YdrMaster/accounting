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
        <div class="day-top">
          <span class="day-number">{{ day.dayOfMonth }}</span>
        </div>
        <div class="day-bottom">
          <template v-if="data && data[day.date]">
            <span v-if="data[day.date].expense" class="expense">
              -{{ fmt(data[day.date].expense) }}
            </span>
            <span v-if="data[day.date].income" class="income">
              +{{ fmt(data[day.date].income) }}
            </span>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import dayjs from 'dayjs'

const props = defineProps<{
  data?: Record<string, { income: number; expense: number }>
  rangeMode?: boolean
}>()

const emit = defineEmits<{
  (e: 'select', date: string): void
  (e: 'selectRange', from: string, to: string): void
  (e: 'clear'): void
}>()

const currentMonth = ref(dayjs().startOf('month'))
const selectedDate = ref<string | null>(null)
const rangeStart = ref<string | null>(null)
const rangeEnd = ref<string | null>(null)

watch(() => props.rangeMode, () => {
  selectedDate.value = null
  rangeStart.value = null
  rangeEnd.value = null
  emit('clear')
})

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

function fmt(n: number) {
  return n.toFixed(2)
}

function handleClick(date: string) {
  if (!props.rangeMode) {
    // 单日模式：单击选单日，再次点击取消
    if (selectedDate.value === date) {
      selectedDate.value = null
      rangeStart.value = null
      rangeEnd.value = null
      emit('clear')
    } else {
      selectedDate.value = date
      rangeStart.value = date
      rangeEnd.value = null
      emit('select', date)
    }
    return
  }

  // 范围模式
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
  border-radius: 12px;
  padding: 12px;
  user-select: none;
  container-type: inline-size;
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  font-size: clamp(14px, 3cqi, 18px);
  font-weight: bold;
}

.arrow {
  cursor: pointer;
  padding: 0 clamp(8px, 2cqi, 12px);
  font-size: clamp(16px, 4cqi, 22px);
  color: #666;
}

.calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  margin-bottom: 6px;
}

.weekday {
  text-align: center;
  color: #999;
  font-size: clamp(10px, 2.5cqi, 14px);
}

.calendar-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 8px;
}

.day-cell {
  aspect-ratio: 1.618 / 1;
  min-height: 48px;
  padding: clamp(2px, 0.6cqi, 5px);
  border-radius: 8px;
  background: #f5f5f5;
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  overflow: hidden;
}

.day-cell:hover {
  background: #e8e8e8;
}

.day-cell.other-month {
  background: #fafafa;
  color: #ccc;
}

.day-cell.other-month .income,
.day-cell.other-month .expense {
  color: #ddd;
}

.day-cell.today {
  background: #fff7e6;
}

.day-cell.today .day-number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: clamp(18px, 5.5cqi, 26px);
  height: clamp(18px, 5.5cqi, 26px);
  background: #fa8c16;
  color: #fff;
  border-radius: 50%;
  font-weight: bold;
}

.day-cell.selected {
  background: #bae7ff;
  box-shadow: 0 0 0 2px #1890ff;
}

.day-cell.in-range {
  background: #e6f7ff;
}

.day-top {
  text-align: center;
}

.day-number {
  font-size: clamp(12px, 3.2cqi, 15px);
  font-weight: 600;
  line-height: 1.4;
}

.day-bottom {
  display: flex;
  flex-direction: column;
  gap: 1px;
  align-items: center;
  justify-content: center;
  font-size: clamp(10px, 2.5cqi, 12px);
  line-height: 1.3;
  overflow: hidden;
}

.income {
  color: #52c41a;
  font-weight: 500;
}

.expense {
  color: #f5222d;
  font-weight: 500;
}
</style>
