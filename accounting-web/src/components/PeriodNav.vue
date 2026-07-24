<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ChartPeriod } from '../types/api'
import { shiftDate, startOfWeekMonday } from '../utils/date'

const props = defineProps<{ period: ChartPeriod }>()
const date = defineModel<string>({ required: true })
const { locale } = useI18n()

function prev() {
  date.value = shiftDate(date.value, props.period, -1)
}

function next() {
  date.value = shiftDate(date.value, props.period, 1)
}

const label = computed(() => {
  const [y, m, d] = date.value.split('-').map(Number)
  const dt = new Date(y, m - 1, d)
  const lang = locale.value.startsWith('zh') ? 'zh-CN' : 'en'
  if (props.period === 'yearly') return String(y)
  if (props.period === 'monthly') {
    return new Intl.DateTimeFormat(lang, { year: 'numeric', month: 'long' }).format(dt)
  }
  const startStr = startOfWeekMonday(date.value)
  const [sy, sm, sd] = startStr.split('-').map(Number)
  const fmt = new Intl.DateTimeFormat(lang, { month: 'short', day: 'numeric' })
  return `${fmt.format(new Date(sy, sm - 1, sd))} – ${fmt.format(new Date(sy, sm - 1, sd + 6))}`
})
</script>

<template>
  <div class="period-nav">
    <button class="nav-btn" @click="prev">◀</button>
    <span class="nav-label">{{ label }}</span>
    <button class="nav-btn" @click="next">▶</button>
  </div>
</template>

<style scoped>
.period-nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.nav-btn {
  background: var(--card-bg-alt);
  color: var(--text-heading);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  cursor: pointer;
}

.nav-label {
  font-size: 0.875rem;
  color: var(--text-heading);
  min-width: 7rem;
  text-align: center;
}
</style>
