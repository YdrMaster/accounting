<template>
  <a-config-provider :theme="{ algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm }">
    <Layout />
  </a-config-provider>
</template>

<script setup lang="ts">
import { theme } from 'ant-design-vue'
import Layout from './components/Layout.vue'
import { useThemeStore } from './stores/theme'

const themeStore = useThemeStore()
const isDark = themeStore.isDark
</script>

<style>
/* 全局暗色主题背景 */
html.dark body {
  background-color: #141414;
  color: #e0e0e0;
}

html.dark .layout-content {
  background-color: #141414 !important;
}

/* ========== TransactionForm ========== */
html.dark .transaction-form {
  background-color: #1f1f1f !important;
}

/* ========== TransactionDetail ========== */
html.dark .transaction-item {
  background-color: #1f1f1f !important;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

html.dark .transaction-desc {
  color: #e0e0e0 !important;
}

html.dark .transaction-date {
  color: #aaa !important;
}

html.dark .transaction-member {
  color: #888 !important;
}

html.dark .transaction-amount {
  color: #ff7875 !important;
}

html.dark .expand-icon {
  color: #888 !important;
}

html.dark .transaction-detail {
  border-top-color: #333 !important;
}

html.dark .posting-row {
  background-color: #2a2a2a !important;
}

html.dark .posting-account {
  color: #e0e0e0 !important;
}

html.dark .posting-commodity {
  color: #aaa !important;
}

/* ========== Dashboard ========== */
html.dark .overview {
  background-color: #1f1f1f !important;
}

html.dark .date-group {
  background-color: #1f1f1f !important;
}

html.dark .date-header {
  background-color: #2a2a2a !important;
  color: #e0e0e0 !important;
}

html.dark .date-header:hover {
  background-color: #333 !important;
}

html.dark .date-icon {
  color: #888 !important;
}

html.dark .date-items .transaction-item {
  border-bottom-color: #333 !important;
}

/* ========== Calendar ========== */
html.dark .calendar,
html.dark .calendar-wrapper {
  background-color: #1f1f1f !important;
}

html.dark .day-cell {
  background-color: #2a2a2a !important;
  border-color: #333 !important;
}

html.dark .day-cell.other-month {
  background-color: #222 !important;
  color: #666 !important;
}

html.dark .day-cell.today {
  background-color: #2b1d0e !important;
}

html.dark .day-number {
  color: #e0e0e0 !important;
}

html.dark .day-cell.today .day-number {
  background-color: #fa8c16;
  color: #fff !important;
}

html.dark .day-cell.selected {
  background-color: #15325b !important;
  box-shadow: 0 0 0 2px #177ddc !important;
}

html.dark .day-cell.in-range {
  background-color: #1a2f4a !important;
}

html.dark .day-cell:hover {
  background-color: #333 !important;
}

html.dark .income,
html.dark .day-income {
  color: #73d13d !important;
}

html.dark .expense,
html.dark .day-expense {
  color: #ff7875 !important;
}

html.dark .weekday {
  color: #888 !important;
}

html.dark .arrow {
  color: #aaa !important;
}

/* ========== Layout ========== */
html.dark .mobile-top-bar {
  background-color: #1f1f1f !important;
  border-bottom-color: #333 !important;
}

html.dark .member-label {
  color: #aaa !important;
}

html.dark .mobile-tab-bar {
  background-color: #1f1f1f !important;
  border-top-color: #333 !important;
}

html.dark .tab-item {
  color: #888 !important;
}

html.dark .tab-item.active {
  color: #177ddc !important;
}

html.dark .drawer-section h4 {
  color: #999 !important;
}

/* ========== ReportView ========== */
html.dark .report-view {
  background-color: #1f1f1f !important;
}

/* ========== AccountTree ========== */
html.dark .tree {
  background-color: #1f1f1f !important;
}

html.dark .detail-panel {
  background-color: #1f1f1f !important;
}

html.dark .detail-label {
  color: #aaa !important;
}

html.dark .node-title.active {
  background-color: #15325b !important;
  color: #177ddc !important;
}

/* ========== Common ========== */
html.dark .empty,
html.dark .loading {
  color: #888 !important;
}
</style>
