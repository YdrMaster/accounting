<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCommodityStore } from '../stores/commodity'
import type { TransactionDto } from '../types/api'
import { sumAssetAmount } from '../utils/amount'

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
  return sumAssetAmount(props.tx.postings)
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

const isTransfer = computed(() => props.tx.is_transfer)

/** 转账转出金额：资产类分录中负金额之和的相反数 */
const transferOutAmount = computed(() => {
  let sum = new Decimal(0)
  for (const p of props.tx.postings) {
    if (p.account_type !== 'asset') continue
    const a = new Decimal(p.amount)
    if (a.lt(0)) sum = sum.plus(a)
  }
  return sum.negated()
})

/** 转账引起的资产变动金额：非资产类分录金额之和 */
const assetChangeAmount = computed(() => {
  let sum = new Decimal(0)
  for (const p of props.tx.postings) {
    if (p.account_type === 'asset') continue
    sum = sum.plus(new Decimal(p.amount))
  }
  return sum
})

/** 折叠态金额：单币种无前缀；多币种带主币种前缀；转账为蓝色转出金额且不带符号 */
const displayAmount = computed(() => {
  const prefix = multiCurrency.value ? commodityPrefix(primaryCommodity.value) : ''
  const refund = isRefund()
  if (isTransfer.value) {
    return { prefix, sign: '', body: formatAmount(transferOutAmount.value), refund, transfer: true }
  }
  const sign = amount.value.gt(0) ? '+' : ''
  return { prefix, sign, body: formatAmount(amount.value), refund, transfer: false }
})

/** 非转账交易金额为 0：低对比度渲染 0.00 */
const isZeroAmount = computed(() => !displayAmount.value.transfer && amount.value.isZero())

/** 转账资产变动额：S>0（资产减少）显示红色 -S；S<0（资产增加）显示绿色 +S；S=0 不显示 */
const assetChange = computed(() => {
  if (!isTransfer.value) return null
  const s = assetChangeAmount.value
  if (s.isZero()) return null
  const decreasing = s.gt(0)
  return {
    sign: decreasing ? '-' : '+',
    body: formatAmount(s.abs()),
    className: decreasing ? 'negative' : 'positive',
  }
})

function isRefund(): boolean {
  return props.tx.kind === 'refund'
}

const hasDescription = computed(() => (props.tx.description ?? '').trim().length > 0)

const refundPrefix = computed(() => (isRefund() ? t('txCard.refundPrefix') : ''))

let touchStartX = 0
let touchStartY = 0
const SWIPE_THRESHOLD = 60

/** 右滑过程反馈：横向位移（px）与是否越过删除阈值 */
const swipeDx = ref(0)
const swipeArmed = ref(false)

const cardStyle = computed(() =>
  swipeDx.value > 0 ? { transform: `translateX(${swipeDx.value}px)` } : {}
)

function onTouchStart(e: TouchEvent) {
  touchStartX = e.changedTouches[0].screenX
  touchStartY = e.changedTouches[0].screenY
  swipeDx.value = 0
  swipeArmed.value = false
}

function onTouchMove(e: TouchEvent) {
  const dx = e.changedTouches[0].screenX - touchStartX
  const dy = e.changedTouches[0].screenY - touchStartY
  if (Math.abs(dy) > Math.abs(dx)) {
    // 纵向滚动优先，放弃横向反馈
    swipeDx.value = 0
    swipeArmed.value = false
    return
  }
  // 仅右滑（删除方向）给出反馈
  swipeDx.value = dx > 0 ? Math.min(dx, SWIPE_THRESHOLD * 1.5) : 0
  swipeArmed.value = dx >= SWIPE_THRESHOLD
}

function onTouchEnd(e: TouchEvent) {
  const dx = e.changedTouches[0].screenX - touchStartX
  const dy = e.changedTouches[0].screenY - touchStartY
  swipeDx.value = 0
  swipeArmed.value = false
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
    :class="{ pending: tx.pending, 'swipe-armed': swipeArmed }"
    :style="cardStyle"
    @click="toggleExpand"
    @dblclick="onDblClick"
    @touchstart="onTouchStart"
    @touchmove="onTouchMove"
    @touchend="onTouchEnd"
  >
    <!-- 描述非空：两行锚定 -->
    <template v-if="hasDescription">
      <div class="row-main">
        <span class="title" :class="{ refund: refundPrefix }"
          >{{ refundPrefix }}{{ tx.description }}</span
        >
        <span class="amount-group">
          <span
            class="amount"
            :class="{
              refund: displayAmount.refund,
              'amount-transfer': displayAmount.transfer,
              'amount-positive': !displayAmount.transfer && amount.gt(0),
              'amount-zero': isZeroAmount,
            }"
            >{{ displayAmount.prefix }}{{ displayAmount.sign }}{{ displayAmount.body }}</span
          >
          <span v-if="assetChange" class="amount-change" :class="assetChange.className"
            >{{ assetChange.sign }}{{ assetChange.body }}</span
          >
        </span>
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
      <span class="amount-group">
        <span
          class="amount"
          :class="{
            refund: displayAmount.refund,
            'amount-transfer': displayAmount.transfer,
            'amount-positive': !displayAmount.transfer && amount.gt(0),
            'amount-zero': isZeroAmount,
          }"
          >{{ displayAmount.prefix }}{{ displayAmount.sign }}{{ displayAmount.body }}</span
        >
        <span v-if="assetChange" class="amount-change" :class="assetChange.className"
          >{{ assetChange.sign }}{{ assetChange.body }}</span
        >
      </span>
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
  transition:
    transform 0.15s ease,
    background-color 0.15s ease;
}

.tx-card:last-child {
  border-bottom: none;
}

/* 待分类：琥珀左→右渐变，容器级标识，文字保持原色 */
.tx-card.pending {
  background: linear-gradient(90deg, rgba(245, 158, 11, 0.2), rgba(245, 158, 11, 0) 65%);
}

/* 右滑越阈：危险色高亮提示将触发删除 */
.tx-card.swipe-armed {
  background: rgba(231, 76, 60, 0.18);
  box-shadow: inset -3px 0 0 var(--color-expense);
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
  margin-left: 0;
  flex-shrink: 0;
}

.amount-group {
  display: flex;
  align-items: baseline;
  gap: 0.375rem;
  margin-left: auto;
  flex-shrink: 0;
}

.amount-transfer {
  color: var(--color-info);
}

.amount-change {
  font-weight: 500;
  font-size: 0.9375rem;
  white-space: nowrap;
}

.amount-change.positive {
  color: var(--color-income);
}

.amount-change.negative {
  color: var(--color-expense);
}

.amount.refund {
  color: var(--text-muted);
}

.amount-positive {
  color: var(--color-income);
}

/* 零金额：低对比度，不占用收入/支出语义色 */
.amount.amount-zero {
  color: var(--text-muted);
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
