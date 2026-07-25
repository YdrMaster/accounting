# report-module delta

## MODIFIED Requirements

### Requirement: 报表模块结构
系统 SHALL 在 `accounting-service/src/report/` 目录下提供 4 个子模块：
- `balance_sheet.rs`：资产负债表
- `cash_flow.rs`：资金流量表（收支账户各层级汇总，原收支分类明细职责并入）
- `budget.rs`：预算执行表
- `net_worth_trend.rs`：资产趋势表

系统 SHALL 删除 `category_breakdown.rs`，其收支分类明细职责由 `cash_flow.rs` 承担。

#### Scenario: 模块导入
- **WHEN** 在 `accounting-service/src/lib.rs` 中声明 `pub mod report;`
- **THEN** 可以通过 `accounting_service::report::balance_sheet`、`accounting_service::report::cash_flow` 等路径访问报表功能，且 `accounting_service::report::category_breakdown` 不存在
