# budget-model

## MODIFIED Requirements

### Requirement: Budget 预算表数据结构
系统 SHALL 定义 `Budget` 结构体，包含字段：`id: BudgetId`、`period: Option<FinancePeriod>`（`None` 表示一次性/无节奏预算）、`deadline: Option<NaiveDate>`（截止日期，`None` 表示永久有效）、`commodity_id: CommodityId`。（名称不在结构体上，存于 `budget_names` 多语言表。）

#### Scenario: 创建循环预算实例
- **WHEN** 用 id=1, name="月度生活", period=Some(Monthly), deadline=None, commodity_id=1 创建 Budget
- **THEN** 各字段值与传入参数一致

#### Scenario: 创建一次性预算实例
- **WHEN** 用 period=None, deadline=Some(2026-09-30) 创建 Budget
- **THEN** period 为 None，deadline 为 2026-09-30

### Requirement: 预算验证算法
系统 SHALL 提供 `validate_budget` 函数，对预算表和限额列表进行验证。验证规则：名称不能为空、限额列表至少 1 条、每个 account_id 必须存在、同一预算表中 account_id 不可重复、限额金额必须 > 0、commodity_id 必须存在、所有限额账户 MUST 位于 Expenses 根账户子树内。

#### Scenario: 有效预算通过验证
- **WHEN** 验证名称非空、限额列表非空、账户存在且均为支出账户、无重复、金额 > 0、币种存在
- **THEN** 返回 Ok(())

#### Scenario: 空名称验证失败
- **WHEN** 验证名称为空字符串的预算表
- **THEN** 返回 Err(BudgetError::EmptyName)

#### Scenario: 空限额列表验证失败
- **WHEN** 验证限额列表为空的预算表
- **THEN** 返回 Err(BudgetError::EmptyLimits)

#### Scenario: 重复账户验证失败
- **WHEN** 验证同一预算表中同一账户出现两次
- **THEN** 返回 Err(BudgetError::DuplicateAccount)

#### Scenario: 非正金额验证失败
- **WHEN** 验证限额金额为 0 或负数
- **THEN** 返回 Err(BudgetError::InvalidAmount)

#### Scenario: 非支出账户验证失败
- **WHEN** 验证限额列表中包含 Assets 根下的账户
- **THEN** 返回 Err(BudgetError::AccountNotExpense)

### Requirement: BudgetError 错误类型
系统 SHALL 定义 `BudgetError` 枚举，包含变体：EmptyName、EmptyLimits、AccountNotFound(AccountId)、DuplicateAccount(AccountId)、InvalidAmount(Decimal)、CommodityNotFound(CommodityId)、BudgetNotFound(BudgetId)、AccountNotExpense(AccountId)、DatabaseError(String)。

#### Scenario: 错误类型可格式化为字符串
- **WHEN** 将 BudgetError::EmptyName 格式化为字符串
- **THEN** 返回有意义的中文错误信息
