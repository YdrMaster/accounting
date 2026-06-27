# finance-period

## Purpose

定义通用财务周期枚举，用于资金流量表、预算执行表等需要按周期统计的报表。从原 `BudgetPeriod` 重命名而来，不再局限于预算场景。

## Requirements

### Requirement: FinancePeriod 枚举定义财务周期类型
系统 SHALL 定义 `FinancePeriod` 枚举，包含 5 个变体：`Daily`(1)、`WeeklyFromSunday`(2)、`WeeklyFromMonday`(3)、`Monthly`(4)、`Yearly`(5)。周起始日直接编码在枚举变体中，无额外字段。

#### Scenario: 枚举值与整数互转
- **WHEN** 将 `FinancePeriod::Monthly` 转为整数
- **THEN** 结果为 4

#### Scenario: 从整数创建枚举
- **WHEN** 从整数 3 创建 FinancePeriod
- **THEN** 结果为 `WeeklyFromMonday`

### Requirement: 周期计算器计算日期范围
FinancePeriod SHALL 提供 `period_range(date: NaiveDate) -> (NaiveDate, NaiveDate)` 方法，给定日期返回其所在财务周期的起止日期。

#### Scenario: Daily 周期
- **WHEN** 对日期 2026-06-27 调用 Daily.period_range()
- **THEN** 返回 (2026-06-27, 2026-06-27)

#### Scenario: WeeklyFromMonday 周期
- **WHEN** 对日期 2026-06-27（周六）调用 WeeklyFromMonday.period_range()
- **THEN** 返回 (2026-06-22, 2026-06-28)（周一到周日）

#### Scenario: WeeklyFromSunday 周期
- **WHEN** 对日期 2026-06-27（周六）调用 WeeklyFromSunday.period_range()
- **THEN** 返回 (2026-06-21, 2026-06-27)（周日到周六）

#### Scenario: Monthly 周期
- **WHEN** 对日期 2026-06-27 调用 Monthly.period_range()
- **THEN** 返回 (2026-06-01, 2026-06-30)

#### Scenario: Yearly 周期
- **WHEN** 对日期 2026-06-27 调用 Yearly.period_range()
- **THEN** 返回 (2026-01-01, 2026-12-31)

### Requirement: FinancePeriod 在 Budget 结构体中使用
`Budget` 结构体的 `period` 字段 SHALL 使用 `FinancePeriod` 类型。

#### Scenario: 创建预算表实例
- **WHEN** 用 period=FinancePeriod::Monthly 创建 Budget
- **THEN** budget.period 字段为 FinancePeriod::Monthly
