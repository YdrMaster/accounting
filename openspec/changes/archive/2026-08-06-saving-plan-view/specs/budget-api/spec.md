# budget-api

## ADDED Requirements

### Requirement: 批量查询预算执行情况

系统 SHALL 提供 `GET /api/budgets/statuses` 端点，接受可选 `date` 查询参数（格式 "YYYY-MM-DD"，默认当天），返回全部预算表的执行情况数组（BudgetStatusDto，与单预算 status 响应同构，含 expired/period_start/period_end/items），按预算 id 升序排列。

#### Scenario: 批量返回全部预算执行情况

- **WHEN** 数据库中有 2 个预算表，请求 `GET /api/budgets/statuses`
- **THEN** 返回 HTTP 200，响应体为 2 个 BudgetStatusDto 的 JSON 数组

#### Scenario: 批量与单条口径一致

- **WHEN** 对同一日期分别请求批量端点与 `GET /api/budgets/:id/status`
- **THEN** 同一预算的各 item 的 actual_amount、remaining、percentage 完全一致

#### Scenario: 无预算时返回空数组

- **WHEN** 数据库中无任何预算表
- **THEN** 返回 HTTP 200，响应体为空 JSON 数组 `[]`

#### Scenario: 日期格式无效

- **WHEN** 请求 `GET /api/budgets/statuses?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息
