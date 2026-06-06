<template>
  <div
    class="transaction-item"
    @click="toggleExpand"
    @dblclick="handleDblClick"
    @contextmenu.prevent="handleContextMenu"
    @touchstart="handleTouchStart"
    @touchend="handleTouchEnd"
  >
    <div class="transaction-header">
      <div class="transaction-info">
        <span class="transaction-date">{{ formattedDate }}</span>
        <span class="transaction-desc">{{ tx.description }}</span>
        <span v-if="memberName" class="transaction-member">{{ memberName }}</span>
      </div>
      <div class="transaction-meta">
        <span class="transaction-amount">¥{{ totalAmount.toFixed(2) }}</span>
        <span class="expand-icon">{{ expanded ? '▼' : '▶' }}</span>
      </div>
    </div>
    <div v-if="expanded" class="transaction-detail">
      <div v-if="postings.length === 0" class="empty">暂无分录</div>
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
import { useRouter } from 'vue-router'
import { Modal } from 'ant-design-vue'
import type { Transaction } from '@/stores/transaction'
import { useMemberStore } from '@/stores/member'
import { useTransactionStore } from '@/stores/transaction'

const props = defineProps<{
  tx: Transaction
}>()

const emit = defineEmits<{
  (e: 'deleted', id: number): void
}>()

const router = useRouter()
const memberStore = useMemberStore()
const transactionStore = useTransactionStore()
const expanded = ref(false)
const postings = computed(() => props.tx.postings || [])

const formattedDate = computed(() => {
  const d = new Date(props.tx.date_time)
  return d.toLocaleString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
})

const memberName = computed(() => {
  if (!props.tx.member_id) return ''
  const m = memberStore.members.find((m) => m.id === props.tx.member_id)
  return m?.name || ''
})

const totalAmount = computed(() => {
  let sum = 0
  for (const p of postings.value) {
    const amount = parseFloat(p.amount)
    if (amount > 0) {
      sum += amount
    }
  }
  return sum
})

function toggleExpand() {
  expanded.value = !expanded.value
}

function handleDblClick() {
  if (window.innerWidth >= 768) {
    router.push(`/transaction?id=${props.tx.id}`)
  }
}

function handleContextMenu() {
  if (window.innerWidth >= 768) {
    Modal.confirm({
      title: '确认删除',
      content: `确定要删除这条交易吗？\n${props.tx.description || '无备注'}`,
      okText: '删除',
      okType: 'danger',
      cancelText: '取消',
      async onOk() {
        try {
          await transactionStore.deleteTransaction(props.tx.id)
          emit('deleted', props.tx.id)
        } catch (e) {
          console.error('删除失败', e)
        }
      },
    })
  }
}

let touchStartX = 0
let touchStartY = 0
const SWIPE_THRESHOLD = 60

function handleTouchStart(e: TouchEvent) {
  touchStartX = e.changedTouches[0].screenX
  touchStartY = e.changedTouches[0].screenY
}

function handleTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].screenX - touchStartX
  const dy = e.changedTouches[0].screenY - touchStartY
  if (Math.abs(dy) > Math.abs(dx)) return
  if (Math.abs(dx) < SWIPE_THRESHOLD) return

  if (dx < 0) {
    // 左滑编辑
    router.push(`/transaction?id=${props.tx.id}`)
  } else {
    // 右滑删除
    Modal.confirm({
      title: '确认删除',
      content: `确定要删除这条交易吗？`,
      okText: '删除',
      okType: 'danger',
      cancelText: '取消',
      async onOk() {
        try {
          await transactionStore.deleteTransaction(props.tx.id)
          emit('deleted', props.tx.id)
        } catch (err: any) {
          console.error('删除失败', err)
        }
      },
    })
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
  user-select: none;
  -webkit-user-select: none;
  touch-action: pan-y;
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

.transaction-member {
  color: #999;
  font-size: 12px;
}

.transaction-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.transaction-amount {
  font-size: 15px;
  font-weight: 600;
  color: #f5222d;
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
