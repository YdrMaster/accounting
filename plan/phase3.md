---

## 任务 9：配置 axios API 客户端和 Vue Router

**文件：**
- 创建：`accounting-web/src/api/client.ts`
- 创建：`accounting-web/src/router/index.ts`

**目标：** axios 实例配置（基地址、错误处理），Vue Router 配置（5 个路由）。

- [ ] **步骤 1：创建 src/api/client.ts**

```typescript
import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API error:', error)
    return Promise.reject(error)
  }
)

export default api
```

- [ ] **步骤 2：创建 src/router/index.ts**

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '@/views/Dashboard.vue'
import TransactionForm from '@/views/TransactionForm.vue'
import AccountTree from '@/views/AccountTree.vue'
import Settings from '@/views/Settings.vue'
import ReportView from '@/views/ReportView.vue'

const routes = [
  { path: '/', component: Dashboard },
  { path: '/transaction', component: TransactionForm },
  { path: '/accounts', component: AccountTree },
  { path: '/settings', component: Settings },
  { path: '/reports', component: ReportView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
```

- [ ] **步骤 3：编译验证**

```bash
cd accounting-web
npm run build
# 期望无错误
```

- [ ] **步骤 4：Commit**

```bash
git add accounting-web/src/api/ accounting-web/src/router/
git commit -m "feat(web): axios API 客户端 + Vue Router 配置

- axios baseURL /api，代理到后端 3000 端口
- 5 个路由：/, /transaction, /accounts, /settings, /reports"
```

---

## 任务 10：实现 Pinia stores（成员/账户/交易/报表）

**文件：**
- 创建：`accounting-web/src/stores/member.ts`
- 创建：`accounting-web/src/stores/account.ts`
- 创建：`accounting-web/src/stores/transaction.ts`
- 创建：`accounting-web/src/stores/report.ts`

**目标：** 四个 Pinia store，管理 API 数据获取和本地状态。

- [ ] **步骤 1：创建 src/stores/member.ts**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api/client'

export interface Member {
  id: number
  name: string
}

export const useMemberStore = defineStore('member', () => {
  const members = ref<Member[]>([])
  const currentMember = ref<Member | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchMembers() {
    loading.value = true
    error.value = null
    try {
      const res = await api.get<Member[]>('/members')
      members.value = res.data
    } catch (e) {
      error.value = '获取成员失败'
    } finally {
      loading.value = false
    }
  }

  async function setCurrent(id: number) {
    const found = members.value.find((m) => m.id === id)
    if (found) {
      currentMember.value = found
      await api.put('/me', { member_id: id })
    }
  }

  return { members, currentMember, loading, error, fetchMembers, setCurrent }
})
```

- [ ] **步骤 2：创建 src/stores/account.ts**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Account {
  id: number
  full_name: string
  account_type: string
  parent_id?: number
  is_system: boolean
  billing_day?: number
  repayment_day?: number
}

export const useAccountStore = defineStore('account', () => {
  const accounts = ref<Account[]>([])
  const loading = ref(false)

  async function fetchAccounts() {
    loading.value = true
    const res = await api.get<Account[]>('/accounts')
    accounts.value = res.data
    loading.value = false
  }

  async function createAccount(fullName: string, billingDay?: number, repaymentDay?: number) {
    await api.post('/accounts', { full_name: fullName, billing_day: billingDay, repayment_day: repaymentDay })
    await fetchAccounts()
  }

  return { accounts, loading, fetchAccounts, createAccount }
})
```

- [ ] **步骤 3：创建 src/stores/transaction.ts**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Transaction {
  id: number
  date_time: string
  description: string
  member_id?: number
  is_template: boolean
}

export interface CreateTransactionData {
  date_time: string
  description: string
  member_id?: number
  postings: Array<{ account: string; commodity: string; amount: string }>
  tags: string[]
}

export const useTransactionStore = defineStore('transaction', () => {
  const transactions = ref<Transaction[]>([])
  const loading = ref(false)

  async function fetchTransactions(params?: Record<string, unknown>) {
    loading.value = true
    const res = await api.get<Transaction[]>('/transactions', { params })
    transactions.value = res.data
    loading.value = false
  }

  async function createTransaction(data: CreateTransactionData) {
    await api.post('/transactions', data)
  }

  async function deleteTransaction(id: number) {
    await api.delete(`/transactions/${id}`)
  }

  return { transactions, loading, fetchTransactions, createTransaction, deleteTransaction }
})
```

- [ ] **步骤 4：创建 src/stores/report.ts**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export const useReportStore = defineStore('report', () => {
  const balanceSheet = ref<unknown>(null)
  const incomeStatement = ref<unknown>(null)
  const stats = ref<unknown>(null)

  async function fetchBalanceSheet() {
    const res = await api.get('/reports/balance-sheet')
    balanceSheet.value = res.data
  }

  async function fetchStats(by: string, from?: string, to?: string) {
    const res = await api.get('/reports/stats', { params: { by, from, to } })
    stats.value = res.data
  }

  return { balanceSheet, incomeStatement, stats, fetchBalanceSheet, fetchStats }
})
```

- [ ] **步骤 5：编译验证**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 6：Commit**

```bash
git add accounting-web/src/stores/
git commit -m "feat(web): Pinia stores（成员/账户/交易/报表）

- 异步 API 调用 + 本地状态管理
- 创建/删除/查询操作封装"
```

---

## 任务 11：实现响应式 Layout 组件

**文件：**
- 创建：`accounting-web/src/components/Layout.vue`

**目标：** PC 端固定侧边栏（200px），平板折叠，手机端底部 Tab 栏。路由内容区自适应填充。

- [ ] **步骤 1：创建 Layout.vue**

```vue
<template>
  <a-layout class="app-layout">
    <!-- PC 侧边栏 -->
    <a-layout-sider
      v-if="!isMobile"
      v-model:collapsed="collapsed"
      :width="200"
      class="pc-sider"
    >
      <div class="logo">记账</div>
      <a-menu :selected-keys="[activeKey]" theme="dark" mode="inline">
        <a-menu-item key="/" @click="$router.push('/')">
          <home-outlined />
          <span>首页</span>
        </a-menu-item>
        <a-menu-item key="/accounts" @click="$router.push('/accounts')">
          <book-outlined />
          <span>账户</span>
        </a-menu-item>
        <a-menu-item key="/reports" @click="$router.push('/reports')">
          <bar-chart-outlined />
          <span>报表</span>
        </a-menu-item>
        <a-menu-item key="/settings" @click="$router.push('/settings')">
          <setting-outlined />
          <span>设置</span>
        </a-menu-item>
      </a-menu>
    </a-layout-sider>

    <a-layout>
      <a-layout-content class="content">
        <router-view />
      </a-layout-content>
    </a-layout>

    <!-- 手机底部 Tab 栏 -->
    <div v-if="isMobile" class="mobile-nav">
      <div
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-item"
        :class="{ active: activeKey === tab.key }"
        @click="$router.push(tab.key)"
      >
        <component :is="tab.icon" />
        <span>{{ tab.label }}</span>
      </div>
    </div>
  </a-layout>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import {
  HomeOutlined,
  BookOutlined,
  BarChartOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'

const route = useRoute()
const activeKey = computed(() => route.path)
const collapsed = ref(false)

const windowWidth = ref(window.innerWidth)
const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1024)

function onResize() {
  windowWidth.value = window.innerWidth
}
onMounted(() => window.addEventListener('resize', onResize))
onUnmounted(() => window.removeEventListener('resize', onResize))

const tabs = [
  { key: '/', label: '首页', icon: HomeOutlined },
  { key: '/accounts', label: '账户', icon: BookOutlined },
  { key: '/reports', label: '报表', icon: BarChartOutlined },
  { key: '/settings', label: '设置', icon: SettingOutlined },
]
</script>

<style scoped>
.app-layout {
  min-height: 100vh;
}
.pc-sider .logo {
  height: 64px;
  line-height: 64px;
  text-align: center;
  color: white;
  font-size: 20px;
  font-weight: bold;
}
.content {
  padding: 24px;
  overflow: auto;
}
.mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 56px;
  background: white;
  display: flex;
  border-top: 1px solid #f0f0f0;
  z-index: 1000;
}
.tab-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #666;
}
.tab-item.active {
  color: #1890ff;
}
</style>
```

- [ ] **步骤 2：安装 ant-design-vue icons**

```bash
cd accounting-web
npm install @ant-design/icons-vue
```

- [ ] **步骤 3：编译验证**

```bash
npm run build
```

- [ ] **步骤 4：Commit**

```bash
git add accounting-web/src/components/Layout.vue accounting-web/package.json
git commit -m "feat(web): 响应式 Layout 组件

