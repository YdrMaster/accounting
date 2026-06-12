<template>
  <div class="dashboard">
    <!-- 工具栏：模式切换 + 筛选 -->
    <div class="toolbar">
      <a-space>
        <a-radio-group v-model:value="mode" button-style="solid" size="small">
          <a-radio-button value="normal">普通</a-radio-button>
          <a-radio-button value="range">范围</a-radio-button>
          <a-radio-button value="refund">退款</a-radio-button>
          <a-radio-button value="reimbursement">报销</a-radio-button>
        </a-radio-group>
        <a-button @click="showFilter = true">
          <FilterOutlined />
          筛选
        </a-button>
        <a-button v-if="hasFilter" type="link" @click="clearFilter">
          清除筛选
        </a-button>
      </a-space>
    </div>

    <Calendar
      :data="calendarData"
      :mode="mode"
      @select="mode !== 'range' ? handleSelect($event) : undefined"
      @select-range="handleSelectRange"
      @clear="handleClear"
    />

    <!-- 默认状态：当月收支概览 -->
    <div v-if="!selectedDate && !rangeFrom && !hasFilter" class="section overview">
      <h3>当月收支概览</h3>
      <div class="stats-row">
        <a-statistic title="收入" :value="monthlyIncome" prefix="¥" :value-style="{ color: '#52c41a' }" />
        <a-statistic title="支出" :value="monthlyExpense" prefix="¥" :value-style="{ color: '#f5222d' }" />
        <a-statistic title="结余" :value="monthlyIncome - monthlyExpense" prefix="¥" :value-style="{ color: monthlyIncome >= monthlyExpense ? '#52c41a' : '#f5222d' }" />
      </div>
    </div>

    <!-- 交易列表（按日分组） -->
    <div v-else class="section">
      <a-button v-if="selectedDate" type="primary" block class="action-btn" @click="goToTransaction">
        记一笔
      </a-button>
      <a-tag v-if="rangeFrom && rangeTo" color="blue" class="range-tag">{{ rangeFrom }} 至 {{ rangeTo }}</a-tag>
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="groupedTransactions.length === 0" class="empty">暂无交易</div>
      <div v-else class="date-groups">
        <div v-for="group in groupedTransactions" :key="group.date" class="date-group">
          <div class="date-header" @click="toggleDate(group.date)">
            <span class="date-title">{{ group.date }}</span>
            <span class="date-icon">{{ isDateExpanded(group.date) ? '▼' : '▶' }}</span>
          </div>
          <div v-if="isDateExpanded(group.date)" class="date-items">
            <TransactionDetail
              v-for="tx in group.transactions"
              :key="tx.id"
              :tx="tx"
              :selectable="isSelectMode"
              :selectable-filter="mode === 'reimbursement' ? reimbursableFilter : undefined"
              :selected-posting-ids="selectedPostingIds"
              @deleted="handleDeleted"
              @select-posting="togglePostingSelection"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 筛选 Modal -->
    <a-modal
      v-model:open="showFilter"
      title="筛选交易"
      @ok="applyFilter"
      @cancel="showFilter = false"
    >
      <a-form layout="vertical">
        <a-form-item label="账户">
          <a-tree-select
            v-model:value="filterForm.account"
            :tree-data="accountTreeData"
            :field-names="{ children: 'children', label: 'title', value: 'key' }"
            placeholder="选择账户"
            allow-clear
            tree-default-expand-all
          />
        </a-form-item>
        <a-form-item label="成员">
          <a-select v-model:value="filterForm.member" placeholder="选择成员" allow-clear>
            <a-select-option
              v-for="m in memberStore.members"
              :key="m.id"
              :value="m.id"
            >
              {{ m.name }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="标签">
          <a-select
            v-model:value="filterForm.tag"
            placeholder="选择标签"
            allow-clear
          >
            <a-select-option
              v-for="t in tags"
              :key="t.id"
              :value="t.name"
            >
              {{ t.name }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="关键词">
          <a-input v-model:value="filterForm.keyword" placeholder="备注关键词" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 底部抽屉：选中分录 -->
    <div v-if="isSelectMode && selectedPostingIds.size > 0" class="bottom-drawer" :class="{ collapsed: !drawerExpanded }">
      <div class="drawer-left">
        <span class="count-badge">{{ selectedPostingIds.size }}</span>
        <span class="toggle-icon" @click="drawerExpanded = !drawerExpanded">
          {{ drawerExpanded ? '▶' : '◀' }}
        </span>
      </div>
      <div v-if="drawerExpanded" class="drawer-body">
        <div class="drawer-actions">
          <a-button type="primary" style="background: #52c41a; border-color: #52c41a" @click="confirmSelection">
            确定
          </a-button>
          <a-button @click="cancelSelection">取消</a-button>
        </div>
        <div class="drawer-cards">
          <div v-for="p in selectedPostingsInfo" :key="p.id" class="selected-card">
            <div class="card-account">{{ p.account }}</div>
            <div class="card-meta">{{ p.date }} · {{ p.description }}</div>
            <div class="card-amount">{{ p.amount }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import dayjs from 'dayjs'
import { FilterOutlined } from '@ant-design/icons-vue'
import Calendar from '@/components/Calendar.vue'
import TransactionDetail from '@/components/TransactionDetail.vue'
import { useTransactionStore, type Transaction, type Posting } from '@/stores/transaction'
import { useAccountStore } from '@/stores/account'
import { useMemberStore } from '@/stores/member'
import { useChannelStore } from '@/stores/channel'
import api from '@/api/client'

const router = useRouter()
const transactionStore = useTransactionStore()
const accountStore = useAccountStore()
const memberStore = useMemberStore()
const channelStore = useChannelStore()

type DashboardMode = 'normal' | 'range' | 'refund' | 'reimbursement'
const mode = ref<DashboardMode>('normal')
const selectedPostingIds = ref<Set<number>>(new Set())
const drawerExpanded = ref(false)
const isSelectMode = computed(() => mode.value === 'refund' || mode.value === 'reimbursement')

const selectedDate = ref<string | null>(null)
const rangeFrom = ref<string | null>(null)
const rangeTo = ref<string | null>(null)
const showFilter = ref(false)

const tags = ref<{ id: number; name: string }[]>([])

// 专门用于日历和当月概览的交易数据（不受单日/范围筛选影响）
const monthTransactions = ref<Transaction[]>([])

const filterForm = ref({
  account: undefined as number | undefined,
  member: undefined as number | undefined,
  tag: undefined as string | undefined,
  keyword: '',
})

const hasFilter = computed(() => {
  return (
    filterForm.value.account !== undefined ||
    filterForm.value.member !== undefined ||
    filterForm.value.tag !== undefined ||
    filterForm.value.keyword !== ''
  )
})

const loading = computed(() => transactionStore.loading)
const filteredTransactions = computed(() => transactionStore.transactions)

const expandedDates = ref<Set<string>>(new Set())

const groupedTransactions = computed(() => {
  const groups: Record<string, Transaction[]> = {}
  for (const tx of filteredTransactions.value) {
    const date = tx.date_time.slice(0, 10)
    if (!groups[date]) groups[date] = []
    groups[date].push(tx)
  }
  // 按日期倒序排列
  return Object.keys(groups)
    .sort()
    .reverse()
    .map((date) => ({ date, transactions: groups[date] }))
})

function toggleDate(date: string) {
  const set = new Set(expandedDates.value)
  if (set.has(date)) {
    set.delete(date)
  } else {
    set.add(date)
  }
  expandedDates.value = set
}

function isDateExpanded(date: string) {
  return expandedDates.value.has(date)
}

watch(selectedDate, (date) => {
  if (date) {
    expandedDates.value = new Set([date])
  }
})

function calcIncome(transactions: Transaction[]) {
  let sum = 0
  for (const tx of transactions) {
    for (const p of tx.postings || []) {
      const acc = accountStore.accounts.find((a) => a.full_name === p.account)
      if (acc?.account_type === 'Income') {
        sum += Math.abs(parseFloat(p.amount) || 0)
      }
    }
  }
  return sum
}

function calcExpense(transactions: Transaction[]) {
  let sum = 0
  for (const tx of transactions) {
    for (const p of tx.postings || []) {
      const acc = accountStore.accounts.find((a) => a.full_name === p.account)
      if (acc?.account_type === 'Expense') {
        sum += Math.abs(parseFloat(p.amount) || 0)
      }
    }
  }
  return sum
}

const monthlyIncome = computed(() => calcIncome(monthTransactions.value))
const monthlyExpense = computed(() => calcExpense(monthTransactions.value))

const calendarData = computed(() => {
  const data: Record<string, { income: number; expense: number }> = {}
  for (const tx of monthTransactions.value) {
    const date = tx.date_time.slice(0, 10)
    if (!data[date]) {
      data[date] = { income: 0, expense: 0 }
    }
    for (const p of tx.postings || []) {
      const acc = accountStore.accounts.find((a) => a.full_name === p.account)
      const amount = Math.abs(parseFloat(p.amount) || 0)
      if (acc?.account_type === 'Income') {
        data[date].income += amount
      } else if (acc?.account_type === 'Expense') {
        data[date].expense += amount
      }
    }
  }
  return data
})

const accountTreeData = computed(() => {
  const accounts = accountStore.accounts
  const map = new Map<number, any>()
  const roots: any[] = []

  accounts.forEach((acc) => {
    const segments = acc.full_name.split(':')
    map.set(acc.id, {
      title: segments[segments.length - 1],
      key: acc.id,
      children: [],
    })
  })

  accounts.forEach((acc) => {
    const node = map.get(acc.id)
    if (!node) return
    if (acc.parent_id && map.has(acc.parent_id)) {
      map.get(acc.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  return roots
})

function handleSelect(date: string) {
  selectedDate.value = date
  rangeFrom.value = null
  rangeTo.value = null
  selectedPostingIds.value = new Set()
  drawerExpanded.value = false
  fetchData()
}

function handleSelectRange(from: string, to: string) {
  selectedDate.value = null
  rangeFrom.value = from
  rangeTo.value = to
  selectedPostingIds.value = new Set()
  drawerExpanded.value = false
  fetchData()
}

function handleClear() {
  selectedDate.value = null
  rangeFrom.value = null
  rangeTo.value = null
  selectedPostingIds.value = new Set()
  drawerExpanded.value = false
  if (!hasFilter.value) {
    fetchMonthData()
  } else {
    fetchData()
  }
}

async function fetchTags() {
  try {
    const res = await api.get<{ id: number; name: string }[]>('/tags')
    tags.value = res.data
  } catch (e) {
    console.error('获取标签失败', e)
  }
}

function buildParams(): Record<string, unknown> {
  const params: Record<string, unknown> = {}
  if (selectedDate.value) {
    params.from = selectedDate.value
    params.to = selectedDate.value
  } else if (rangeFrom.value && rangeTo.value) {
    params.from = rangeFrom.value
    params.to = rangeTo.value
  } else {
    const start = dayjs().startOf('month').format('YYYY-MM-DD')
    const end = dayjs().endOf('month').format('YYYY-MM-DD')
    params.from = start
    params.to = end
  }
  if (filterForm.value.account) {
    params.account = filterForm.value.account
  }
  if (filterForm.value.member) {
    params.member = filterForm.value.member
  }
  if (filterForm.value.tag) {
    params.tag = filterForm.value.tag
  }
  if (filterForm.value.keyword) {
    params.keyword = filterForm.value.keyword
  }
  if (mode.value === 'reimbursement') {
    params.reimbursable = true
  }
  return params
}

function fetchData() {
  transactionStore.fetchTransactions(buildParams())
}

async function fetchMonthData() {
  const start = dayjs().startOf('month').format('YYYY-MM-DD')
  const end = dayjs().endOf('month').format('YYYY-MM-DD')
  await transactionStore.fetchTransactions({ from: start, to: end })
  monthTransactions.value = transactionStore.transactions
}

function applyFilter() {
  showFilter.value = false
  fetchData()
}

function clearFilter() {
  filterForm.value = {
    account: undefined,
    member: undefined,
    tag: undefined,
    keyword: '',
  }
  fetchData()
}

function goToTransaction() {
  if (selectedDate.value) {
    router.push(`/transaction?date=${selectedDate.value}`)
  }
}

function handleDeleted() {
  fetchData()
}

function togglePostingSelection(id: number) {
  const next = new Set(selectedPostingIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedPostingIds.value = next
}

const reimbursableFilter = (p: Posting) => p.is_reimbursable

const selectedPostingsInfo = computed(() => {
  const result: any[] = []
  for (const tx of transactionStore.transactions) {
    for (const p of tx.postings) {
      if (selectedPostingIds.value.has(p.id)) {
        result.push({ ...p, date: tx.date_time.slice(0, 10), description: tx.description })
      }
    }
  }
  return result
})

function confirmSelection() {
  const ids = Array.from(selectedPostingIds.value).join(',')
  const path = mode.value === 'refund' ? '/transaction/refund' : '/transaction/reimbursement'
  router.push(`${path}?posting_ids=${ids}`)
}

function cancelSelection() {
  selectedPostingIds.value = new Set()
  drawerExpanded.value = false
}

watch(mode, (newMode) => {
  selectedPostingIds.value = new Set()
  drawerExpanded.value = false
  if (newMode === 'reimbursement') {
    fetchData()
  } else if (newMode === 'normal' || newMode === 'refund') {
    fetchData()
  }
})

onMounted(() => {
  fetchMonthData()
  accountStore.fetchAccounts()
  memberStore.fetchMembers()
  channelStore.fetchChannels()
  fetchTags()
})
</script>

<style scoped>
.dashboard {
  max-width: 800px;
  margin: 0 auto;
}

.toolbar {
  margin-bottom: 16px;
  display: flex;
  justify-content: flex-end;
}

.section {
  margin-top: 24px;
}

.overview {
  background: #fff;
  padding: 24px;
  border-radius: 8px;
}

.stats-row {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.action-btn {
  margin-bottom: 16px;
}

.range-tag {
  margin-bottom: 16px;
  font-size: 14px;
}

.date-groups {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.date-group {
  background: #fff;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.date-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  background: #fafafa;
  font-size: 15px;
  font-weight: 500;
  color: #333;
  transition: background 0.2s;
}

.date-header:hover {
  background: #f0f0f0;
}

.date-icon {
  color: #999;
  font-size: 12px;
}

.date-items {
  padding: 8px 16px 12px;
}

.date-items .transaction-item {
  margin-bottom: 4px;
  box-shadow: none;
  border-radius: 4px;
  padding: 10px 0;
  border-bottom: 1px solid #f0f0f0;
}

.date-items .transaction-item:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

.loading,
.empty {
  text-align: center;
  color: #999;
  padding: 24px;
}

.bottom-drawer {
  position: fixed;
  bottom: 0;
  right: 0;
  background: #fff;
  border: 1px solid #d9d9d9;
  border-radius: 8px 0 0 0;
  padding: 8px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  z-index: 1000;
  max-width: 80vw;
}
.bottom-drawer.collapsed {
  padding: 8px 12px;
}
.drawer-left {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.count-badge {
  background: #1890ff;
  color: #fff;
  border-radius: 10px;
  padding: 2px 8px;
  font-size: 14px;
  font-weight: bold;
}
.toggle-icon { cursor: pointer; }
.drawer-body { display: flex; align-items: flex-start; gap: 8px; }
.drawer-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.drawer-cards {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  scrollbar-width: none;
}
.drawer-cards::-webkit-scrollbar { display: none; }
.selected-card {
  background: #f5f5f5;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  padding: 6px 10px;
  white-space: nowrap;
  min-width: 140px;
}
.card-account { font-weight: 600; }
.card-meta { font-size: 12px; color: #999; }
.card-amount { font-size: 14px; }
</style>
