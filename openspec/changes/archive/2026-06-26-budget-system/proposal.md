## Why

项目缺少预算管理能力。用户可以记账但没有方式设定支出/收入上限、追踪预算执行情况。预算系统是 Phase 2 规划的核心功能之一，且前端 BudgetView 已有概念原型（硬编码 mockup），需要后端支撑才能落地。

## What Changes

- 新增预算表（Budget）数据模型：包含名称、周期类型、统一币种
- 新增预算限额映射（BudgetLimit）：账户 → 金额的映射，同一预算表中账户唯一
- 新增 BudgetPeriod 枚举：支持自然日、自然周（周日/周一起始）、自然月、自然年
- 新增周期计算器：给定日期自动计算所在预算周期的起止范围
- 新增内置标签：`不计收支`（排除收支统计）、`不计预算`（排除预算统计），互相独立
- 新增 budgets + budget_limits 数据库表，BudgetRepo trait
- 新增 PostingRepo::sum_by_account_with_descendants 统计查询（闭包表后代聚合 + 标签排除）
- 新增 BudgetService：预算表 CRUD + 预算执行情况查询
- 新增 CLI budget 子命令：create / list / show / update / delete
- 种子数据新增 4 条内置标签记录（中英文各 2 条）

## Capabilities

### New Capabilities
- `budget-model`: 预算核心数据模型（BudgetPeriod 枚举、Budget、BudgetLimit、BudgetError、周期计算算法、验证算法）
- `budget-tracking`: 预算执行情况追踪（BudgetService、BudgetStatus、sum_by_account_with_descendants 查询、标签排除逻辑）
- `budget-cli`: CLI 预算管理命令（budget create/list/show/update/delete）
- `built-in-tags-exclude`: 内置标签扩展（不计收支、不计预算）

### Modified Capabilities

## Impact

- `accounting` crate：新增 budget.rs（核心类型）、修改 id.rs（新增 BudgetId）、修改 lib.rs
- `accounting-sql` crate：新增 repo/budget.rs、修改 schema.rs（新增表）、修改种子数据、扩展 PostingRepo、扩展 Database trait
- `accounting-service` crate：新增 budget_service.rs、修改 mod.rs
- `accounting-cli` crate：新增 cmd/budget.rs、修改 cmd/mod.rs
- 数据库迁移：2 张新表 + 4 条内置标签种子数据