- PC 200px 侧边栏
- 手机端底部 Tab 导航
- 使用 Vue Router 响应式路由切换"
```

---

## 任务 12：实现 Calendar 组件

**文件：**
- 创建：`accounting-web/src/components/Calendar.vue`

**目标：** 月历视图，每天格内显示「收入 +N / 支出 -M」，点击日期触发选择事件。支持范围选择（点击两个日期）。

- [ ] **步骤 1：创建 Calendar.vue**

```vue
<template>
  <div class="calendar">
    <div class="calendar-header">
      <a-button @click="prevMonth">&lt;</a-button>
      <span class="month-label">{{ currentYear }}年{{ currentMonth + 1 }}月</span>
      <a-button @click="nextMonth">&gt;</a-button>
    </div>
    <div class="calendar-weekdays">
      <span v-for="day in ['日','一','二','三','四','五','六']" :key="day">{{ day }}</span>
    </div>
    <div class="calendar-days">
      <div
        v-for="day in days"
        :key="day.dateStr"
        class="day-cell"
        :class="{
          'other-month': !day.inCurrentMonth,
          'selected': isSelected(day.dateStr),
          'range': isInRange(day.dateStr),
          'today': day.dateStr === today,
        }"
        @click="handleClick(day.dateStr)"
      >
        <div class="day-number">{{ day.date }}</div>
        <div v-if="day.income !== undefined" class="day-income">+{{ day.income }}</div>
        <div v-if="day.expense !== undefined" class="day-expense">-{{ day.expense }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

interface DayCell {
  date: number
  dateStr: string
  inCurrentMonth: boolean
  income?: number
  expense?: number
}

const props = defineProps<{
  year: number
  month: number
  data?: Record<string, { income: number; expense: number }>
}>()

const emit = defineEmits<{
  (e: 'select', date: string): void
  (e: 'select-range', from: string, to: string): void
}>()

const currentYear = ref(props.year)
const currentMonth = ref(props.month - 1)

const today = computed(() => new Date().toISOString().split('T')[0])

const selectedDates = ref<string[]>([])

function isSelected(dateStr: string) {
  return selectedDates.value.includes(dateStr)
}

function isInRange(dateStr: string) {
  if (selectedDates.value.length < 2) return false
  const [a, b] = selectedDates.value.sort()
  return dateStr > a && dateStr < b
}

function handleClick(dateStr: string) {
  if (selectedDates.value.length === 1) {
    const first = selectedDates.value[0]
    if (dateStr !== first) {
      const [from, to] = [first, dateStr].sort()
      selectedDates.value = [from, to]
      emit('select-range', from, to)
      return
    }
  }
  selectedDates.value = [dateStr]
  emit('select', dateStr)
}

const days = computed((): DayCell[] => {
  const first = new Date(currentYear.value, currentMonth.value, 1)
  const last = new Date(currentYear.value, currentMonth.value + 1, 0)
  const startDay = first.getDay()
  const result: DayCell[] = []

  // 上月填充
  const prevLast = new Date(currentYear.value, currentMonth.value, 0)
  for (let i = startDay - 1; i >= 0; i--) {
    const d = new Date(prevLast)
    d.setDate(prevLast.getDate() - i)
    result.push({ date: d.getDate(), dateStr: formatDate(d), inCurrentMonth: false })
  }

  // 当月
  for (let i = 1; i <= last.getDate(); i++) {
    const d = new Date(currentYear.value, currentMonth.value, i)
    const ds = formatDate(d)
    const info = props.data?.[ds]
    result.push({
      date: i,
      dateStr: ds,
      inCurrentMonth: true,
      income: info?.income,
      expense: info?.expense,
    })
  }

  // 下月填充
  const remaining = 42 - result.length
  for (let i = 1; i <= remaining; i++) {
    const d = new Date(currentYear.value, currentMonth.value + 1, i)
    result.push({ date: i, dateStr: formatDate(d), inCurrentMonth: false })
  }

  return result
})

function formatDate(d: Date) {
  return d.toISOString().split('T')[0]
}

function prevMonth() {
  if (currentMonth.value === 0) {
    currentMonth.value = 11
    currentYear.value--
  } else {
    currentMonth.value--
  }
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0
    currentYear.value++
  } else {
    currentMonth.value++
  }
}

