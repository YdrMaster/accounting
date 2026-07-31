# saving-plan-model

## Purpose

定义攒钱计划系统的核心数据模型，包括攒钱计划表结构、账户关联结构、ID 类型、验证算法和错误类型。攒钱计划是「资产存量的下限」：一组账户共享一个目标金额，与预算的 per-account 限额模型不同。

## Requirements

### Requirement: SavingPlan 攒钱计划数据结构
系统 SHALL 定义 `SavingPlan` 结构体，包含字段：`id: SavingPlanId`、`period: Option<FinancePeriod>`、`deadline: Option<NaiveDate>`、`commodity_id: CommodityId`、`target_amount: Decimal`。`period` 为 `None` 表示一次性/无节奏计划；`deadline` 为 `None` 表示永久有效。（名称不在结构体上，存于 `saving_plan_names` 多语言表。）

#### Scenario: 创建循环计划实例
- **WHEN** 用 id=1, name="房租备用金", period=Some(Monthly), deadline=None, commodity_id=1, target_amount=6000 创建 SavingPlan
- **THEN** 各字段值与传入参数一致

#### Scenario: 创建一次性计划实例
- **WHEN** 用 id=2, name="旅行基金", period=None, deadline=Some(2026-09-30), commodity_id=1, target_amount=5000 创建 SavingPlan
- **THEN** period 为 None，deadline 为 2026-09-30

### Requirement: SavingPlanAccount 账户关联数据结构
系统 SHALL 定义 `SavingPlanAccount` 结构体，包含字段：`plan_id: SavingPlanId`、`account_id: AccountId`。同一攒钱计划中同一账户 SHALL 只出现一次。与预算的 per-account 限额不同，攒钱计划的账户集合 SHALL 共享一个目标金额（pooled target），判定口径为集合余额合计。

#### Scenario: 创建账户关联
- **WHEN** 用 plan_id=1, account_id=5 创建 SavingPlanAccount
- **THEN** 各字段值与传入参数一致

### Requirement: SavingPlanId 类型
系统 SHALL 定义 `SavingPlanId(i64)` 新类型，与 BudgetId/AccountId 等现有 ID 类型模式一致。

#### Scenario: SavingPlanId 创建和比较
- **WHEN** 创建 SavingPlanId(1) 和 SavingPlanId(1)
- **THEN** 两者相等

### Requirement: 攒钱计划验证算法
系统 SHALL 提供 `validate_saving_plan` 函数，对攒钱计划和账户集合进行验证。验证规则：名称不能为空、账户集合至少 1 个、每个 account_id 必须存在、账户集合中 account_id 不可重复、target_amount 必须 > 0、commodity_id 必须存在、所有账户 MUST 位于 Assets 根账户子树内（利用闭包表 `account_ancestors` 判断祖先）。

#### Scenario: 有效攒钱计划通过验证
- **WHEN** 验证名称非空、账户集合非空、账户存在、无重复、目标金额 > 0、币种存在、所有账户位于 Assets 子树内
- **THEN** 返回 Ok(())

#### Scenario: 空名称验证失败
- **WHEN** 验证名称为空字符串的攒钱计划
- **THEN** 返回 Err(SavingPlanError::EmptyName)

#### Scenario: 空账户集合验证失败
- **WHEN** 验证账户集合为空的攒钱计划
- **THEN** 返回 Err(SavingPlanError::EmptyAccounts)

#### Scenario: 重复账户验证失败
- **WHEN** 验证账户集合中同一账户出现两次
- **THEN** 返回 Err(SavingPlanError::DuplicateAccount)

#### Scenario: 非正目标金额验证失败
- **WHEN** 验证目标金额为 0 或负数
- **THEN** 返回 Err(SavingPlanError::InvalidAmount)

#### Scenario: 非资产账户验证失败
- **WHEN** 验证账户集合中包含位于 Expenses 子树的账户
- **THEN** 返回 Err(SavingPlanError::AccountNotAsset)

### Requirement: SavingPlanError 错误类型
系统 SHALL 定义 `SavingPlanError` 枚举，包含变体：EmptyName、EmptyAccounts、AccountNotFound(AccountId)、DuplicateAccount(AccountId)、InvalidAmount(Decimal)、CommodityNotFound(CommodityId)、PlanNotFound(SavingPlanId)、AccountNotAsset(AccountId)、DatabaseError(String)。

#### Scenario: 错误类型可格式化为字符串
- **WHEN** 将 SavingPlanError::EmptyName 格式化为字符串
- **THEN** 返回有意义的中文错误信息
