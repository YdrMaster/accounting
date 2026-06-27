# budget-model

## Purpose

定义预算系统的核心数据模型，包括预算周期枚举、预算表结构、限额映射、ID 类型、验证算法和错误类型。

## Requirements

### Requirement: FinancePeriod 枚举定义预算周期类型
系统 SHALL 定义 `FinancePeriod` 枚举，包含 5 个变体：`Daily`(1)、`WeeklyFromSunday`(2)、`WeeklyFromMonday`(3)、`Monthly`(4)、`Yearly`(5)。周起始日直接编码在枚举变体中，无额外字段。

#### Scenario: 枚举值与整数互转
- **WHEN** 将 `FinancePeriod::Monthly` 转为整数
- **THEN** 结果为 4

#### Scenario: 从整数创建枚举
- **WHEN** 从整数 3 创建 FinancePeriod
- **THEN** 结果为 `WeeklyFromMonday`

### Requirement: 周期计算器计算日期范围
FinancePeriod SHALL 提供 `period_range(date: NaiveDate) -> (NaiveDate, NaiveDate)` 方法，给定日期返回其所在预算周期的起止日期。

#### Scenario: Daily 周期
- **WHEN** 对日期 2026-06-26 调用 Daily.period_range()
- **THEN** 返回 (2026-06-26, 2026-06-26)

#### Scenario: WeeklyFromMonday 周期
- **WHEN** 对日期 2026-06-26（周五）调用 WeeklyFromMonday.period_range()
- **THEN** 返回 (2026-06-22, 2026-06-28)（周一到周日）

#### Scenario: WeeklyFromSunday 周期
- **WHEN** 对日期 2026-06-26（周五）调用 WeeklyFromSunday.period_range()
- **THEN** 返回 (2026-06-21, 2026-06-27)（周日到周六）

#### Scenario: Monthly 周期
- **WHEN** 对日期 2026-06-26 调用 Monthly.period_range()
- **THEN** 返回 (2026-06-01, 2026-06-30)

#### Scenario: Yearly 周期
- **WHEN** 对日期 2026-06-26 调用 Yearly.period_range()
- **THEN** 返回 (2026-01-01, 2026-12-31)

### Requirement: Budget 预算表数据结构
系统 SHALL 定义 `Budget` 结构体，包含字段：`id: BudgetId`、`name: String`、`period: FinancePeriod`、`commodity_id: CommodityId`。

#### Scenario: 创建预算表实例
- **WHEN** 用 id=1, name="月度生活", period=Monthly, commodity_id=1 创建 Budget
- **THEN** 各字段值与传入参数一致

### Requirement: BudgetLimit 限额映射数据结构
系统 SHALL 定义 `BudgetLimit` 结构体，包含字段：`budget_id: BudgetId`、`account_id: AccountId`、`amount: Decimal`。同一预算表中同一账户 SHALL 只出现一次。

#### Scenario: 创建限额映射
- **WHEN** 用 budget_id=1, account_id=5, amount=2000 创建 BudgetLimit
- **THEN** 各字段值与传入参数一致

### Requirement: BudgetId 类型
系统 SHALL 定义 `BudgetId(i64)` 新类型，与 AccountId/TagId 等现有 ID 类型模式一致。

#### Scenario: BudgetId 创建和比较
- **WHEN** 创建 BudgetId(1) 和 BudgetId(1)
- **THEN** 两者相等

### Requirement: 预算验证算法
系统 SHALL 提供 `validate_budget` 函数，对预算表和限额列表进行验证。验证规则：名称不能为空、限额列表至少 1 条、每个 account_id 必须存在、同一预算表中 account_id 不可重复、限额金额必须 > 0、commodity_id 必须存在。

#### Scenario: 有效预算通过验证
- **WHEN** 验证名称非空、限额列表非空、账户存在、无重复、金额 > 0、币种存在
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

### Requirement: BudgetError 错误类型
系统 SHALL 定义 `BudgetError` 枚举，包含变体：EmptyName、EmptyLimits、AccountNotFound(AccountId)、DuplicateAccount(AccountId)、InvalidAmount(Decimal)、CommodityNotFound(CommodityId)、BudgetNotFound(BudgetId)、DatabaseError(String)。

#### Scenario: 错误类型可格式化为字符串
- **WHEN** 将 BudgetError::EmptyName 格式化为字符串
- **THEN** 返回有意义的中文错误信息