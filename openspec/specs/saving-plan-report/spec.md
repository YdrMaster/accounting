# saving-plan-report

## Purpose

攒钱计划状态表，包含攒钱计划详情与状态计算功能，基于账户集合（含后代子账户）截至查询日的实时余额合计与共享目标金额对比，给出缺口与达标判定。

## Requirements

### Requirement: 攒钱计划状态数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct SavingPlanDetail {
    pub plan: SavingPlan,
    pub account_ids: Vec<AccountId>,
}

pub struct SavingPlanStatus {
    pub plan: SavingPlan,
    pub expired: bool,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub target_amount: Decimal,
    pub current_balance: Decimal,
    pub gap: Decimal,
    pub met: bool,
}
```

其中 `gap = target_amount − current_balance`，`met = current_balance ≥ target_amount`。

#### Scenario: 未达标时 gap 为正
- **WHEN** 目标金额 5000，当前余额合计 3200
- **THEN** gap=1800，met=false

#### Scenario: 已达标时 met 为 true
- **WHEN** 目标金额 5000，当前余额合计 5300
- **THEN** gap=-300，met=true

### Requirement: 攒钱计划状态计算
SavingPlanService SHALL 提供 `get_saving_plan_status(plan_id, date)` 方法。`current_balance` SHALL 为账户集合（含所有后代子账户，利用闭包表 `account_ancestors` 展开）截至查询日（含当日）的余额合计，且仅统计 commodity_id 与计划币种匹配的分录。

#### Scenario: 多账户合并余额
- **WHEN** 攒钱计划关联余额宝（余额 3000）和微信零钱（余额 2000）
- **THEN** current_balance = 5000

#### Scenario: 包含后代子账户余额
- **WHEN** 攒钱计划关联 Assets:Bank（自身余额 1000），其子账户 Assets:Bank:Checking 余额 2500
- **THEN** current_balance = 3500

#### Scenario: 非本币分录不计入
- **WHEN** 计划币种为 CNY，账户有 CNY 余额 4000 和 USD 余额 100
- **THEN** current_balance = 4000 CNY（100 USD 不计入）

#### Scenario: 查询截至当日的余额
- **WHEN** 账户在查询日之后有一笔 500 入账，查询日之前余额 4500
- **THEN** current_balance = 4500（查询日后的交易不计入）

### Requirement: 周期区间计算
`period` 非空时，`period_start`/`period_end` SHALL 为查询日所在 `FinancePeriod` 周期的起止日期；`period` 为空（一次性/无节奏）时两者 SHALL 均为 None。周期只提供节奏信息（当前周期区间、下一个检查点），不锁定余额计算窗口。

#### Scenario: 循环计划返回周期区间
- **WHEN** 查询 period=Monthly 的攒钱计划在 2026-06-26 的状态
- **THEN** period_start=Some(2026-06-01), period_end=Some(2026-06-30)

#### Scenario: 一次性计划无周期区间
- **WHEN** 查询 period=None 的攒钱计划在 2026-06-26 的状态
- **THEN** period_start=None, period_end=None

### Requirement: 过期计划的状态返回
查询日 > deadline 时，状态 SHALL 返回 `expired=true`，同时仍返回其余字段（period_start/period_end、target_amount、current_balance、gap、met）的正常计算值。

#### Scenario: 过期计划仍返回完整状态
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-10-15 的状态，当前余额 5200，目标 5000
- **THEN** expired=true，且 current_balance=5200、gap=-200、met=true 正常返回

#### Scenario: 未过期计划 expired 为 false
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-09-30 的状态
- **THEN** expired=false

### Requirement: 攒钱计划验证
创建和更新攒钱计划时 SHALL 调用 `validate_saving_plan` 函数进行验证。

#### Scenario: 验证失败时返回错误
- **WHEN** 创建攒钱计划时账户集合为空
- **THEN** 返回 Err(SavingPlanError::EmptyAccounts)
