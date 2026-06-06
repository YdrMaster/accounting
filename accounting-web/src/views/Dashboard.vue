<template>
  <div class="dashboard">
    <!-- 工具栏：toggle + 筛选 -->
    <div class="toolbar">
      <a-space>
        <span>范围选择</span>
        <a-switch v-model:checked="rangeMode" />
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
      :range-mode="rangeMode"
      @select="handleSelect"
      @select-range="handleSelectRange"
      @clear="handleClear"
    />

    <!-- 默认状态：当月收支概览 -->
    <div v-if="!selectedDate && !rangeFrom && !hasFilter" class="section overview">
      <h3>当月收支概览</h3>
      <div class="stats-row">
        <a-statistic title="收入" :value="monthlyIncome" prefix="¥" />
        <a-statistic title="支出" :value="monthlyExpense" prefix="¥" />
        <a-statistic title="结余" :value="monthlyIncome - monthlyExpense" prefix="¥" />
      </div>
    </div>

    <!-- 单日选择 -->
    <div v-else-if="selectedDate && !rangeFrom" class="section">
      <a-button type="primary" block class="action-btn" @click="goToTransaction">
        记一笔
      </a-button>
      <h3>{{ selectedDate }} 交易</h3>
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="filteredTransactions.length === 0" class="empty">暂无交易</div>
      <TransactionDetail
        v-for="tx in filteredTransactions"
        :key="tx.id"
        :tx="tx"
      />
    </div>

    <!-- 范围选择 -->
    <div v-else-if="rangeFrom && rangeTo" class="section">
      <a-tag color="blue" class="range-tag">{{ rangeFrom }} 至 {{ rangeTo }}</a-tag>
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="filteredTransactions.length === 0" class="empty">暂无交易</div>
      <TransactionDetail
        v-for="tx in filteredTransactions"
        :key="tx.id"
        :tx="tx"
      />
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import dayjs from 'dayjs'
import { FilterOutlined } from '@ant-design/icons-vue'
import Calendar from '@/components/Calendar.vue'
import TransactionDetail from '@/components/TransactionDetail.vue'
import { useTransactionStore } from '@/stores/transaction'
import { useAccountStore } from '@/stores/account'
import { useMemberStore } from '@/stores/member'
import api from '@/api/client'

const router = useRouter()
const transactionStore = useTransactionStore()
const accountStore = useAccountStore()
const memberStore = useMemberStore()

const rangeMode = ref(false)
const selectedDate = ref<string | null>(null)
const rangeFrom = ref<string | null>(null)
const rangeTo = ref<string | null>(null)
const showFilter = ref(false)

const tags = ref<{ id: number; name: string }[]>([])

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

const monthlyIncome = computed(() => 0)
const monthlyExpense = computed(() => 0)

const calendarData = computed(() => {
  return {} as Record<string, { income: number; expense: number }>
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
  fetchData()
}

function handleSelectRange(from: string, to: string) {
  selectedDate.value = null
  rangeFrom.value = from
  rangeTo.value = to
  fetchData()
}

function handleClear() {
  selectedDate.value = null
  rangeFrom.value = null
  rangeTo.value = null
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
  return params
}

function fetchData() {
  transactionStore.fetchTransactions(buildParams())
}

function fetchMonthData() {
  const start = dayjs().startOf('month').format('YYYY-MM-DD')
  const end = dayjs().endOf('month').format('YYYY-MM-DD')
  transactionStore.fetchTransactions({ from: start, to: end })
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

onMounted(() => {
  fetchMonthData()
  accountStore.fetchAccounts()
  memberStore.fetchMembers()
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

.loading,
.empty {
  text-align: center;
  color: #999;
  padding: 24px;
}
</style>
