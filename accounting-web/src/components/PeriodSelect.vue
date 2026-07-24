<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ChartPeriod } from '../types/api'

const model = defineModel<ChartPeriod>({ required: true })
const { t } = useI18n()

const options: ChartPeriod[] = ['weekly', 'monthly', 'yearly']
</script>

<template>
  <select v-model="model" class="period-select">
    <option v-for="p in options" :key="p" :value="p">
      {{ t(`assets.period.${p}`) }}
    </option>
  </select>
</template>

<style scoped>
.period-select {
  background: var(--card-bg-alt);
  color: var(--text-heading);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.875rem;
}
</style>
