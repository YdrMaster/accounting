<template>
  <a-config-provider :theme="antTheme">
    <Layout />
  </a-config-provider>
</template>

<script setup lang="ts">
import { theme } from 'ant-design-vue'
import { computed, watch, onMounted } from 'vue'
import Layout from './components/Layout.vue'
import { useThemeStore } from './stores/theme'

const themeStore = useThemeStore()

const antTheme = computed(() => ({
  algorithm: themeStore.isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
}))

// 确保 html.dark 类与 isDark 同步
watch(
  () => themeStore.isDark,
  (dark) => {
    document.documentElement.classList.toggle('dark', dark)
  },
  { immediate: true },
)

onMounted(() => {
  document.documentElement.classList.toggle('dark', themeStore.isDark)
})
</script>

<style>
/* 全局字号统一 */
h2 {
  font-size: 20px;
}

h3 {
  font-size: 16px;
}

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
  color: #888 !important;
}

html.dark .transaction-summary {
  color: #e0e0e0 !important;
}

html.dark .transaction-date {
  color: #aaa !important;
}

html.dark .transaction-member {
  color: #888 !important;
}

html.dark .amount-expense {
  color: #f5222d !important;
}

html.dark .amount-income {
  color: #52c41a !important;
}

html.dark .amount-neutral,
html.dark .asset-accounts {
  color: #aaa !important;
}

html.dark .tag-income {
  color: #52c41a !important;
}

html.dark .tag-expense {
  color: #f5222d !important;
}

html.dark .col-member {
  border-right-color: #333 !important;
}

html.dark .col-amount {
  border-left-color: #333 !important;
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
  color: #52c41a !important;
}

html.dark .expense,
html.dark .day-expense {
  color: #f5222d !important;
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
html.dark .ant-tree-treenode:hover {
  background-color: #2a2a2a !important;
}

html.dark .ant-tree-treenode-selected {
  background-color: #15325b !important;
}

html.dark .ant-tree-treenode-selected .node-title {
  color: #177ddc !important;
}

/* 树控件整体透明 */
.account-tree .ant-tree {
  background: transparent !important;
}

/* 亮色模式下 AccountTree 样式 */
.account-tree .ant-tree-treenode {
  padding: 0 16px !important;
  min-height: 40px;
  display: flex !important;
  align-items: center;
  transition: background 0.15s;
}

.account-tree .ant-tree-treenode:hover {
  background: #f5f5f5;
}

.account-tree .ant-tree-treenode-selected {
  background: #e6f7ff !important;
}

.account-tree .ant-tree-node-content-wrapper,
.account-tree .ant-tree-node-content-wrapper-normal,
.account-tree .ant-tree-node-content-wrapper-open,
.account-tree .ant-tree-node-content-wrapper-close {
  flex: 1 1 auto !important;
  display: flex !important;
  align-items: center;
  width: 100% !important;
  background: transparent !important;
  box-shadow: none !important;
}

/* 消除 AntDV 选中态内层框 */
.account-tree .ant-tree-node-selected {
  background: transparent !important;
  box-shadow: none !important;
}

.account-tree .ant-tree-title {
  display: block !important;
  width: 100% !important;
}

.account-tree .ant-tree-treenode:hover .add-btn {
  opacity: 1;
}

.account-tree .ant-tree-list-holder-inner {
  width: 100%;
}

/* 去掉树控件所有圆角 */
.account-tree,
.account-tree .ant-tree,
.account-tree .ant-tree-list,
.account-tree .ant-tree-list-holder {
  border-radius: 0 !important;
}

/* ========== Common ========== */
html.dark .empty,
html.dark .loading {
  color: #888 !important;
}

/* ========== Posting rows & bottom drawer ========== */
html.dark .posting-row.selected {
  background: #15395b;
  border-color: #177ddc;
}
html.dark .posting-row.reimbursable {
  background: #111d2c;
}
html.dark .bottom-drawer {
  background: #1f1f1f;
  border-color: #434343;
}
html.dark .selected-card {
  background: #262626;
  border-color: #434343;
}
html.dark .card-meta {
  color: #888;
}
html.dark .badge-refund {
  color: #ff7a45;
  border-color: #ff7a45;
}
html.dark .badge-reimb {
  color: #69c0ff;
  border-color: #69c0ff;
}
</style>
