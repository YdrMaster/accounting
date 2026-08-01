# saving-plan-tracking

## ADDED Requirements

### Requirement: SavingPlanService 批量查询计划状态
SavingPlanService SHALL 提供 `list_saving_plan_statuses(date)` 方法，基于一次全局分配遍历返回所有攒钱计划的状态列表（含 allocated/satisfaction/账户明细），供列表展示等场景消费。单计划状态查询 `get_saving_plan_status(plan_id, date)` SHALL 复用同一全局分配计算，保证两种入口的满足率口径一致。

#### Scenario: 批量状态与单条状态口径一致
- **WHEN** 数据库中有 3 个互相共享账户的攒钱计划，分别调用 `list_saving_plan_statuses` 与逐个调用 `get_saving_plan_status`
- **THEN** 同一计划在两种入口下的 allocated、satisfaction 完全一致

#### Scenario: 空计划列表
- **WHEN** 数据库中无任何攒钱计划，调用 `list_saving_plan_statuses`
- **THEN** 返回空列表
