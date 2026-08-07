# Proposal: fix-data-refresh

## Why

前端在导入账单或新增/修改/删除交易后，界面不刷新：交易列表、日历、月度汇总、资产负债表、预算与攒钱进度全部停留在旧数据。根因是 Pinia store 的缓存（交易列表、calendarDays、balanceSheet 等）没有任何失效机制——`clearCache()` 已定义但全代码库无任何调用方，导入路径（`channel.importFile` / `ConfigPanel.doImport`）成功后完全不触碰任何业务 store。这是正确性问题，直接影响用户对账的信任。

## What Changes

- 建立统一的数据失效与刷新机制：任何会改变账目数据的操作（导入账单、交易增删改、账户增删改、映射变更）完成后，所有受影响 store 的缓存被失效并重新拉取。
- 修复 `transaction` store 的行为不一致：`create`/`update` 与 `remove` 一样正确处理 `calendarDays` 缓存。
- 修复导入路径：`importFile` 成功后触发交易、账户、报表、预算、攒钱等 store 的刷新。
- 修复编辑/新建交易后预算（BudgetView）与攒钱计划（SavingPlanView）状态不更新的问题。
- 不改变任何 API 契约与数据结构，纯前端行为修复。

## Capabilities

### New Capabilities

- `data-refresh`: 前端数据一致性能力——定义哪些变更操作必须触发哪些视图/缓存的失效与重新加载，以及刷新时机（操作成功后、抽屉/表单关闭前）。

### Modified Capabilities

- `bill-import`: 导入成功后必须触发受影响数据域的重新加载，而不仅是弹出结果提示。

## Impact

- **代码**:`accounting-web/src/stores/`(transaction、channel、report、budget、savingPlan、account)、`ConfigPanel.vue`、`TransactionView.vue`、`CalendarView.vue`、`BudgetView.vue`、`SavingPlanView.vue`、`AccountDrawer.vue` 等。
- **API**：无变化。
- **风险**：刷新范围过大会导致多余请求；需要通过设计明确失效矩阵，控制在按需重拉。
