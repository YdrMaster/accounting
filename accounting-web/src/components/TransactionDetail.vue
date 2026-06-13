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
      <!-- 栏1：记账人+时间 -->
      <div class="col-member">
        <span v-if="memberName" class="transaction-member">{{ memberName }}</span>
        <span class="transaction-date">{{ formattedDate }}</span>
      </div>
      <!-- 栏2：交易摘要+备注 -->
      <div class="col-info">
        <div class="account-tags">
          <template v-if="incomeAccounts.length && !expenseAccounts.length">
            <span
              v-for="name in incomeAccounts"
              :key="name"
              class="tag-income"
            >{{ name }}</span>
          </template>
          <template v-else-if="expenseAccounts.length && !incomeAccounts.length">
            <span
              v-for="name in expenseAccounts"
              :key="name"
              class="tag-expense"
            >{{ name }}</span>
          </template>
          <template v-else-if="isTransferType">
            <span class="transaction-summary">{{ hasRepaymentTag ? '还款' : '转账' }}</span>
          </template>
        </div>
        <span class="transaction-desc" :title="tx.description">{{ firstLinePreview }}</span>
      </div>
      <!-- 栏3：金额+资产账户 -->
      <div class="col-amount">
        <span class="transaction-amount" :class="amountColorClass">¥{{ Math.abs(netAmount).toFixed(2) }}</span>
        <span v-if="hasReversal" class="original-amount">¥{{ Math.abs(totalAmount).toFixed(2) }}</span>
        <div class="meta-row">
          <span v-if="channelName" class="channel-tag">{{ channelName }}</span>
          <span class="asset-accounts">{{ assetAccounts.join(' ') }}</span>
        </div>
      </div>
      <span class="expand-icon">{{ expanded ? '▼' : '▶' }}</span>
    </div>
    <div v-if="expanded" class="transaction-detail">
      <div v-if="postings.length === 0" class="empty">暂无分录</div>
      <div v-else class="postings">
        <div
          v-for="p in postings"
          :key="p.id"
          class="posting-row"
          :class="postingRowClass(p)"
          @click.stop="onPostingClick(p)"
        >
          <span class="posting-account">
            {{ p.account }}
            <span v-if="p.linked_posting_id != null && tx.kind === 'refund'" class="badge-refund">退</span>
            <span v-else-if="p.linked_posting_id != null && tx.kind === 'reimbursement'" class="badge-reimb">报</span>
            <span v-else-if="p.is_reimbursable" class="badge-reimb">报</span>
            <span v-if="p.linked_posting_id" class="linked-tag">冲减分录 #{{ p.linked_posting_id }}</span>
          </span>
          <span class="posting-commodity">{{ p.commodity }}</span>
          <span class="posting-amount" :class="{ positive: Number(p.amount) > 0, negative: Number(p.amount) < 0 }">
            <template v-if="Number(p.reversal_total) !== 0">
              ¥{{ Math.abs(postingNetAmount(p)).toFixed(2) }}
              <span class="original-amount">¥{{ Math.abs(Number(p.amount)).toFixed(2) }}</span>
            </template>
            <template v-else>
              ¥{{ Math.abs(Number(p.amount)).toFixed(2) }}
            </template>
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
import type { Transaction, Posting } from '@/stores/transaction'
import { useMemberStore } from '@/stores/member'
import { useTransactionStore } from '@/stores/transaction'
import { useAccountStore } from '@/stores/account'
import { useChannelStore } from '@/stores/channel'

const props = defineProps<{
  tx: Transaction
  selectable?: boolean
  selectableFilter?: (p: Posting) => boolean
  selectedPostingIds?: Set<number>
}>()

const emit = defineEmits<{
  (e: 'deleted', id: number): void
  (e: 'select-posting', id: number): void
}>()

const router = useRouter()
const memberStore = useMemberStore()
const transactionStore = useTransactionStore()
const accountStore = useAccountStore()
const channelStore = useChannelStore()
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

const channelName = computed(() => {
  if (!props.tx.channel_id) return ''
  const c = channelStore.channels.find((c) => c.id === props.tx.channel_id)
  return c?.name || ''
})

const totalAmount = computed(() => {
  let expenseSum = 0
  let incomeSum = 0
  for (const p of postings.value) {
    const acc = accountStore.accounts.find((a) => a.full_name === p.account)
    if (acc?.account_type === 'Expense') {
      expenseSum += parseFloat(p.amount) || 0
    } else if (acc?.account_type === 'Income') {
      incomeSum += parseFloat(p.amount) || 0
    }
  }
  return -(expenseSum + incomeSum)
})

const netAmount = computed(() => {
  let expenseNet = 0
  let incomeSum = 0
  for (const p of postings.value) {
    const acc = accountStore.accounts.find((a) => a.full_name === p.account)
    if (acc?.account_type === 'Expense') {
      expenseNet += (parseFloat(p.amount) || 0) + (parseFloat(p.reversal_total || '0'))
    } else if (acc?.account_type === 'Income') {
      incomeSum += parseFloat(p.amount) || 0
    }
  }
  return -(expenseNet + incomeSum)
})