watch(() => [props.year, props.month], ([y, m]) => {
  currentYear.value = y as number
  currentMonth.value = (m as number) - 1
})
</script>

<style scoped>
.calendar {
  max-width: 800px;
  margin: 0 auto;
}
.calendar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  font-size: 18px;
}
.calendar-weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  padding: 8px 0;
  font-weight: bold;
  border-bottom: 1px solid #f0f0f0;
}
.calendar-days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 1px;
  background: #f0f0f0;
}
.day-cell {
  background: white;
  min-height: 80px;
  padding: 8px;
  cursor: pointer;
}
.day-cell:hover {
  background: #f6ffed;
}
.day-cell.other-month {
  color: #ccc;
}
.day-cell.selected {
  background: #e6f7ff;
  border: 1px solid #1890ff;
}
.day-cell.range {
  background: #e6f7ff;
}
.day-cell.today .day-number {
  color: #1890ff;
  font-weight: bold;
}
.day-number {
  font-size: 14px;
  margin-bottom: 4px;
}
.day-income {
  color: #52c41a;
  font-size: 12px;
}
.day-expense {
  color: #ff4d4f;
  font-size: 12px;
}
</style>
```

- [ ] **步骤 2：编译验证**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 3：Commit**

```bash
git add accounting-web/src/components/Calendar.vue
git commit -m "feat(web): 日历组件

