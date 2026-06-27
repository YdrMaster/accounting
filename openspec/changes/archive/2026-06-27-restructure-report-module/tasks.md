## 1. 重命名 BudgetPeriod 为 FinancePeriod

**文件**: `accounting/src/budget.rs` → 拆分为 `accounting/src/finance_period.rs` + `accounting/src/budget.rs`

**任务**:
- [x] 新建 `accounting/src/finance_period.rs`，将 `BudgetPeriod` 枚举及其实现移动过去，重命名为 `FinancePeriod`
- [x] 更新 `accounting/src/lib.rs`，导出 `finance_period` 模块
- [x] 更新 `accounting/src/budget.rs`，移除 `BudgetPeriod` 定义，改为 `use crate::finance_period::FinancePeriod`
- [x] 更新 `Budget` 结构体的 `period` 字段类型为 `FinancePeriod`
- [x] 运行 `cargo build` 确认编译通过

**验证**: `cargo test -p accounting` 全部通过

---

## 2. 更新 accounting-sql 中的 FinancePeriod 引用

**文件**: `accounting-sql/src/repo/budget.rs`、`accounting-sql/src/database.rs`、`accounting-sql/src/transaction.rs`

**任务**:
- [x] 批量替换所有 `BudgetPeriod` → `FinancePeriod`
- [x] 更新 import 语句：`use accounting::budget::BudgetPeriod` → `use accounting::finance_period::FinancePeriod`
- [x] 运行 `cargo build` 确认编译通过

**验证**: `cargo test -p accounting-sql` 全部通过

---

## 3. 更新 accounting-service 中的 FinancePeriod 引用

**文件**: `accounting-service/src/budget_service.rs`、`accounting-service/src/config/service.rs`

**任务**:
- [x] 批量替换所有 `BudgetPeriod` → `FinancePeriod`
- [x] 更新 import 语句
- [x] 运行 `cargo build` 确认编译通过

**验证**: `cargo test -p accounting-service` 全部通过

---

## 4. 更新 accounting-cli 和 accounting-api 中的 FinancePeriod 引用

**文件**: `accounting-cli/src/cmd/budget.rs`、`accounting-api/src/handlers/budget.rs`（如有）

**任务**:
- [x] 批量替换所有 `BudgetPeriod` → `FinancePeriod`
- [x] 更新 import 语句
- [x] 运行 `cargo build` 确认编译通过

**验证**: `cargo build` 全项目编译通过

---

## 5. 提取共享的周期聚合查询方法

**文件**: `accounting-sql/src/repo/posting.rs`

**任务**:
- [x] 新增 `posting_sum_by_period` 方法：
  ```rust
  pub async fn posting_sum_by_period(
      conn: &mut SqliteConnection,
      account_ids: &[AccountId],
      start_date: NaiveDate,
      end_date: NaiveDate,
      exclude_tag_ids: &[TagId],
      commodity_id: CommodityId,
  ) -> Result<Vec<(AccountId, Decimal)>, DbError>
  ```
- [x] 实现 SQL：按 `account_id` 分组，`SUM(amount)`，过滤日期范围和排除标签
- [x] 在 `SqliteDatabase` 中暴露该方法
- [x] 编写单元测试

**验证**: `cargo test -p accounting-sql posting_sum_by_period` 通过

---

## 6. 创建 report 模块结构

**文件**: `accounting-service/src/report/mod.rs`、`accounting-service/src/lib.rs`

**任务**:
- [x] 新建 `accounting-service/src/report/` 目录
- [x] 新建 `accounting-service/src/report/mod.rs`，声明 3 个子模块
- [x] 更新 `accounting-service/src/lib.rs`，移除 `budget_service` 和 `report_service`，新增 `report` 模块
- [x] 运行 `cargo build` 确认编译通过（此时会有未实现错误，正常）

**验证**: 编译错误仅为「未找到子模块」

---

## 7. 实现资产负债表子模块

**文件**: `accounting-service/src/report/balance_sheet.rs`

**任务**:
- [x] 定义数据结构：
  ```rust
  pub struct AccountBalance {
      pub account: Account,
      pub balances: Vec<(CommodityId, Decimal)>,
  }
  
  pub struct BalanceSheet {
      pub assets: Vec<AccountBalance>,
  }
  ```
- [x] 实现 `balance_sheet` 函数，使用单条 SQL 统计所有资产账户余额
- [x] 在 `accounting-sql/src/repo/posting.rs` 中新增 `posting_sum_all_assets` 方法
- [x] 编写单元测试：验证仅包含资产类账户，不包含权益类

**验证**: `cargo test -p accounting-service balance_sheet` 通过

---

## 8. 实现资金流量表子模块