const hasReversal = computed(() => {
  return postings.value.some(p => Number(p.reversal_total) !== 0)
})

const firstLinePreview = computed(() => {
  const text = props.tx.description || ''
  const firstLine = text.split('\n')[0] || ''
  return firstLine
})

function lastSegment(fullName: string): string {
  const segments = fullName.split(':')
  return segments[segments.length - 1] || fullName
}

function accountsByType(type: string): string[] {
  const names = new Set<string>()
  for (const p of postings.value) {
    const acc = accountStore.accounts.find((a) => a.full_name === p.account)
    if (acc?.account_type === type) {
      names.add(lastSegment(acc.full_name))
    }
  }
  return Array.from(names)
}

const expenseAccounts = computed(() => accountsByType('Expense'))
const incomeAccounts = computed(() => accountsByType('Income'))
const assetAccounts = computed(() => accountsByType('Asset'))

const amountColorClass = computed(() => {
  if (netAmount.value > 0) return 'amount-income'
  if (netAmount.value < 0) return 'amount-expense'
  return 'amount-neutral'
})

const isTransferType = computed(() => {
  if (postings.value.length === 0) return false
  for (const p of postings.value) {
    const acc = accountStore.accounts.find((a) => a.full_name === p.account)
    if (!acc) return false
    if (acc.account_type === 'Asset') continue
    if (acc.account_type === 'Expense' && acc.parent_id != null) continue
    return false
  }
  return true
})

const hasRepaymentTag = computed(() => {
  return (props.tx.tags || []).includes('还款')
})

function postingNetAmount(p: Posting): number {
  return parseFloat(p.amount) + parseFloat(p.reversal_total || '0')
}

function postingRowClass(p: Posting): Record<string, boolean> {
  const cls: Record<string, boolean> = {}
  if (props.selectable) {
    if (props.selectableFilter && !props.selectableFilter(p)) {
      cls['selectable-disabled'] = true
    } else {
      cls['selectable'] = true
    }
  }
  if (props.selectedPostingIds?.has(p.id)) {
    cls['selected'] = true
  }
  if (p.is_reimbursable && p.linked_posting_id == null) {
    cls['reimbursable'] = true
  }
  return cls
}

function onPostingClick(p: Posting) {
  if (!props.selectable) return
  if (props.selectableFilter && !props.selectableFilter(p)) return
  emit('select-posting', p.id)
}

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
  align-items: stretch;
  gap: 12px;
}

/* 栏1：记账人+时间 */
.col-member {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 4px;
  padding-right: 12px;
  border-right: 1px solid #f0f0f0;
  min-width: 52px;
  flex-shrink: 0;
}

.transaction-member {
  color: #1890ff;
  font-size: 12px;
  font-weight: 500;
}

.transaction-date {
  color: #666;
  font-size: 12px;
}

/* 栏2：收支账户+备注 */
.col-info {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.account-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  font-size: 13px;
}

.tag-income {
  color: #52c41a;
  font-size: 15px;
  font-weight: bold;
}

.tag-expense {
  color: #f5222d;
  font-size: 15px;
  font-weight: bold;
}

.transaction-desc {
  color: #666;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.transaction-summary {
  font-size: 15px;
  font-weight: bold;
  color: #333;
}

/* 栏3：金额+资产 */
.col-amount {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: flex-end;
  gap: 4px;
  padding-left: 12px;
  border-left: 1px solid #f0f0f0;
  min-width: 80px;
  flex-shrink: 0;
}

.transaction-amount {
  font-size: 15px;
  font-weight: 600;
}

.amount-expense {
  color: #f5222d;
}

.amount-income {
  color: #52c41a;
}

.amount-neutral {
  color: #666;
}

.meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
}

.asset-accounts {
  color: #666;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.channel-tag {
  display: inline-block;
  font-size: 11px;
  color: #fa8c16;
  background: transparent;
  border: 1px solid #fa8c16;
  padding: 0 5px;
  border-radius: 4px;
  white-space: nowrap;
  flex-shrink: 0;
}

.expand-icon {
  color: #999;
  font-size: 12px;
  align-self: center;
  margin-left: 4px;
  flex-shrink: 0;
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

.posting-row.selectable { cursor: pointer; }
.posting-row.selectable-disabled { opacity: 0.4; cursor: not-allowed; }
.posting-row.selected { background: #bae7ff; border: 1px solid #1890ff; }
.posting-row.reimbursable { background: #e6f7ff; }

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

.badge-refund {
  color: #fa541c;
  border: 1px solid #fa541c;
  font-size: 12px;
  padding: 0 4px;
  border-radius: 2px;
  margin-left: 4px;
}

.badge-reimb {
  color: #1890ff;
  border: 1px solid #1890ff;
  font-size: 12px;
  padding: 0 4px;
  border-radius: 2px;
  margin-left: 4px;
}

.linked-tag {
  color: #999;
  font-size: 11px;
  margin-left: 4px;
}

.original-amount {
  color: #1890ff;
  text-decoration: line-through;
  font-size: 12px;
  margin-left: 4px;
}
</style>
