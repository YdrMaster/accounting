## MODIFIED Requirements

### Requirement: 报表模块结构
系统 SHALL 在 `accounting-service/src/report/` 目录下提供 5 个子模块：
- `balance_sheet.rs`：资产负债表
- `cash_flow.rs`：资金流量表
- `budget.rs`：预算执行表
- `net_worth_trend.rs`：资产趋势表
- `category_breakdown.rs`：收支分类明细

#### Scenario: 模块导入
- **WHEN** 在 `accounting-service/src/lib.rs` 中声明 `pub mod report;`
- **THEN** 可以通过 `accounting_service::report::balance_sheet`、`accounting_service::report::net_worth_trend` 等路径访问报表功能
