<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, ref } from 'vue'
import type { TransactionDto } from '../types/api'

const props = defineProps<{
  tx: TransactionDto
}>()

const emit = defineEmits<{
  (e: 'edit', id: number): void
  (e: 'delete', id: number): void
}>()

const expanded = ref(false)

function toggleExpand() {
  expanded.value = !expanded.value
}

function computeAmount(): Decimal {
  const assetPostings = props.tx.postings.filter((p) => p.account_type === 'asset')
  const sum = assetPostings.reduce(
    (acc, p) => acc.plus(new Decimal(p.amount)),
    new Decimal(0),
  )
  if (!sum.isZero()) return sum
  return assetPostings.reduce(
    (acc, p) => {
      const a = new Decimal(p.amount)
      return a.gt(0) ? acc.plus(a) : acc
    },
    new Decimal(0),
  )
}

const amount = computed(() => computeAmount())

function isTransfer(): boolean {
  return !props.tx.postings.some(
    (p) => p.account_type === 'income' || p.account_type === 'expense',
  )
}

function isPureImport(): boolean {
  return (
    props.tx.postings.length > 0 &&
    props.tx.postings.every((p) => p.account.includes(':Import:'))
  )
}

function getIncomeExpenseAccounts(): string[] {
  return props.tx.postings
    .filter((p) => p.account_type === 'income' || p.account_type === 'expense')
    .map((p) => shortAccountName(p.account))
}

function getAssetAccounts(): string[] {
  return props.tx.postings
    .filter((p) => p.account_type === 'asset')
    .map((p) => shortAccountName(p.account))
}

function shortAccountName(path: string): string {
  const parts = path.split(':')
  return parts[parts.length - 1] || path
}

function formatAmount(amt: Decimal): string {
  const fixed = amt.toFixed(2)
  const [intPart, decPart] = fixed.split('.')
  const sign = intPart.startsWith('-') ? '-' : ''
  const abs = intPart.replace('-', '')
  const formatted = abs.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  return `${sign}${formatted}.${decPart}`
}

function isRefund(): boolean {
  return props.tx.kind === 'refund'
}

let touchStartX = 0
let touchStartY = 0
const SWIPE_THRESHOLD = 60

function onTouchStart(e: TouchEvent) {
  touchStartX = e.changedTouches[0].screenX
  touchStartY = e.changedTouches[0].screenY
}

function onTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].screenX - touchStartX
  const dy = e.changedTouches[0].screenY - touchStartY
  if (Math.abs(dy) > Math.abs(dx)) return
  if (Math.abs(dx) < SWIPE_THRESHOLD) return

  if (dx < 0) {
    emit('edit', props.tx.id)
  } else {
    emit('delete', props.tx.id)
  }
}

function onDblClick() {
  emit('edit', props.tx.id)
}
</script>

<template>
  <div
    class="tx-card"
    @click="toggleExpand"
    @dblclick="onDblClick"
    @touchstart="onTouchStart"
    @touchend="onTouchEnd"
  >
    <div class="tx-top">
      <span v-if="isTransfer() && !isPureImport()" class="transfer-label">转账</span>
      <span v-else-if="!isPureImport()" class="ie-accounts">{{
        getIncomeExpenseAccounts().join(' ')
      }}</span>
      <span v-if="tx.member_name" class="tx-member">{{ tx.member_name }}</span>
      <div v-if="tx.tags.length" class="tags">
        <span v-for="tag in tx.tags" :key="tag" class="tag">{{ tag }}</span>
      </div>
      <span class="expand-indicator">{{ expanded ? '▲' : '▼' }}</span>
    </div>
    <div class="tx-middle">
      <span class="tx-name" :class="{ refund: isRefund() }">
        {{ isRefund() ? '退款 · ' : '' }}{{ tx.description || '' }}
      </span>
      <div
        v-if="!isPureImport()"
        class="tx-amount"
        :class="{ refund: isRefund(), positive: amount.gt(0) }"
      >
        <span v-if="amount.gt(0)">+</span>¥{{ formatAmount(amount) }}
      </div>
    </div>
    <div v-if="!isPureImport()" class="tx-bottom">
      <span class="asset-accounts">{{ getAssetAccounts().join(' ') }}</span>
    </div>
    <Transition name="expand">
      <div v-if="expanded" class="tx-entries">
        <div v-for="posting in tx.postings" :key="posting.id" class="entry-row">
          <span class="entry-account">{{ shortAccountName(posting.account) }}</span>
          <span class="entry-commodity">{{ posting.commodity }}</span>
          <span
            class="entry-amount"
            :class="{
              positive: new Decimal(posting.amount).gt(0),
              negative: new Decimal(posting.amount).lt(0),
            }"
          >
            <span v-if="new Decimal(posting.amount).gt(0)">+</span>{{
              formatAmount(new Decimal(posting.amount))
            }}
          </span>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.tx-card {
  padding: 0.75rem 0.5rem;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
  touch-action: pan-y;
}

.tx-card:last-child {
  border-bottom: none;
}

.tx-top {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.ie-accounts {
  color: var(--text-heading);
  font-size: 0.8125rem;
  font-weight: 500;
}

.transfer-label {
  color: var(--text-muted);
  font-size: 0.75rem;
  background: var(--border);
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
}

.tx-member {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.tags {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
}

.tag {
  color: #e74c3c;
  font-size: 0.6875rem;
  border: 1px solid #e74c3c;
  border-radius: 0.25rem;
  padding: 0 0.375rem;
  line-height: 1.4;
}

.tx-middle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.tx-name {
  flex: 1;
  color: var(--text-heading);
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.tx-name.refund {
  color: #999;
}

.tx-amount {
  color: #e74c3c;
  font-weight: 500;
  font-size: 0.9375rem;
  white-space: nowrap;
  text-align: right;
}

.tx-amount.positive {
  color: #27ae60;
}

.tx-amount.refund {
  color: #999;
}

.tx-bottom {
  display: flex;
  justify-content: flex-end;
}

.asset-accounts {
  color: var(--text-muted);
  font-size: 0.75rem;
  text-align: right;
}

.expand-indicator {
  margin-left: auto;
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.tx-entries {
  margin-top: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--card-bg, #1e1e1e);
  border-radius: 0.5rem;
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 0.25rem 0.75rem;
  overflow: hidden;
}

.entry-row {
  display: contents;
}

.entry-account {
  color: var(--text-heading);
  font-weight: 500;
  font-size: 0.75rem;
  text-align: left;
}

.entry-commodity {
  color: var(--text-muted);
  font-size: 0.75rem;
  text-align: right;
}

.entry-amount {
  color: #3498db;
  font-weight: 500;
  font-size: 0.75rem;
  white-space: nowrap;
  text-align: right;
}

.entry-amount.positive {
  color: #27ae60;
}

.entry-amount.negative {
  color: #e74c3c;
}

.expand-enter-active,
.expand-leave-active {
  transition:
    max-height 0.3s ease-in-out,
    opacity 0.3s ease-in-out;
  max-height: 500px;
  opacity: 1;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
  overflow: hidden;
}
</style>
