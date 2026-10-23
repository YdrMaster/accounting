# Proposal: restore-view-state-on-detail-close

## Why

从交易页或日历页的交易列表打开交易详情（表单覆盖层）再关闭时，底层视图会被重置：交易列表跳回顶部（即最新交易），日历跳回当月。用户丢失进入前的位置，需要重新滚动或翻月才能继续看原来的记录。

## What Changes

- 打开交易表单覆盖层时不再卸载底层页面内容；关闭后底层视图恢复到打开前的状态——交易页的列表滚动位置不变，日历页的显示月份与选中日期不变。
- 该行为覆盖编辑与新建两条打开路径，以及取消、保存、删除三种关闭结果。
- 筛选条件变化后的既有行为不变（仍按 `lazy-load-transactions` 重置到顶部），本变更只约束覆盖层往返。

## Capabilities

### New Capabilities

- `view-state-restoration`: 交易表单覆盖层往返期间保留底层视图状态（交易列表滚动位置、日历显示月份与选中日期）。

### Modified Capabilities

<!-- 无既有能力的需求变更 -->

## Impact

- 前端 `accounting-web`：`src/views/TransactionView.vue`、`src/views/CalendarView.vue` 的覆盖层挂载方式；可能涉及 `src/components/layout/TransactionFormOverlay.vue` 的底层交互屏蔽。
- 无后端、API、数据模型或依赖变更。
- 相关测试：`src/views/__tests__/TransactionView.spec.ts`、`src/views/__tests__/CalendarViewRefresh.spec.ts` 可能需要补充覆盖层往返的断言。
