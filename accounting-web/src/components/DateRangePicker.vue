<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatDate } from '../utils/date'

const props = defineProps<{
  from?: string
  to?: string
}>()

const emit = defineEmits<{
  'update:from': [val: string | undefined]
  'update:to': [val: string | undefined]
}>()

const { t, tm } = useI18n()
const open = ref(false)
const viewYear = ref(new Date().getFullYear())
const viewMonth = ref(new Date().getMonth())
const pickingEnd = ref(false)
const hoverDate = ref<string | null>(null)

const weekdays = computed(() => tm('txList.weekdays') as string[])

const monthLabel = computed(() => {
  const y = viewYear.value
  const m = viewMonth.value + 1
  return t('transactions.monthLabel', { year: y, month: String(m).padStart(2, '0') })
})

interface DayCell {
  date: string
  day: number
  inMonth: boolean
  isToday: boolean
  isStart: boolean
  isEnd: boolean
  inRange: boolean
  inHoverRange: boolean
}

const days = computed<DayCell[]>(() => {
  const y = viewYear.value
  const m = viewMonth.value
  const firstDay = new Date(y, m, 1)
  const startOffset = firstDay.getDay()
  const today = formatDate(new Date())

  const cells: DayCell[] = []
  const gridStart = new Date(y, m, 1 - startOffset)

  for (let i = 0; i < 42; i++) {
    const d = new Date(gridStart)
    d.setDate(gridStart.getDate() + i)
    const dateStr = formatDate(d)
    const inMonth = d.getMonth() === m
    const isToday = dateStr === today
    const isStart = dateStr === props.from
    const isEnd = dateStr === props.to
    const inRange = !!(props.from && props.to && dateStr > props.from && dateStr < props.to)
    const inHoverRange = !!(
      props.from &&
      !props.to &&
      hoverDate.value &&
      pickingEnd.value &&
      dateStr > props.from &&
      dateStr <= hoverDate.value
    )
    cells.push({ date: dateStr, day: d.getDate(), inMonth, isToday, isStart, isEnd, inRange, inHoverRange })
  }
  return cells
})

function prevMonth() {
  if (viewMonth.value === 0) {
    viewMonth.value = 11
    viewYear.value--
  } else {
    viewMonth.value--
  }
}

function nextMonth() {
  if (viewMonth.value === 11) {
    viewMonth.value = 0
    viewYear.value++
  } else {
    viewMonth.value++
  }
}

function onDayClick(cell: DayCell) {
  if (!cell.inMonth) return
  if (!pickingEnd.value) {
    emit('update:from', cell.date)
    emit('update:to', undefined)
    pickingEnd.value = true
  } else {
    if (cell.date < (props.from ?? '')) {
      emit('update:from', cell.date)
      emit('update:to', props.from)
    } else {
      emit('update:to', cell.date)
    }
    pickingEnd.value = false
    open.value = false
  }
}

function toggle() {
  open.value = !open.value
  if (open.value) {
    pickingEnd.value = false
    const ref = props.to || props.from
    if (ref) {
      const [y, m] = ref.split('-').map(Number)
      viewYear.value = y
      viewMonth.value = m - 1
    }
  }
}

function clear() {
  emit('update:from', undefined)
  emit('update:to', undefined)
  pickingEnd.value = false
}

const displayFrom = computed(() => props.from || '—')
const displayTo = computed(() => props.to || '—')
</script>

