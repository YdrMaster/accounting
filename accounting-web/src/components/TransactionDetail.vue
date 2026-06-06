<template>
  <div class="transaction-item" @click="toggleExpand">
    <div class="transaction-header">
      <div class="transaction-info">
        <span class="transaction-date">{{ formattedDate }}</span>
        <span class="transaction-desc">{{ tx.description }}</span>
      </div>
      <span class="expand-icon">{{ expanded ? '▼' : '▶' }}</span>
    </div>
    <div v-if="expanded" class="transaction-detail">
      <p class="placeholder">分录详情待加载</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Transaction } from '@/stores/transaction'

const props = defineProps<{
  tx: Transaction
}>()

const expanded = ref(false)

const formattedDate = computed(() => {
  const d = new Date(props.tx.date_time)
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
})

function toggleExpand() {
  expanded.value = !expanded.value
}
</script>

<style scoped>
.transaction-item {
  background: #fff;
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 8px;
  cursor: pointer;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.transaction-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.transaction-info {
  display: flex;
  gap: 12px;
  align-items: center;
}

.transaction-date {
  color: #666;
  font-size: 14px;
  white-space: nowrap;
}

.transaction-desc {
  color: #333;
  font-size: 14px;
}

.expand-icon {
  color: #999;
  font-size: 12px;
}

.transaction-detail {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.placeholder {
  color: #999;
  font-size: 13px;
  margin: 0;
}
</style>
