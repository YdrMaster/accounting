<template>
  <div class="dashboard">
    <Calendar :data="calendarData" @select="handleSelect" @select-range="handleSelectRange" />

    <!-- 默认状态：当月收支概览 -->
    <div v-if="!selectedDate && !rangeFrom" class="section overview">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import dayjs from 'dayjs'
import Calendar from '@/components/Calendar.vue'
import TransactionDetail from '@/components/TransactionDetail.vue'
import { useTransactionStore } from '@/stores/transaction'

const router = useRouter()
const transactionStore = useTransactionStore()

const selectedDate = ref<string | null>(null)
const rangeFrom = ref<string | null>(null)
const rangeTo = ref<string | null>(null)

const loading = computed(() => transactionStore.loading)
const filteredTransactions = computed(() => transactionStore.transactions)

// TODO: 待 API 增强后按收入/支出账户类型聚合统计
const monthlyIncome = computed(() => 0)
const monthlyExpense = computed(() => 0)

const calendarData = computed(() => {
  // TODO: 按日期聚合交易数据（需 API 增强）
  return {} as Record<string, { income: number; expense: number }>
})

function handleSelect(date: string) {
  selectedDate.value = date
  rangeFrom.value = null
  rangeTo.value = null
  transactionStore.fetchTransactions({ date })
}

function handleSelectRange(from: string, to: string) {
  selectedDate.value = null
  rangeFrom.value = from
  rangeTo.value = to
  transactionStore.fetchTransactions({ from, to })
}

function goToTransaction() {
  if (selectedDate.value) {
    router.push(`/transaction?date=${selectedDate.value}`)
  }
}

onMounted(() => {
  const start = dayjs().startOf('month').format('YYYY-MM-DD')
  const end = dayjs().endOf('month').format('YYYY-MM-DD')
  transactionStore.fetchTransactions({ from: start, to: end })
})
</script>

<style scoped>
.dashboard {
  max-width: 800px;
  margin: 0 auto;
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
