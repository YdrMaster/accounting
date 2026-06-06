<template>
  <div class="report-view">
    <a-tabs v-model:activeKey="activeKey">
      <a-tab-pane key="balance" tab="资产负债表">
        <div v-if="reportStore.balanceSheet" class="report-content">
          <a-list :data-source="balanceSheetList" bordered>
            <template #renderItem="{ item }">
              <a-list-item>
                <a-list-item-meta :title="item.title" />
                <template #actions>
                  <span>{{ item.value }}</span>
                </template>
              </a-list-item>
            </template>
          </a-list>
        </div>
        <div v-else class="empty">加载中...</div>
      </a-tab-pane>

      <a-tab-pane key="income" tab="损益表">
        <div v-if="reportStore.incomeStatement" class="report-content">
          <a-list :data-source="incomeStatementList" bordered>
            <template #renderItem="{ item }">
              <a-list-item>
                <a-list-item-meta :title="item.title" />
                <template #actions>
                  <span>{{ item.value }}</span>
                </template>
              </a-list-item>
            </template>
          </a-list>
        </div>
        <div v-else class="empty">加载中...</div>
      </a-tab-pane>

      <a-tab-pane key="stats" tab="统计">
        <div class="stats-filter">
          <a-select v-model:value="statsBy" style="width: 120px" placeholder="维度">
            <a-select-option value="tag">标签</a-select-option>
            <a-select-option value="member">成员</a-select-option>
            <a-select-option value="channel">渠道</a-select-option>
          </a-select>
          <a-range-picker v-model:value="statsRange" />
          <a-button type="primary" @click="fetchStats">查询</a-button>
        </div>
        <div v-if="reportStore.stats" class="report-content">
          <pre>{{ JSON.stringify(reportStore.stats, null, 2) }}</pre>
        </div>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Dayjs } from 'dayjs'
import { useReportStore } from '@/stores/report'

const reportStore = useReportStore()
const activeKey = ref('balance')
const statsBy = ref<'tag' | 'member' | 'channel'>('tag')
const statsRange = ref<[Dayjs, Dayjs] | null>(null)

interface ListItem {
  title: string
  value: string
}

function flattenReport(data: unknown, prefix = ''): ListItem[] {
  if (data === null || data === undefined) return []
  if (typeof data !== 'object') return [{ title: prefix, value: String(data) }]
  if (Array.isArray(data)) {
    return data.flatMap((item, i) => flattenReport(item, `${prefix}[${i}]`))
  }
  const obj = data as Record<string, unknown>
  return Object.entries(obj).flatMap(([key, value]) => {
    const newKey = prefix ? `${prefix} / ${key}` : key
    if (typeof value === 'number') {
      return [{ title: newKey, value: String(value) }]
    }
    if (typeof value === 'string') {
      return [{ title: newKey, value }]
    }
    return flattenReport(value, newKey)
  })
}

const balanceSheetList = computed<ListItem[]>(() =>
  flattenReport(reportStore.balanceSheet)
)

const incomeStatementList = computed<ListItem[]>(() =>
  flattenReport(reportStore.incomeStatement)
)

watch(
  activeKey,
  (key) => {
    if (key === 'balance') {
      reportStore.fetchBalanceSheet()
    } else if (key === 'income') {
      reportStore.fetchIncomeStatement()
    }
  },
  { immediate: true }
)

function fetchStats() {
  if (!statsRange.value) {
    reportStore.fetchStats(statsBy.value)
  } else {
    reportStore.fetchStats(
      statsBy.value,
      statsRange.value[0].format('YYYY-MM-DD'),
      statsRange.value[1].format('YYYY-MM-DD')
    )
  }
}
</script>

<style scoped>
.report-view {
  background: #fff;
  padding: 24px;
  border-radius: 8px;
}

.report-content {
  margin-top: 16px;
}

.stats-filter {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.empty {
  text-align: center;
  color: #999;
  padding: 48px;
}
</style>