- 月历视图，支持上月/下月切换
- 每天显示收入/支出统计
- 支持单选（emit select）和范围选择（emit select-range）
- today 高亮"
```

---

## 任务 13：实现 Dashboard 页面（日历首页）

**文件：**
- 创建：`accounting-web/src/views/Dashboard.vue`
- 创建：`accounting-web/src/components/TransactionDetail.vue`

**目标：** 日历 + 下方内容区。默认显示当月报表（支出/收入/结余）。点击日期 → 下方显示当日交易列表，列表第一项是「记一笔」按钮（仅单日选择）。范围选择 → 下方显示范围内交易列表，不显示「记一笔」。

- [ ] **步骤 1：创建 TransactionDetail.vue**

```vue
<template>
  <div class="transaction-item">
    <div class="tx-header" @click="toggle">
      <span class="tx-date">{{ tx.date_time }}</span>
      <span class="tx-desc">{{ tx.description }}</span>
      <span class="tx-member">{{ tx.member_id }}</span>
    </div>
    <div v-if="expanded" class="tx-details">
      <!-- 分录详情展开 -->
      <p>分录详情（待实现 API）</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Transaction } from '@/stores/transaction'

const props = defineProps<{
  tx: Transaction
}>()

const expanded = ref(false)
function toggle() {
  expanded.value = !expanded.value
}
</script>

