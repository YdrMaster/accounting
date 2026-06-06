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
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="postings.length === 0" class="empty">暂无分录</div>
      <div v-else class="postings">
        <div v-for="p in postings" :key="p.id" class="posting-row">
          <span class="posting-account">{{ p.account }}</span>
          <span class="posting-commodity">{{ p.commodity }}</span>
          <span class="posting-amount" :class="{ positive: Number(p.amount) > 0, negative: Number(p.amount) < 0 }">
            {{ Number(p.amount) > 0 ? '+' : '' }}{{ p.amount }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import api from '@/api/client'
import type { Transaction } from '@/stores/transaction'

interface Posting {
  id: number
  account: string
  commodity: string
  amount: string
}

const props = defineProps<{
  tx: Transaction
}>()

const expanded = ref(false)
const loading = ref(false)
const postings = ref<Posting[]>([])

const formattedDate = computed(() => {
  const d = new Date(props.tx.date_time)
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
})

async function toggleExpand() {
  expanded.value = !expanded.value
  if (expanded.value && postings.value.length === 0) {
    loading.value = true
    try {
      const res = await api.get<{
        id: number
        date_time: string
        description: string
        member_id?: number
        is_template: boolean
        postings: Posting[]
      }>(`/transactions/${props.tx.id}`)
      postings.value = res.data.postings
    } catch (e) {
      console.error('获取分录失败', e)
    } finally {
      loading.value = false
    }
  }
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

.loading,
.empty {
  color: #999;
  font-size: 13px;
  text-align: center;
  padding: 8px 0;
}

.postings {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.posting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  padding: 6px 8px;
  background: #f8f8f8;
  border-radius: 4px;
}

.posting-account {
  flex: 1;
  color: #333;
}

.posting-commodity {
  width: 50px;
  color: #666;
  text-align: center;
}

.posting-amount {
  width: 80px;
  text-align: right;
  font-weight: 500;
}

.posting-amount.positive {
  color: #52c41a;
}

.posting-amount.negative {
  color: #f5222d;
}
</style>
