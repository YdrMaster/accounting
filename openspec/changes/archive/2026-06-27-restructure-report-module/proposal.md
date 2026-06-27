## Why

当前 `accounting-service` 中的报表相关代码分散在两个独立的 service 文件中（`report_service.rs` 和 `budget_service.rs`），职责边界模糊。`report_service` 混合了资产负债表、损益表、收支汇总、多维度统计等多种功能，其中损益表和收支汇总与即将新增的资金流量表功能重叠，多维度统计（标签/成员/渠道）属于账目筛选范畴而非报表。预算相关代码包含 CRUD 和执行查询，两者都是报表体系的组成部分。需要统一重组为清晰的报表模块。

## What Changes

- 新建 `report` 模块（`report/mod.rs`），包含 3 个子模块：
  - `balance_sheet.rs`：资产负债表，仅统计资产类账户（资产根账户下的账户），不含权益类。优化为单条 SQL 统计所有资产账户余额
  - `cash_flow.rs`：资金流量表，基于 `FinancePeriod` 计算每个资产账户和总资产在指定周期内的流入/流出/净额
  - `budget.rs`：预算执行表，包含预算 CRUD 和执行查询，基于 `FinancePeriod` 计算非资产指定账户的周期流量与预算额的比例
- `accounting` 核心 crate 中 `BudgetPeriod` 重命名为 `FinancePeriod`（财务周期），影响 `accounting-sql`、`accounting-service`、`accounting-cli`、`accounting-api`
- 移除 `report_service.rs` 中的 `income_statement`、`summary`、`get_balance`、`stats_by_tag`、`stats_by_member`、`stats_by_channel`
- 移除 `budget_service.rs`，功能迁移至 `report/budget.rs`
- 提取共享的数据库查询方法供 `cash_flow` 和 `budget` 复用
- 同步更新 API handler 和 CLI 命令，移除已删除功能的端点和命令

## Capabilities

### New Capabilities

- `report-module`: 统一的报表模块，包含资产负债表、资金流量表、预算执行表三个子模块
- `cash-flow-report`: 资金流量表，基于财务周期统计资产账户的周期流量
- `finance-period`: 通用财务周期概念（Daily/WeeklyFromSunday/WeeklyFromMonday/Monthly/Yearly），替代原 BudgetPeriod

### Modified Capabilities

- `balance-sheet`: 资产负债表简化为仅统计资产类账户，移除权益类，优化为单条 SQL
- `budget-report`: 预算执行表从独立 service 迁移至报表模块，保留 CRUD 和执行查询功能

### Removed Capabilities

- `income-statement`: 损益表功能移除（被资金流量表覆盖）
- `report-summary`: 收支汇总功能移除（被资金流量表的总资产行覆盖）
- `report-stats-by-dimension`: 标签/成员/渠道统计移除（后续在账目筛选中重做）
- `report-single-balance`: 单账户余额查询移除

## Impact

- **accounting crate**: `budget.rs` 中 `BudgetPeriod` → `FinancePeriod`，可能拆分为 `finance_period.rs` + `budget.rs`
- **accounting-sql crate**: 所有引用 `BudgetPeriod` 的地方需更新，新增共享的周期聚合查询方法
- **accounting-service crate**: 删除 `report_service.rs` 和 `budget_service.rs`，新建 `report/` 模块结构
- **accounting-api crate**: 移除 `/api/reports/income-statement`、`/api/reports/summary`、`/api/reports/stats` 端点，更新资产负债表 handler，新增资金流量表和预算 API
- **accounting-cli crate**: 移除 `report is`、`report stat`、`report balance` 命令，更新 `report bs`，新增资金流量表和预算命令
