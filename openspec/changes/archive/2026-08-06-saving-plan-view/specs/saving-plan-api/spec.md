# saving-plan-api

## ADDED Requirements

### Requirement: 批量查询攒钱计划状态

系统 SHALL 提供 `GET /api/saving-plans/statuses` 端点，接受可选 `date` 查询参数（格式 "YYYY-MM-DD"，默认当天），返回全部攒钱计划的状态数组（SavingPlanStatusDto，与单计划 status 响应同构，含 allocated/satisfaction/accounts）。参与全局分配的计划 SHALL 按（检查点, plan_id）升序排列在前，不参与分配的计划（过期/永久）排列在后。

#### Scenario: 批量返回全部计划状态

- **WHEN** 数据库中有 3 个攒钱计划，请求 `GET /api/saving-plans/statuses`
- **THEN** 返回 HTTP 200，响应体为 3 个 SavingPlanStatusDto 的 JSON 数组，含 allocated/satisfaction/accounts 字段

#### Scenario: 按检查点升序排列

- **WHEN** 计划 1 检查点为 2026-09-30，计划 2 检查点为 2026-07-31
- **THEN** 响应数组中计划 2 排在计划 1 之前

#### Scenario: 批量与单条口径一致

- **WHEN** 对同一日期分别请求批量端点与 `GET /api/saving-plans/:id/status`
- **THEN** 同一计划的 allocated、satisfaction 完全一致

#### Scenario: 无计划时返回空数组

- **WHEN** 数据库中无任何攒钱计划
- **THEN** 返回 HTTP 200，响应体为空 JSON 数组 `[]`

#### Scenario: 日期格式无效

- **WHEN** 请求 `GET /api/saving-plans/statuses?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息
