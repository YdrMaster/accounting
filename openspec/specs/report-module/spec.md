# report-module

## Purpose

统一的报表模块，包含资产负债表、资金流量表、预算执行表三个子模块，提供完整的财务报表服务。

## Requirements

### Requirement: 报表模块结构
系统 SHALL 在 `accounting-service/src/report/` 目录下提供 3 个子模块：
- `balance_sheet.rs`：资产负债表
- `cash_flow.rs`：资金流量表
- `budget.rs`：预算执行表

#### Scenario: 模块导入
- **WHEN** 在 `accounting-service/src/lib.rs` 中声明 `pub mod report;`
- **THEN** 可以通过 `accounting_service::report::balance_sheet` 等路径访问报表功能

### Requirement: 移除旧的 service 文件
系统 SHALL 删除 `report_service.rs` 和 `budget_service.rs`，其功能迁移至 `report/` 模块。

#### Scenario: 旧文件不存在
- **WHEN** 检查 `accounting-service/src/` 目录
- **THEN** 不存在 `report_service.rs` 和 `budget_service.rs` 文件

### Requirement: 移除冗余功能
系统 SHALL 移除以下功能：
- `income_statement`（损益表）
- `summary`（收支汇总）
- `get_balance`（单账户余额）
- `stats_by_tag`、`stats_by_member`、`stats_by_channel`（多维度统计）

#### Scenario: 旧功能不可访问
- **WHEN** 尝试调用 `ReportService::income_statement()`
- **THEN** 编译错误：方法不存在