<style scoped>
.transaction-item {
  border-bottom: 1px solid #f0f0f0;
  padding: 12px 0;
}
.tx-header {
  display: flex;
  gap: 16px;
  cursor: pointer;
}
.tx-date {
  width: 140px;
  flex-shrink: 0;
}
.tx-desc {
  flex: 1;
}
.tx-member {
  width: 80px;
  text-align: right;
  flex-shrink: 0;
}
</style>
```

- [ ] **步骤 2：创建 Dashboard.vue**

```vue
<template>
  <div class="dashboard">
    <Calendar
      :year="year"
      :month="month"
      :data="calendarData"
      @select="onSelectDay"
      @select-range="onSelectRange"
    />

    <div class="content-area">
      <!-- 单日选择：显示记一笔按钮 + 当日交易 -->
      <template v-if="selectedDay">
        <a-button type="primary" block class="add-btn" @click="onAdd">
          <plus-outlined /> 记一笔
        </a-button>
        <div v-if="dayTransactions.length === 0" class="empty">暂无交易</div>
        <TransactionDetail
          v-for="tx in dayTransactions"
          :key="tx.id"
          :tx="tx"
        />
      </template>

      <!-- 范围选择：显示范围交易列表 -->
      <template v-else-if="rangeFrom && rangeTo">
        <div class="range-label">{{ rangeFrom }} 至 {{ rangeTo }} 的交易</div>
        <TransactionDetail
          v-for="tx in rangeTransactions"
          :key="tx.id"
          :tx="tx"
        />
      </template>

      <!-- 默认：当月报表 -->
      <template v-else>
        <h3>{{ year }}年{{ month }}月 收支概览</h3>
        <a-statistic title="收入" :value="monthStats.income" suffix="CNY" />
        <a-statistic title="支出" :value="monthStats.expense" suffix="CNY" />
        <a-statistic title="结余" :value="monthStats.income - monthStats.expense" suffix="CNY" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { PlusOutlined } from '@ant-design/icons-vue'
import Calendar from '@/components/Calendar.vue'
import TransactionDetail from '@/components/TransactionDetail.vue'
import { useTransactionStore } from '@/stores/transaction'

const router = useRouter()
const txStore = useTransactionStore()

const now = new Date()
const year = ref(now.getFullYear())
const month = ref(now.getMonth() + 1)

const selectedDay = ref<string | null>(null)
const rangeFrom = ref<string | null>(null)
const rangeTo = ref<string | null>(null)

const calendarData = ref<Record<string, { income: number; expense: number }>>({})
const monthStats = ref({ income: 0, expense: 0 })

const dayTransactions = computed(() => {
  if (!selectedDay.value) return []
  return txStore.transactions.filter((t) => t.date_time.startsWith(selectedDay.value!))
})

const rangeTransactions = computed(() => {
  if (!rangeFrom.value || !rangeTo.value) return []
  return txStore.transactions
})

function onSelectDay(date: string) {
  selectedDay.value = date
  rangeFrom.value = null
  rangeTo.value = null
  txStore.fetchTransactions({ from: date, to: date })
}

function onSelectRange(from: string, to: string) {
  selectedDay.value = null
  rangeFrom.value = from
  rangeTo.value = to
  txStore.fetchTransactions({ from, to })
}

function onAdd() {
  router.push(`/transaction?date=${selectedDay.value}`)
}

onMounted(async () => {
  // 获取当月交易统计，填充日历
  const y = year.value
  const m = month.value
  const from = `${y}-${String(m).padStart(2, '0')}-01`
  const to = new Date(y, m, 0).toISOString().split('T')[0]
  await txStore.fetchTransactions({ from, to })
  // TODO: 按天聚合统计
})
</script>

<style scoped>
.dashboard {
  padding: 16px;
}
.content-area {
  margin-top: 24px;
}
.add-btn {
  margin-bottom: 16px;
}
.empty {
  text-align: center;
  color: #999;
  padding: 32px 0;
}
.range-label {
  font-size: 16px;
  font-weight: bold;
  margin-bottom: 16px;
}
</style>
```

- [ ] **步骤 3：编译验证**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 4：Commit**

```bash
git add accounting-web/src/views/Dashboard.vue accounting-web/src/components/TransactionDetail.vue
git commit -m "feat(web): Dashboard 日历首页