<template>
  <div class="date-range-picker">
    <button class="trigger" @click="toggle" :class="{ active: open }">
      <span class="trigger-from">{{ displayFrom }}</span>
      <span class="trigger-sep">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
      </span>
      <span class="trigger-to">{{ displayTo }}</span>
      <svg class="trigger-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
    </button>

    <Transition name="pop">
      <div v-if="open" class="popover" @click.stop>
        <div class="cal-header">
          <button class="nav-btn" @click="prevMonth">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
          </button>
          <span class="cal-title">{{ monthLabel }}</span>
          <button class="nav-btn" @click="nextMonth">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
          </button>
        </div>

        <div class="cal-weekdays">
          <span v-for="(wd, i) in weekdays" :key="i" class="wd">{{ wd }}</span>
        </div>

        <div class="cal-grid">
          <button
            v-for="cell in days"
            :key="cell.date"
            class="cal-day"
            :class="{
              'out': !cell.inMonth,
              'today': cell.isToday,
              'start': cell.isStart,
              'end': cell.isEnd,
              'in-range': cell.inRange || cell.inHoverRange,
            }"
            @click="onDayClick(cell)"
            @mouseenter="hoverDate = cell.date"
            @mouseleave="hoverDate = null"
          >
            {{ cell.day }}
          </button>
        </div>

        <div class="cal-footer">
          <button class="cal-clear" @click="clear">{{ t('common.none') }}</button>
          <button class="cal-close" @click="open = false">{{ t('txFilter.done') }}</button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.date-range-picker {
  position: relative;
}

.trigger {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--card-bg-alt, #252525);
  color: var(--text-body);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.trigger:hover,
.trigger.active {
  border-color: var(--accent, #646cff);
  box-shadow: 0 0 0 2px rgba(100, 108, 255, 0.15);
}

.trigger-from,
.trigger-to {
  flex: 1;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.trigger-sep {
  color: var(--text-muted);
  display: flex;
  align-items: center;
}

.trigger-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

/* Popover */
.popover {
  position: absolute;
  top: calc(100% + 0.5rem);
  left: 0;
  right: 0;
  z-index: 10;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  padding: 0.75rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
}

.pop-enter-active {
  animation: pop-in 0.2s ease-out;
}
.pop-leave-active {
  animation: pop-in 0.15s ease-in reverse;
}
@keyframes pop-in {
  from { opacity: 0; transform: translateY(-6px) scale(0.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.cal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.cal-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-heading);
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.nav-btn:hover {
  background: var(--border);
  color: var(--text-heading);
}

.cal-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  margin-bottom: 0.25rem;
}

.wd {
  text-align: center;
  font-size: 0.6875rem;
  color: var(--text-muted);
  padding: 0.25rem 0;
}

.cal-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.cal-day {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 2rem;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-body);
  font-size: 0.75rem;
  cursor: pointer;
  transition: background 0.12s, color 0.12s, transform 0.12s;
}

.cal-day:hover:not(.out) {
  background: rgba(100, 108, 255, 0.12);
  transform: scale(1.1);
}

.cal-day.out {
  color: var(--text-muted);
  opacity: 0.35;
  cursor: default;
}

.cal-day.today {
  font-weight: 700;
  color: var(--accent, #646cff);
}

.cal-day.today::after {
  content: '';
  position: absolute;
  bottom: 3px;
  left: 50%;
  transform: translateX(-50%);
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent, #646cff);
}

.cal-day.start,
.cal-day.end {
  background: var(--accent, #646cff);
  color: #fff;
  font-weight: 600;
  border-radius: 0.5rem;
}

.cal-day.start:hover,
.cal-day.end:hover {
  background: var(--accent, #646cff);
  filter: brightness(1.15);
}

.cal-day.in-range {
  background: rgba(100, 108, 255, 0.15);
  border-radius: 0;
  color: var(--text-heading);
}

.cal-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--border);
}

.cal-clear,
.cal-close {
  padding: 0.3rem 0.75rem;
  border-radius: 0.375rem;
  font-size: 0.75rem;
  cursor: pointer;
  border: none;
  transition: background 0.15s;
}

.cal-clear {
  background: transparent;
  color: var(--text-muted);
}

.cal-clear:hover {
  color: var(--text-body);
  background: var(--border);
}

.cal-close {
  background: var(--accent, #646cff);
  color: #fff;
}

.cal-close:hover {
  filter: brightness(1.1);
}
</style>
