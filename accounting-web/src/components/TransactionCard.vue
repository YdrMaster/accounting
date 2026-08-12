<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCommodityStore } from '../stores/commodity'
import type { TransactionDto } from '../types/api'

const { t } = useI18n()
const commodityStore = useCommodityStore()

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
  const assetPostings = props.tx.postings.filter(p => p.account_type === 'asset')
  const sum = assetPostings.reduce((acc, p) => acc.plus(new Decimal(p.amount)), new Decimal(0))
  if (!sum.isZero()) return sum
  return assetPostings.reduce((acc, p) => {
    const a = new Decimal(p.amount)
    return a.gt(0) ? acc.plus(a) : acc
  }, new Decimal(0))
}

const amount = computed(() => computeAmount())

/** 资金流方向摘要：非正值（≤0）分录在前、正值在后，同侧顿号连接，不依赖账户类型 */
const summary = computed(() => {
  const negative: string[] = []
  const positive: string[] = []
  for (const p of props.tx.postings) {
    const amt = new Decimal(p.amount)
    if (amt.lte(0)) negative.push(shortAccountName(p.account))
    else positive.push(shortAccountName(p.account))
  }
  const left = negative.join('、')
  const right = positive.join('、')
  if (!left) return right
  if (!right) return left
  return `${left} → ${right}`
})

/** 按出现次数降序排列的币种名列表 */
const commodities = computed(() => {
  const counts = new Map<string, number>()
  for (const p of props.tx.postings) {
    counts.set(p.commodity, (counts.get(p.commodity) ?? 0) + 1)
  }
  return [...counts.entries()].sort((a, b) => b[1] - a[1]).map(([c]) => c)
})

const multiCurrency = computed(() => commodities.value.length > 1)
const primaryCommodity = computed(() => commodities.value[0] ?? '')
const secondaryCommodity = computed(() => commodities.value[1] ?? '')

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

const COMMODITY_SYMBOLS: Record<string, string> = {
  CNY: '¥',
  USD: '$',
  EUR: '€',
  GBP: '£',
  JPY: '¥',
  HKD: 'HK$',
}

/** 多币种时金额前缀：优先商品符号，未知回退主币种代码 */
function commodityPrefix(code: string): string {
  const dto = commodityStore.commodities.find(c => c.symbol === code || c.name === code)
  const dtoCode = dto?.symbol ?? code
  return COMMODITY_SYMBOLS[dtoCode] ?? dtoCode
}

/** 折叠态金额：单币种无前缀；多币种带主币种前缀 */
const displayAmount = computed(() => {
  const prefix = multiCurrency.value ? commodityPrefix(primaryCommodity.value) : ''
  const sign = amount.value.gt(0) ? '+' : ''
  const body = formatAmount(amount.value)
  const refund = isRefund()
  return { prefix, sign, body, refund }
})

function isRefund(): boolean {
  return props.tx.kind === 'refund'
}

const hasDescription = computed(() => (props.tx.description ?? '').trim().length > 0)

const refundPrefix = computed(() => (isRefund() ? t('txCard.refundPrefix') : ''))

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
    :class="{ pending: tx.pending }"
    @click="toggleExpand"
    @dblclick="onDblClick"
    @touchstart="onTouchStart"
    @touchend="onTouchEnd"
  >
    <!-- 描述非空：两行锚定 -->
    <template v-if="hasDescription">
      <div class="row-main">
        <span class="title" :class="{ refund: refundPrefix }"
          >{{ refundPrefix }}{{ tx.description }}</span
        >
        <span
          class="amount"
          :class="{ refund: displayAmount.refund, 'amount-positive': amount.gt(0) }"
          >{{ displayAmount.prefix }}{{ displayAmount.sign }}{{ displayAmount.body }}</span
        >
      </div>
      <div class="row-sub">
        <span v-if="tx.member_name" class="member">{{ tx.member_name }}</span>
        <span class="summary">{{ summary }}</span>
        <span v-if="multiCurrency && secondaryCommodity" class="currency">{{
          secondaryCommodity
        }}</span>
        <span v-for="tag in tx.tags" :key="tag" class="tag">{{ tag }}</span>
        <span class="expand-indicator">{{ expanded ? '▲' : '▼' }}</span>
      </div>
    </template>
    <!-- 描述为空：单行合并，摘要充当主标题 -->
    <div v-else class="row-main merged">
      <span v-if="tx.member_name" class="member">{{ tx.member_name }}</span>
      <span class="title" :class="{ refund: refundPrefix }">{{ refundPrefix }}{{ summary }}</span>
      <span v-if="multiCurrency && secondaryCommodity" class="currency">{{
        secondaryCommodity
      }}</span>
      <span v-for="tag in tx.tags" :key="tag" class="tag">{{ tag }}</span>
      <span
        class="amount"
        :class="{ refund: displayAmount.refund, 'amount-positive': amount.gt(0) }"
        >{{ displayAmount.prefix }}{{ displayAmount.sign }}{{ displayAmount.body }}</span
      >
      <span class="expand-indicator">{{ expanded ? '▲' : '▼' }}</span>
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
            <span v-if="new Decimal(posting.amount).gt(0)">+</span
            >{{ formatAmount(new Decimal(posting.amount)) }}
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

/* 待分类：琥珀左→右渐变，容器级标识，文字保持原色 */
.tx-card.pending {
  background: linear-gradient(90deg, rgba(245, 158, 11, 0.2), rgba(245, 158, 11, 0) 65%);
}

.row-main {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.row-main.merged {
  gap: 0.5rem;
}

.row-sub {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  min-width: 0;
}

.title {
  flex: 1;
  min-width: 0;
  color: var(--text-heading);
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 单行合并时主标题退缩回内容宽度，避免把金额/标签挤向右缘 */
.row-main.merged .title {
  flex: 0 1 auto;
}

.title.refund {
  color: var(--text-muted);
}

.amount {
  color: var(--color-expense);
  font-weight: 500;
  font-size: 0.9375rem;
  white-space: nowrap;
  text-align: right;
  margin-left: auto;
  flex-shrink: 0;
}

.amount.refund {
  color: var(--text-muted);
}

.amount-positive {
  color: var(--color-income);
}

.member {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.summary {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.currency {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.tag {
  color: var(--color-expense);
  font-size: 0.75rem;
  border: 1px solid var(--color-expense);
  border-radius: 0.25rem;
  padding: 0 0.375rem;
  line-height: 1.4;
  flex-shrink: 0;
}

.expand-indicator {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.row-sub .expand-indicator {
  margin-left: auto;
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
  color: var(--color-info);
  font-weight: 500;
  font-size: 0.75rem;
  white-space: nowrap;
  text-align: right;
}

.entry-amount.positive {
  color: var(--color-income);
}

.entry-amount.negative {
  color: var(--color-expense);
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
