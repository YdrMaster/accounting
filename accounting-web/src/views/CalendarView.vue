<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CalendarGrid from '../components/CalendarGrid.vue'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'
import { useTransactionStore } from '../stores/transaction'
import { todayStr } from '../utils/date'

const txStore = useTransactionStore()
const { t } = useI18n()

const selectedDate = ref<string>('')

const showFormOverlay = ref(false)
const editingTxId = ref<number | undefined>(undefined)

onMounted(async () => {
  await txStore.loadInitial(todayStr(), 100)
  if (txStore.loadedRange) {
    selectedDate.value = txStore.loadedRange.to
  } else {
    selectedDate.value = todayStr()
  }
})

const filteredTransactions = computed(() => {
  return txStore.transactions.filter(tx => tx.date_time.slice(0, 10) === selectedDate.value)
})

function onSelectDate(date: string) {
  selectedDate.value = date
  txStore.loadDay(date)
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
  if (confirm(t('calendar.confirmDelete'))) {
    txStore.remove(id)
  }
}

function onFormClosed() {
  showFormOverlay.value = false
  editingTxId.value = undefined
}

function onFormSaved() {
  // Data is already updated via create/update in store
}
</script>

<template>
  <div class="calendar-view" :class="{ 'no-scroll': showFormOverlay }">
    <template v-if="!showFormOverlay">
      <div class="header-actions">
        <button class="new-tx-btn" @click="onNewTx">+ {{ t('calendar.newTransaction') }}</button>
      </div>

      <CalendarGrid
        :transaction-dates="txStore.transactionDates"
        :selected-date="selectedDate"
        @select-date="onSelectDate"
      />

      <div v-if="txStore.loading && filteredTransactions.length === 0" class="loading">
        {{ t('common.loading') }}
      </div>
      <div v-else-if="txStore.error" class="error">{{ txStore.error }}</div>

      <TransactionList :transactions="filteredTransactions" @edit="onEditTx" @delete="onDeleteTx" />
    </template>

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
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.calendar-view.no-scroll {
  overflow: hidden;
}

.calendar-view::-webkit-scrollbar {
  display: none;
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

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