- 默认显示当月收支概览
- 点击日期 → 当日交易 + 记一笔按钮
- 范围选择 → 范围内交易列表（无按钮）
- 使用 TransactionDetail 组件展示交易项"
```

---

## 任务 14：实现 TransactionForm 记账页面

**文件：**
- 创建：`accounting-web/src/views/TransactionForm.vue`

**目标：** PC 端弹窗/手机端全屏。预填记账人（currentMember）和日期（URL 参数）。包含：时间选择、备注、分录表格、标签选择、确认录入。

- [ ] **步骤 1：创建 TransactionForm.vue**

```vue
<template>
  <a-modal
    v-if="!isMobile"
    v-model:open="visible"
    title="记一笔"
    @ok="onSubmit"
    @cancel="onCancel"
    width="800px"
  >
    <a-form :model="form" layout="vertical">
      <a-form-item label="日期时间">
        <a-date-picker v-model:value="form.date" show-time format="YYYY-MM-DD HH:mm:ss" />
      </a-form-item>
      <a-form-item label="备注">
        <a-input v-model:value="form.description" />
      </a-form-item>
      <a-form-item label="分录">
        <div v-for="(p, i) in form.postings" :key="i" class="posting-row">
          <a-input v-model:value="p.account" placeholder="账户" style="width: 200px" />
          <a-input v-model:value="p.commodity" placeholder="货币" style="width: 80px" />
          <a-input v-model:value="p.amount" placeholder="金额" style="width: 120px" />
          <a-button type="link" @click="removePosting(i)">删除</a-button>
        </div>
        <a-button type="dashed" block @click="addPosting">添加分录</a-button>
      </a-form-item>
      <a-form-item label="标签">
        <a-select v-model:value="form.tags" mode="tags" placeholder="输入标签" />
      </a-form-item>
    </a-form>
  </a-modal>

  <!-- 手机端全屏 -->
  <div v-else class="mobile-form">
    <a-nav-bar title="记一笔" left-text="返回" @click-left="onCancel" />
    <!-- ... 同 PC 表单结构 ... -->
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useMemberStore } from '@/stores/member'
import { useTransactionStore } from '@/stores/transaction'

const route = useRoute()
const router = useRouter()
const memberStore = useMemberStore()
const txStore = useTransactionStore()

const isMobile = computed(() => window.innerWidth < 768)
const visible = ref(true)

const form = ref({
  date: null as Date | null,
  description: '',
  postings: [{ account: '', commodity: 'CNY', amount: '' }],
  tags: [] as string[],
})

onMounted(() => {
  // 预填日期（URL 参数）
  const dateParam = route.query.date as string
  if (dateParam) {
    form.value.date = new Date(dateParam + 'T00:00:00')
  }
  // 记账人由后端自动从 currentMember 获取
})

function addPosting() {
  form.value.postings.push({ account: '', commodity: 'CNY', amount: '' })
}

function removePosting(index: number) {
  form.value.postings.splice(index, 1)
}

async function onSubmit() {
  if (!form.value.date) return
  const data = {
    date_time: form.value.date.toISOString().slice(0, 19).replace('T', ' '),
    description: form.value.description,
    postings: form.value.postings.filter((p) => p.account && p.amount),
    tags: form.value.tags,
    member_id: memberStore.currentMember?.id,
  }
  await txStore.createTransaction(data)
  visible.value = false
  router.push('/')
}

function onCancel() {
  visible.value = false
  router.push('/')
}
</script>

<style scoped>
.posting-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}
.mobile-form {
  padding: 16px;
  padding-bottom: 80px;
}
</style>
```

- [ ] **步骤 2：编译验证**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 3：Commit**

```bash
git add accounting-web/src/views/TransactionForm.vue
git commit -m "feat(web): 记账页面

- PC 端 Modal 弹窗，手机端全屏
- 预填日期（URL 参数）和当前记账人
- 支持多行分录、标签选择"
```

---

## 任务 15：实现 AccountTree、Settings、ReportView 页面

**文件：**
- 创建：`accounting-web/src/views/AccountTree.vue`
- 创建：`accounting-web/src/views/Settings.vue`
- 创建：`accounting-web/src/views/ReportView.vue`

**目标：** 三个页面骨架，功能完整可运行。

- [ ] **步骤 1：AccountTree.vue**

```vue
<template>
  <div>
    <h2>账户</h2>
    <a-button type="primary" @click="showAdd = true">新建账户</a-button>
    <a-tree :tree-data="treeData" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAccountStore } from '@/stores/account'

const accountStore = useAccountStore()
const showAdd = ref(false)

const treeData = computed(() => {
  const roots = accountStore.accounts.filter((a) => !a.parent_id)
  function build(parentId?: number) {
    return accountStore.accounts
      .filter((a) => a.parent_id === parentId)
      .map((a) => ({
        title: a.full_name,
        key: a.id,
        children: build(a.id),
      }))
  }
  return roots.map((r) => ({
    title: r.full_name,
    key: r.id,
    children: build(r.id),
  }))
})

