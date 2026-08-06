# saving-plan-tracking

## Purpose

定义攒钱计划系统的业务服务层，包括攒钱计划的 CRUD 操作、账户限制（仅 Assets 子树）与 deadline 失效判定。

## Requirements

### Requirement: SavingPlanService 创建攒钱计划
SavingPlanService SHALL 提供 `create_saving_plan` 方法，在事务中创建攒钱计划并插入所有账户关联。创建前 SHALL 调用 validate_saving_plan 验证。

#### Scenario: 成功创建攒钱计划
- **WHEN** 用名称"旅行基金"、一次性（period=None）、deadline=2026-09-30、币种 CNY、目标金额 5000、账户[余额宝, 微信零钱] 创建攒钱计划
- **THEN** 返回新 SavingPlanId，数据库中 saving_plans 表有 1 行，saving_plan_accounts 表有 2 行

#### Scenario: 验证失败时拒绝创建
- **WHEN** 用空名称创建攒钱计划
- **THEN** 返回 Err，数据库无变化

### Requirement: SavingPlanService 更新攒钱计划
SavingPlanService SHALL 提供 `update_saving_plan` 方法，在事务中替换攒钱计划的名称、周期、deadline、币种、目标金额和整个账户集合。

#### Scenario: 更新攒钱计划和账户集合
- **WHEN** 更新攒钱计划名称为"欧洲旅行基金"、替换账户集合为[招行储蓄卡]
- **THEN** 攒钱计划名称已更新，旧账户关联全部删除，新账户关联已插入

#### Scenario: 更新不存在的攒钱计划
- **WHEN** 更新 ID 为 999 的攒钱计划
- **THEN** 返回 Err(SavingPlanError::PlanNotFound)

### Requirement: SavingPlanService 删除攒钱计划
SavingPlanService SHALL 提供 `delete_saving_plan` 方法，级联删除攒钱计划及其所有账户关联。

#### Scenario: 成功删除攒钱计划
- **WHEN** 删除一个存在的攒钱计划
- **THEN** saving_plans 和 saving_plan_accounts 中对应记录均被删除

### Requirement: SavingPlanService 列出攒钱计划
SavingPlanService SHALL 提供 `list_saving_plans` 方法，返回所有攒钱计划列表。

#### Scenario: 列出多个攒钱计划
- **WHEN** 数据库中有 2 个攒钱计划
- **THEN** 返回包含 2 个 SavingPlan 的列表

### Requirement: SavingPlanService 批量查询计划状态
SavingPlanService SHALL 提供 `list_saving_plan_statuses(date)` 方法，基于一次全局分配遍历返回所有攒钱计划的状态列表（含 allocated/satisfaction/账户明细），供列表展示等场景消费。单计划状态查询 `get_saving_plan_status(plan_id, date)` SHALL 复用同一全局分配计算，保证两种入口的满足率口径一致。

#### Scenario: 批量状态与单条状态口径一致
- **WHEN** 数据库中有 3 个互相共享账户的攒钱计划，分别调用 `list_saving_plan_statuses` 与逐个调用 `get_saving_plan_status`
- **THEN** 同一计划在两种入口下的 allocated、satisfaction 完全一致

#### Scenario: 空计划列表
- **WHEN** 数据库中无任何攒钱计划，调用 `list_saving_plan_statuses`
- **THEN** 返回空列表

### Requirement: SavingPlanService 获取攒钱计划详情
SavingPlanService SHALL 提供 `get_saving_plan_detail` 方法，返回攒钱计划及其账户 ID 列表。

#### Scenario: 获取含账户的攒钱计划详情
- **WHEN** 查询关联了 3 个账户的攒钱计划
- **THEN** 返回 SavingPlanDetail 包含 SavingPlan 和 3 个 AccountId

### Requirement: 攒钱计划账户仅限 Assets 子树
创建和更新攒钱计划时，账户集合中的每个账户 MUST 位于 Assets 根账户子树内；挂非资产账户 SHALL 被拒绝。

#### Scenario: 创建时挂支出账户被拒绝
- **WHEN** 创建攒钱计划，账户集合包含 Expenses:Food
- **THEN** 返回 Err(SavingPlanError::AccountNotAsset)，数据库无变化

#### Scenario: 更新时挂负债之外的非资产账户被拒绝
- **WHEN** 更新攒钱计划，将账户集合替换为包含 Income:Salary 的集合
- **THEN** 返回 Err(SavingPlanError::AccountNotAsset)

### Requirement: deadline 失效判定
攒钱计划的失效判定 SHALL 在 service 层进行：`查询日 > deadline` 时计划失效；`查询日 = deadline` 时计划仍有效；`deadline` 为 None 时计划永久有效。`FinancePeriod::period_range` 接口保持不变。

#### Scenario: 查询日晚于 deadline 计划失效
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-10-01 的状态
- **THEN** 判定计划已失效

#### Scenario: deadline 当天计划仍有效
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-09-30 的状态
- **THEN** 判定计划有效

#### Scenario: 无 deadline 永久有效
- **WHEN** 查询 deadline=None 的攒钱计划在任意日期的状态
- **THEN** 判定计划有效

### Requirement: 不计预算标签不适用于攒钱计划
`exclude-from-budget`（不计预算）标签的排除规则 SHALL NOT 应用于攒钱计划的余额统计——余额是事实，不可豁免。

#### Scenario: 带不计预算标签的交易仍计入余额
- **WHEN** 攒钱计划账户有 3 笔交易共 5000 CNY，其中 1 笔带"不计预算"标签计 200 CNY
- **THEN** 余额合计仍为 5000 CNY
