# budget-report

## Purpose

预算执行表，包含预算 CRUD 和执行查询功能，基于财务周期计算非资产指定账户的周期流量与预算额的比例。

## Requirements

### Requirement: 预算 CRUD 功能
预算执行表 SHALL 提供以下 CRUD 操作：
- `create_budget`：创建预算表
- `update_budget`：更新预算表
- `delete_budget`：删除预算表
- `list_budgets`：列出所有预算表
- `get_budget_detail`：获取预算表详情（含限额列表）

#### Scenario: 创建预算表
- **WHEN** 调用 `create_budget("月度生活", FinancePeriod::Monthly, commodity_id, &[(account_id, 2000)])`
- **THEN** 返回新创建的 BudgetId

#### Scenario: 列出所有预算表
- **WHEN** 调用 `list_budgets()`
- **THEN** 返回所有预算表的列表

### Requirement: 预算执行查询
预算执行表 SHALL 提供 `get_budget_status(budget_id, date)` 方法，查询指定日期的预算执行情况。

#### Scenario: 查询预算执行状态
- **WHEN** 调用 `get_budget_status(budget_id, 2026-06-15)`
- **THEN** 返回包含 period_start、period_end、items（各账户执行情况）的 BudgetStatus

### Requirement: 预算执行数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct BudgetDetail {
    pub budget: Budget,
    pub limits: Vec<BudgetLimit>,
}

pub struct BudgetStatus {
    pub budget: Budget,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub items: Vec<BudgetItemStatus>,
}

pub struct BudgetItemStatus {
    pub account_id: AccountId,
    pub limit_amount: Decimal,
    pub actual_amount: Decimal,
    pub remaining: Decimal,
    pub percentage: Decimal,
}
```

#### Scenario: 预算执行项包含实际和限额对比
- **WHEN** 某账户限额 2000，实际支出 800
- **THEN** BudgetItemStatus 为 limit_amount=2000, actual_amount=800, remaining=1200, percentage=40

### Requirement: 使用共享的周期聚合查询
预算执行表 SHALL 使用 `posting_sum_by_period` 共享查询方法获取实际支出数据。

#### Scenario: 调用共享查询
- **WHEN** 查询预算执行状态
- **THEN** 调用 `db.posting_sum_by_period(account_ids, period_start, period_end, ...)`

### Requirement: 排除不计预算的标签
预算执行表 SHALL 排除带有 "exclude-from-budget" 或 "不计预算" 标签的分录。

#### Scenario: 排除特定标签
- **WHEN** 某分录带有 "不计预算" 标签
- **THEN** 该分录的金额不计入实际支出统计

### Requirement: 预算验证
创建和更新预算表时 SHALL 调用 `validate_budget` 函数进行验证。

#### Scenario: 验证失败时返回错误
- **WHEN** 创建预算表时限额列表为空
- **THEN** 返回 Err(BudgetError::EmptyLimits)