**文件**: `accounting-service/src/report/cash_flow.rs`

**任务**:
- [x] 定义数据结构：
  ```rust
  pub struct CashFlowItem {
      pub account: Account,
      pub inflow: Decimal,
      pub outflow: Decimal,
      pub net: Decimal,
  }
  
  pub struct CashFlowTotal {
      pub inflow: Decimal,
      pub outflow: Decimal,
      pub net: Decimal,
  }
  
  pub struct CashFlowReport {
      pub period_start: NaiveDate,
      pub period_end: NaiveDate,
      pub items: Vec<CashFlowItem>,
      pub total: CashFlowTotal,
  }
  ```
- [x] 实现 `cash_flow_report` 函数，接受 `date: NaiveDate` 和 `period: FinancePeriod`
- [x] 使用共享的 `posting_sum_by_period` 方法查询数据
- [x] 计算每个资产账户的流入（正金额）、流出（负金额绝对值）、净额
- [x] 计算总资产汇总行
- [x] 编写单元测试

**验证**: `cargo test -p accounting-service cash_flow` 通过

---

## 9. 实现预算执行表子模块

**文件**: `accounting-service/src/report/budget.rs`

**任务**:
- [x] 从 `budget_service.rs` 迁移以下内容：
  - `BudgetDetail`、`BudgetStatus`、`BudgetItemStatus` 数据结构
  - `BudgetService` struct 及其所有方法
- [x] 更新 `get_budget_status` 方法，使用共享的 `posting_sum_by_period`
- [x] 重命名 `BudgetService` 为 `BudgetReport` 或保留原名
- [x] 迁移所有单元测试
- [x] 运行测试确认通过

**验证**: `cargo test -p accounting-service budget` 通过

---

## 10. 删除旧的 service 文件

**文件**: `accounting-service/src/report_service.rs`、`accounting-service/src/budget_service.rs`

**任务**:
- [x] 删除 `accounting-service/src/report_service.rs`
- [x] 删除 `accounting-service/src/budget_service.rs`
- [x] 运行 `cargo build` 确认编译通过

**验证**: 编译通过，无未使用代码警告

---

## 11. 更新 accounting-api handler

**文件**: `accounting-api/src/handlers/report.rs`

**任务**:
- [x] 移除 `income_statement` handler 和路由
- [x] 移除 `summary` handler 和路由
- [x] 移除 `stats` handler 和路由
- [x] 更新 `balance_sheet` handler，适配新的 `BalanceSheet` 结构（仅 assets）
- [x] 新增 `cash_flow` handler：
  ```rust
  async fn cash_flow(
      State(state): State<Arc<AppState>>,
      Query(query): Query<CashFlowQuery>,
  ) -> Result<Json<CashFlowResponse>, String>
  ```
- [x] 新增预算 API handler（从 `budget_service` 迁移到 `report::budget`）
- [x] 更新路由定义

**验证**: `cargo build -p accounting-api` 编译通过

---

## 12. 更新 accounting-cli 命令

**文件**: `accounting-cli/src/cmd/report.rs`、`accounting-cli/src/cmd/budget.rs`

**任务**:
- [x] 移除 `ReportCmd::Is`（损益表）
- [x] 移除 `ReportCmd::Stat`（多维度统计）
- [x] 移除 `ReportCmd::Balance`（单账户余额）
- [x] 更新 `ReportCmd::Bs`，适配新的 `BalanceSheet` 结构
- [x] 新增 `ReportCmd::CashFlow` 命令
- [x] 更新 budget 命令，使用新的 `report::budget` 模块
- [x] 运行 `cargo build` 确认编译通过

**验证**: `cargo build -p accounting-cli` 编译通过

---

## 13. 全项目集成测试

**任务**:
- [x] 运行 `cargo build` 确认全项目编译通过
- [x] 运行 `cargo test` 确认所有测试通过
- [x] 运行 `cargo clippy` 确认无警告
- [x] 手动测试 CLI 命令：
  - `accounting-cli report bs` 正常输出资产列表
  - `accounting-cli report cash-flow --date 2026-06-27 --period monthly` 正常输出
  - `accounting-cli budget list` 正常输出

**验证**: 所有命令正常执行，无 panic 或错误

---

## 14. 文档更新

**文件**: `openspec/specs/` 相关 spec 文件

**任务**:
- [x] 更新 `budget-model/spec.md`，反映 `BudgetPeriod` → `FinancePeriod`
- [x] 新增 `cash-flow-report/spec.md`，描述资金流量表功能
- [x] 更新 `balance-sheet/spec.md`（如有），反映仅统计资产类
- [x] 归档本变更（可选）

**验证**: 文档与实际实现一致
