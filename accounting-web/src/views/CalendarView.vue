<script setup lang="ts">
const weekdays = ['一', '二', '三', '四', '五', '六', '日']
const days = Array.from({ length: 28 }, (_, i) => i + 1)
</script>

<template>
  <div class="calendar">
    <div class="month-header">
      <button type="button">‹</button>
      <span>2026-02</span>
      <button type="button">›</button>
    </div>

    <div class="weekdays">
      <span v-for="day in weekdays" :key="day">{{ day }}</span>
    </div>

    <div class="days">
      <div v-for="day in days" :key="day" class="day">
        <span class="date">{{ day }}</span>
        <span class="out">-{{ (day * 13) % 200 }}</span>
        <span class="in">+{{ (day * 7) % 100 }}</span>
      </div>
    </div>

    <div class="summary">
      <span>月收入：¥173,692.17</span>
      <span>月支出：¥34,611.83</span>
      <span>月结余：¥139,080.34</span>
    </div>
  </div>
</template>

<style scoped>
.calendar {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.month-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 1.25rem;
  font-weight: 500;
}

.month-header button {
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 1.25rem;
  cursor: pointer;
}

.weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.5rem;
}

.day {
  aspect-ratio: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--card-bg-alt);
  border-radius: 0.5rem;
  font-size: 0.75rem;
}

.day .date {
  font-size: 1rem;
  font-weight: 500;
  color: var(--text-heading);
}

.day .out {
  color: #ff7b7b;
}

.day .in {
  color: #7bffb3;
}

.summary {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}
</style>