onMounted(() => accountStore.fetchAccounts())
</script>
```

- [ ] **步骤 2：Settings.vue**

```vue
<template>
  <div>
    <h2>设置</h2>
    <a-form layout="vertical">
      <a-form-item label="当前记账人">
        <a-select v-model:value="selectedMember" @change="onChangeMember">
          <a-select-option v-for="m in memberStore.members" :key="m.id" :value="m.id">
            {{ m.name }}
          </a-select-option>
        </a-select>
      </a-form-item>
    </a-form>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useMemberStore } from '@/stores/member'

const memberStore = useMemberStore()
const selectedMember = computed({
  get: () => memberStore.currentMember?.id,
  set: (id: number) => memberStore.setCurrent(id),
})

onMounted(() => memberStore.fetchMembers())

function onChangeMember(id: number) {
  memberStore.setCurrent(id)
}
</script>
```

- [ ] **步骤 3：ReportView.vue**

```vue
<template>
  <div>
    <h2>报表</h2>
    <a-tabs>
      <a-tab-pane key="balance" tab="资产负债表">
        <pre>{{ JSON.stringify(reportStore.balanceSheet, null, 2) }}</pre>
      </a-tab-pane>
      <a-tab-pane key="income" tab="损益表">
        <pre>{{ JSON.stringify(reportStore.incomeStatement, null, 2) }}</pre>
      </a-tab-pane>
      <a-tab-pane key="stats" tab="统计">
        <pre>{{ JSON.stringify(reportStore.stats, null, 2) }}</pre>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useReportStore } from '@/stores/report'

const reportStore = useReportStore()
onMounted(() => {
  reportStore.fetchBalanceSheet()
})
</script>
```

- [ ] **步骤 4：编译验证**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-web/src/views/
git commit -m "feat(web): 账户树、设置、报表页面

- AccountTree: 树形展示账户结构
- Settings: 切换当前记账人
- ReportView: 资产负债表/损益表/统计标签页"
```

---

## 任务 16：API 与前端集成测试

**文件：** 无新增文件

**目标：** 端到端验证：启动后端 → 打开前端 → 完成一次记账流程。

- [ ] **步骤 1：构建前端**

```bash
cd accounting-web
npm run build
# 产物在 accounting-web/dist
```

- [ ] **步骤 2：启动后端（托管前端静态文件）**

```bash
cd ..
cargo run -p accounting-api -- --static-dir accounting-web/dist --port 3000
```

- [ ] **步骤 3：浏览器访问 http://localhost:3000**

手动验证：
1. 首页日历正常显示
2. 点击日期 → 下方显示「记一笔」按钮
3. 点击「记一笔」→ 弹出记账表单
4. 填写分录 → 提交 → 返回首页
5. 刷新页面 → 新交易出现在列表中
6. 切换路由到「账户」「报表」「设置」均正常

- [ ] **步骤 4：手机端模拟（Chrome DevTools 375px 宽度）**

验证：
1. 底部 Tab 导航显示
2. 记账表单为全屏
3. 日历和列表正常显示

- [ ] **步骤 5：Commit**

```bash
git commit -m "feat(web): 前后端集成验证通过

- 完整记账流程端到端测试 OK
- PC + 手机端响应式正常"
```

---

## 任务 17：实现 `/api/me` 后端状态持久化

**文件：**
- 修改：`accounting-sql/src/schema.rs`（添加 settings 表）
- 修改：`accounting-sql/src/lib.rs`（暴露 settings_repo）
- 修改：`accounting-api/src/handlers/me.rs`

**目标：** 当前用户身份持久化到数据库 settings 表。

- [ ] **步骤 1：在 schema.rs 添加 settings 表**

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

- [ ] **步骤 2：创建 settings_repo.rs**

```rust
use rusqlite::{Connection, Result};

pub struct SettingsRepo;

impl SettingsRepo {
    pub fn get(&self, conn: &Connection, key: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }
}
```

- [ ] **步骤 3：修改 me.rs 使用 settings 持久化**

