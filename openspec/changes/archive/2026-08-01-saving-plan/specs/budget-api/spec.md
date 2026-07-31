# budget-api

## MODIFIED Requirements

### Requirement: 创建预算表
系统 SHALL 提供 `POST /api/budgets` 端点，接受 name、period（可选，缺省 null 表示一次性预算）、deadline（可选，"YYYY-MM-DD"）、commodity_id 和 limits 参数，创建新预算表。

#### Scenario: 成功创建预算表
- **WHEN** 发送 `POST /api/budgets`，body 为 `{"name":"月度生活","period":"monthly","commodity_id":1,"limits":[{"account_id":5,"amount":"2000"},{"account_id":6,"amount":"500"}]}`
- **THEN** 返回 HTTP 201，响应体为新预算表的 BudgetDto

#### Scenario: 成功创建一次性预算表
- **WHEN** 发送创建请求，body 不含 period、含 `"deadline":"2026-09-30"`
- **THEN** 返回 HTTP 201，响应 DTO 的 period 为 null、deadline 为 "2026-09-30"

#### Scenario: 名称为空时拒绝创建
- **WHEN** 发送创建请求，name 为空字符串
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 限额列表为空时拒绝创建
- **WHEN** 发送创建请求，limits 为空数组
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 账户不存在时拒绝创建
- **WHEN** 发送创建请求，limits 中包含不存在的 account_id
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 限额挂在非支出账户时拒绝创建
- **WHEN** 发送创建请求，limits 中包含 Assets 根下的账户
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 币种不存在时拒绝创建
- **WHEN** 发送创建请求，commodity_id 不存在
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 更新预算表
系统 SHALL 提供 `PUT /api/budgets/:id` 端点，接受 name、period（可为 null）、deadline（可为 null）、commodity_id 和 limits 参数，更新已有预算表。

#### Scenario: 成功更新预算表
- **WHEN** 发送 `PUT /api/budgets/1`，body 为 `{"name":"月度家庭","period":"monthly","commodity_id":1,"limits":[{"account_id":5,"amount":"3000"}]}`
- **THEN** 返回 HTTP 200，预算表名称和限额已更新

#### Scenario: 预算表不存在
- **WHEN** 更新不存在的预算表 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 验证失败时拒绝更新
- **WHEN** 更新请求中 limits 包含重复的 account_id
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 查询预算执行情况
系统 SHALL 提供 `GET /api/budgets/:id/status` 端点，接受可选 `date` 查询参数（默认当天），返回预算执行情况。查询日晚于 deadline 时仍返回 HTTP 200，响应体 expired 为 true。

#### Scenario: 查询当月预算执行情况
- **WHEN** 请求 `GET /api/budgets/1/status`
- **THEN** 返回 HTTP 200，响应体包含 period_start、period_end、items（各账户执行情况），expired 为 false

#### Scenario: 查询指定日期的预算执行情况
- **WHEN** 请求 `GET /api/budgets/1/status?date=2025-12-15`
- **THEN** 返回 2025 年 12 月的预算执行情况

#### Scenario: 查询已失效预算的执行情况
- **WHEN** 请求 deadline 早于查询日期的预算的 status
- **THEN** 返回 HTTP 200，响应体 expired 为 true

#### Scenario: 预算表不存在
- **WHEN** 查询不存在的预算表 ID 的执行情况
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 日期格式无效
- **WHEN** 请求 `GET /api/budgets/1/status?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: BudgetDto 响应结构
预算表响应 SHALL 包含字段：id (i64)、name (string)、period (string 或 null)、deadline (string "YYYY-MM-DD" 或 null)、commodity_id (i64)。period 值为 "daily"、"weekly-sun"、"weekly-mon"、"monthly"、"yearly" 之一或 null（一次性预算）。

#### Scenario: 月度预算的 DTO 序列化
- **WHEN** 预算表 period 为 FinancePeriod::Monthly
- **THEN** 响应 JSON 中 period 字段值为 "monthly"

#### Scenario: 一次性预算的 DTO 序列化
- **WHEN** 预算表 period 为 None、deadline 为 2026-09-30
- **THEN** 响应 JSON 中 period 为 null、deadline 为 "2026-09-30"

### Requirement: 预算执行情况响应结构
预算执行情况响应 SHALL 包含：budget (BudgetDto)、expired (bool)、period_start (string 或 null)、period_end (string 或 null)、items (BudgetItemStatusDto 数组)。BudgetItemStatusDto SHALL 包含：account_id (i64)、limit_amount (string)、actual_amount (string)、remaining (string)、percentage (string)。一次性预算的 period_start/period_end 为 null。

#### Scenario: 正常执行情况
- **WHEN** 限额 2000，实际支出 800
- **THEN** 对应 item 的 remaining 为 "1200"，percentage 为 "40"

#### Scenario: 超支情况
- **WHEN** 限额 500，实际支出 502.10
- **THEN** 对应 item 的 remaining 为 "-2.10"，percentage 为 "100.42"

#### Scenario: 已失效预算的响应
- **WHEN** 查询日晚于 deadline
- **THEN** 响应 JSON 中 expired 为 true
