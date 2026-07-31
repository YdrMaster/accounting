# budget-report

## MODIFIED Requirements

### Requirement: 预算执行查询
预算执行表 SHALL 提供 `get_budget_status(budget_id, date)` 方法，查询指定日期的预算执行情况。预算 period 为空（一次性预算）时，计量窗口为从最早记录累计到 min(date, deadline)；date 晚于 deadline 时返回 expired=true 的状态。

#### Scenario: 查询预算执行状态
- **WHEN** 调用 `get_budget_status(budget_id, 2026-06-15)`（period 非空）
- **THEN** 返回包含 period_start、period_end、items（各账户执行情况）的 BudgetStatus，expired=false

#### Scenario: 查询一次性预算执行状态
- **WHEN** 调用 period 为空的预算的 `get_budget_status(budget_id, 2026-09-15)`
- **THEN** 返回 period_start/period_end 为 None、actual 为全部历史累计的 BudgetStatus

#### Scenario: 查询已失效预算
- **WHEN** 调用 deadline 早于查询日期的预算的 `get_budget_status`
- **THEN** 返回 expired=true 的 BudgetStatus（其余字段仍填充）

### Requirement: 预算执行数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct BudgetDetail {
    pub budget: Budget,
    pub limits: Vec<BudgetLimit>,
}

pub struct BudgetStatus {
    pub budget: Budget,
    pub expired: bool,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
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

`period_start`/`period_end` 在预算 period 为空时为 None；`expired` 在查询日晚于 deadline 时为 true。

#### Scenario: 预算执行项包含实际和限额对比
- **WHEN** 某账户限额 2000，实际支出 800
- **THEN** BudgetItemStatus 为 limit_amount=2000, actual_amount=800, remaining=1200, percentage=40

#### Scenario: 一次性预算无周期区间
- **WHEN** 预算 period 为 None
- **THEN** BudgetStatus 的 period_start 和 period_end 均为 None