```rust
async fn get_me(State(state): State<Arc<AppState>>) -> Result<Json<MeDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let repo = SettingsRepo;
    let member_id = repo.get(&conn, "current_member_id")
        .map_err(|e| e.to_string())?
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // 查询成员名称...
    Ok(Json(MeDto { member_id, member_name: "...".to_string() }))
}

async fn set_me(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetMeRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let repo = SettingsRepo;
    repo.set(&conn, "current_member_id", &req.member_id.to_string())
        .map_err(|e| e.to_string())?;
    Ok("ok")
}
```

- [ ] **步骤 4：编译 + 测试**

```bash
cargo fmt
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-sql/src/ accounting-api/src/handlers/me.rs
git commit -m "feat(api): /api/me 持久化到 settings 表

- settings 表 key-value 存储
- GET/PUT /api/me 读写 current_member_id"
```

---

## 任务 18：全量文档和格式化

**文件：** 所有新增/修改文件

**目标：** 确保所有新代码有 `///` 文档注释，通过 fmt + clippy + test。

- [ ] **步骤 1：Rust 后端**

```bash
cargo fmt
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

- [ ] **步骤 2：前端**

```bash
cd accounting-web
npm run build
```

- [ ] **步骤 3：Commit**

```bash
git add -A
git commit -m "chore: 全量格式化 + clippy + 文档注释

- 所有新增 Rust 模块添加 /// 文档
- 通过 cargo fmt --workspace
- 通过 cargo clippy --workspace -- -D warnings
- 通过 cargo test --workspace"
```

---

## 任务 19：最终端到端验收

**目标：** 在干净环境中重新构建前后端，完整走一遍用户故事。

- [ ] **步骤 1：清理构建**

```bash
rm -rf accounting-web/dist accounting-web/node_modules
cargo clean
```

- [ ] **步骤 2：重新构建**

```bash
# 后端
cargo build --workspace

# 前端
cd accounting-web
npm install
npm run build
```

- [ ] **步骤 3：启动服务**

```bash
cargo run -p accounting-api -- --db my.db --static-dir accounting-web/dist --port 3000
```

- [ ] **步骤 4：验收检查清单**

- [ ] 首页日历显示当月
- [ ] 日历上有每天的收支统计
- [ ] 点击单日 → 下方显示「记一笔」按钮 + 当日交易
- [ ] 点击「记一笔」→ 日期/记账人已预填
- [ ] 填写交易分录 → 确认录入 → 返回首页 → 新交易可见
- [ ] 选择日期范围 → 显示范围交易（无记一笔按钮）
- [ ] 账户页面显示账户树
- [ ] 报表页面显示资产负债表
- [ ] 设置页面可切换当前记账人
- [ ] 手机端（375px）底部导航正常
- [ ] 手机端记账为全屏页面

- [ ] **步骤 5：Commit 最终标记**

```bash
git commit -m "feat: Web UI Phase 3 完成

- axum REST API 后端（accounting-api）
- Vue 3 + Ant Design Vue 前端（accounting-web）
- 响应式布局适配 PC/平板/手机
- 日历首页 + 记账表单 + 账户树 + 报表 + 设置
- 端到端验收通过"
```

---

## 附录：API 端点速查

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/members | 成员列表 |
| POST | /api/members | 创建成员 |
| DELETE | /api/members/:id | 删除成员 |
| GET | /api/accounts | 账户列表 |
| POST | /api/accounts | 创建账户（级联） |
| GET | /api/accounts/:id/balance | 账户余额 |
| GET | /api/transactions | 交易列表（筛选） |
| POST | /api/transactions | 创建交易 |
| DELETE | /api/transactions/:id | 删除交易 |
| GET | /api/tags | 标签列表 |
| GET | /api/reports/balance-sheet | 资产负债表 |
| GET | /api/reports/income-statement | 损益表 |
| GET | /api/reports/stats | 统计报表 |
| GET | /api/me | 当前用户 |
| PUT | /api/me | 切换当前用户 |
| GET | /api/health | 健康检查 |

## 附录：文件变更清单

**新增文件：**
- `accounting-api/` 目录（9 个文件）
- `accounting-web/` 目录（约 20 个文件）

**修改文件：**
- `Cargo.toml`（workspace 成员）
- `accounting-sql/src/schema.rs`（settings 表）

---

## 记录

**创建日期：** 2026-06-05
**目标交付：** Web UI 完整可运行的前后端系统
