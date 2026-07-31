# budget-tracking

## MODIFIED Requirements

### Requirement: BudgetService 创建预算表
BudgetService SHALL 提供 `create_budget` 方法，在事务中创建预算表并插入所有限额映射。创建前 SHALL 调用 validate_budget 验证。创建参数 SHALL 支持可选 deadline；period 为 None 时创建一次性预算。

#### Scenario: 成功创建预算表
- **WHEN** 用名称"月度生活"、周期 Monthly、币种 CNY、限额[(餐饮, 2000), (交通, 500)] 创建预算表
- **THEN** 返回新 BudgetId，数据库中 budgets 表有 1 行，budget_limits 表有 2 行

#### Scenario: 成功创建一次性预算表
- **WHEN** 用 period=None、deadline=2026-09-30 创建预算表
- **THEN** budgets 表中该行 period 为 NULL、deadline 为 '2026-09-30'

#### Scenario: 验证失败时拒绝创建
- **WHEN** 用空名称创建预算表
- **THEN** 返回 Err，数据库无变化

#### Scenario: 限额挂在非支出账户时拒绝创建
- **WHEN** 限额列表包含 Assets 根下的账户
- **THEN** 返回 Err(BudgetError::AccountNotExpense)，数据库无变化

### Requirement: BudgetService 更新预算表
BudgetService SHALL 提供 `update_budget` 方法，在事务中替换预算表的名称、周期（含置空）、deadline、币种和所有限额映射。

#### Scenario: 更新预算表和限额
- **WHEN** 更新预算表名称为"月度家庭"、替换限额为[(餐饮, 2500)]
- **THEN** 预算表名称已更新，旧限额全部删除，新限额已插入

#### Scenario: 更新 deadline 与置空周期
- **WHEN** 将预算表 period 置为 None、deadline 设为 2026-12-31
- **THEN** 预算表变为一次性预算，deadline 已更新

#### Scenario: 更新不存在的预算表
- **WHEN** 更新 ID 为 999 的预算表
- **THEN** 返回 Err(BudgetError::BudgetNotFound)

### Requirement: BudgetService 查询预算执行情况
BudgetService SHALL 提供 `get_budget_status(budget_id, date)` 方法，返回指定日期的预算执行情况。date 参数支持任意日期（含预算创建之前的日期）。当预算 period 非空时，返回 date 所在周期的执行情况；当 period 为空（一次性预算）时，period_start/period_end 为 None，实际金额为从最早记录累计到 min(date, deadline) 的合计。当 date 晚于 deadline 时，返回结果 expired=true。

#### Scenario: 月度预算当月执行情况
- **WHEN** 查询 Monthly 预算表在 2026-06-26 的执行情况
- **THEN** period_start=2026-06-01, period_end=2026-06-30，每个限额项包含 limit_amount、actual_amount、remaining、percentage，expired=false

#### Scenario: 周度预算当周执行情况
- **WHEN** 查询 WeeklyFromMonday 预算表在 2026-06-26（周五）的执行情况
- **THEN** period_start=2026-06-22, period_end=2026-06-28

#### Scenario: 查询历史日期
- **WHEN** 查询 Monthly 预算表在 2025-12-15 的执行情况
- **THEN** period_start=2025-12-01, period_end=2025-12-31

#### Scenario: 一次性预算累计全部历史
- **WHEN** 查询 period=None、deadline=2026-09-30 的预算表在 2026-09-15 的执行情况
- **THEN** period_start/period_end 为 None，actual_amount 为截至 2026-09-15 的全部历史合计，expired=false

#### Scenario: deadline 之后查询返回已失效
- **WHEN** 查询 deadline=2026-09-30 的预算表在 2026-10-01 的执行情况
- **THEN** 返回结果 expired=true

#### Scenario: deadline 当天仍有效
- **WHEN** 查询 deadline=2026-09-30 的预算表在 2026-09-30 的执行情况
- **THEN** 返回结果 expired=false

#### Scenario: 不存在的预算表
- **WHEN** 查询 ID 为 999 的预算表执行情况
- **THEN** 返回 Err(BudgetError::BudgetNotFound)
