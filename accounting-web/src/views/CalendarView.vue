<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useTransactionStore } from '../stores/transaction'
import CalendarGrid from '../components/CalendarGrid.vue'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'

const txStore = useTransactionStore()

const currentYear = ref(0)
const currentMonth = ref(0)
const selectedDate = ref<string | null>(null)

const showFormOverlay = ref(false)
const editingTxId = ref<number | undefined>(undefined)

onMounted(async () => {
  const now = new Date()
  currentYear.value = now.getFullYear()
  currentMonth.value = now.getMonth() + 1
  await txStore.loadMonth(currentYear.value, currentMonth.value)
  // If current month has no data, auto-load previous month
  const currentData = txStore.getMonthTransactions(currentYear.value, currentMonth.value)
  if (currentData.length === 0) {
    if (currentMonth.value === 1) {
      currentYear.value = currentYear.value - 1
      currentMonth.value = 12
    } else {
      currentMonth.value = currentMonth.value - 1
    }
    await txStore.loadMonth(currentYear.value, currentMonth.value)
    // Select last day of loaded month
    const lastDay = new Date(currentYear.value, currentMonth.value, 0).getDate()
    selectedDate.value = `${currentYear.value}-${String(currentMonth.value).padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`
  } else {
    // Default to today's date selected
    selectedDate.value = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
  }
})

const transactionDates = computed(() => {
  const dates = new Set<string>()
  for (const tx of txStore.transactions) {
    dates.add(tx.date_time.slice(0, 10))
  }
  return dates
})

const filteredTransactions = computed(() => {
  if (!selectedDate.value) return txStore.transactions
  return txStore.transactions.filter(tx => tx.date_time.slice(0, 10) === selectedDate.value)
})

function onSelectDate(date: string) {
  if (selectedDate.value === date) {
    selectedDate.value = null
  } else {
    selectedDate.value = date
  }
}

function onNewTx() {
  editingTxId.value = undefined
  showFormOverlay.value = true
}

function onEditTx(id: number) {
  editingTxId.value = id
  showFormOverlay.value = true
}

function onDeleteTx(id: number) {
  if (confirm('确定要删除这条交易吗？')) {
    txStore.remove(id)
  }
}

function onFormClosed() {
  showFormOverlay.value = false
  editingTxId.value = undefined
}

function onFormSaved() {
  txStore.loadMonth(currentYear.value, currentMonth.value, true)
}
</script>

<template>
  <div class="calendar-view" :class="{ 'no-scroll': showFormOverlay }">
    <!-- Show normal calendar view when form is not displayed -->
    <template v-if="!showFormOverlay">
      <div class="header-actions">
        <button class="new-tx-btn" @click="onNewTx">+ 新建交易</button>
      </div>

      <CalendarGrid
        :transaction-dates="transactionDates"
        :selected-date="selectedDate"
        @select-date="onSelectDate"
      />

      <div v-if="selectedDate" class="selected-date-label">
        {{ selectedDate }} 的交易
        <button class="clear-date-btn" @click="selectedDate = null">清除</button>
      </div>

      <div v-if="txStore.loading" class="loading">加载中...</div>
      <div v-else-if="txStore.error" class="error">{{ txStore.error }}</div>

      <TransactionList
        :transactions="filteredTransactions"
        @edit="onEditTx"
        @delete="onDeleteTx"
      />
    </template>

    <!-- Show form overlay when editing/creating - completely replaces calendar view -->
    <TransactionFormOverlay
      v-if="showFormOverlay"
      :edit-id="editingTxId"
      @close="onFormClosed"
      @saved="onFormSaved"
    />
  </div>
</template>

<style scoped>
.calendar-view {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
  height: 100%;
  overflow-y: auto;
}

.calendar-view.no-scroll {
  overflow: hidden;
}

.header-actions {
  display: flex;
  justify-content: flex-end;
}

.new-tx-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.new-tx-btn:hover {
  opacity: 0.9;
}

.selected-date-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-heading);
  font-size: 0.875rem;
  font-weight: 500;
  padding: 0.5rem 0.75rem;
  background: var(--card-bg-alt, #252525);
  border-radius: 0.5rem;
}

.clear-date-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
}

.clear-date-btn:hover {
  color: var(--text-heading);
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
