<script setup lang="ts">
import { computed, inject, onMounted, ref, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import { fetchDailySummary } from '../api/client'
import CalendarGrid from '../components/CalendarGrid.vue'
import TransactionList from '../components/TransactionList.vue'
import TransactionFormOverlay from '../components/layout/TransactionFormOverlay.vue'
import { panelActionKey } from '../components/layout/panelAction'
import { useTransactionStore } from '../stores/transaction'
import type { DailySummaryDto } from '../types/api'
import { todayStr } from '../utils/date'

const txStore = useTransactionStore()
const { t } = useI18n()

const selectedDate = ref<string>('')

const showFormOverlay = ref(false)
const editingTxId = ref<number | undefined>(undefined)

const dailyStats = ref<Record<string, DailySummaryDto>>({})
const visibleRange = ref<{ from: string; to: string } | null>(null)

async function loadDailyStats() {
  if (!visibleRange.value) return
  const { from, to } = visibleRange.value
  const items = await fetchDailySummary(from, to)
  const record: Record<string, DailySummaryDto> = {}
  for (const item of items) {
    record[item.date] = item
  }
  dailyStats.value = record
}

function onVisibleRangeChange(from: string, to: string) {
  visibleRange.value = { from, to }
  loadDailyStats()
}

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
    txStore.remove(id).then(() => loadDailyStats())
  }
}

function onFormClosed() {
  showFormOverlay.value = false
  editingTxId.value = undefined
}

function onFormSaved() {
  // Data is already updated via create/update in store
  loadDailyStats()
}

const panelAction = inject(panelActionKey, null)
watchEffect(() => {
  if (!panelAction) return
  panelAction.value = showFormOverlay.value
    ? null
    : { label: t('calendar.newTransaction'), disabled: false, onClick: onNewTx }
})
</script>

<template>
  <div class="calendar-view" :class="{ 'no-scroll': showFormOverlay }">
    <template v-if="!showFormOverlay">
      <CalendarGrid
        :daily-stats="dailyStats"
        :selected-date="selectedDate"
        @select-date="onSelectDate"
        @visible-range-change="onVisibleRangeChange"
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

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
