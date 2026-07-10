## ADDED Requirements

### Requirement: 列出所有预算表
系统 SHALL 提供 `GET /api/budgets` 端点，返回所有预算表列表。响应 SHALL 包含每个预算表的 id、name、period、commodity_id。

#### Scenario: 成功列出预算表
- **WHEN** 数据库中有 2 个预算表
- **THEN** 返回 HTTP 200，响应体为包含 2 个 BudgetDto 的 JSON 数组

#### Scenario: 无预算表时返回空数组
- **WHEN** 数据库中无任何预算表
- **THEN** 返回 HTTP 200，响应体为空 JSON 数组 `[]`

### Requirement: 创建预算表
系统 SHALL 提供 `POST /api/budgets` 端点，接受 name、period、commodity_id 和 limits 参数，创建新预算表。

#### Scenario: 成功创建预算表
- **WHEN** 发送 `POST /api/budgets`，body 为 `{"name":"月度生活","period":"monthly","commodity_id":1,"limits":[{"account_id":5,"amount":"2000"},{"account_id":6,"amount":"500"}]}`
- **THEN** 返回 HTTP 201，响应体为新预算表的 BudgetDto

#### Scenario: 名称为空时拒绝创建
- **WHEN** 发送创建请求，name 为空字符串
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 限额列表为空时拒绝创建
- **WHEN** 发送创建请求，limits 为空数组
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 账户不存在时拒绝创建
- **WHEN** 发送创建请求，limits 中包含不存在的 account_id
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 币种不存在时拒绝创建
- **WHEN** 发送创建请求，commodity_id 不存在
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 获取预算表详情
系统 SHALL 提供 `GET /api/budgets/:id` 端点，返回预算表详情及其限额列表。

#### Scenario: 成功获取详情
- **WHEN** 请求一个有 3 条限额的预算表
- **THEN** 返回 HTTP 200，响应体包含 budget 字段和 limits 数组（3 项）

#### Scenario: 预算表不存在
- **WHEN** 请求不存在的预算表 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

### Requirement: 更新预算表
系统 SHALL 提供 `PUT /api/budgets/:id` 端点，接受 name、period、commodity_id 和 limits 参数，更新已有预算表。

#### Scenario: 成功更新预算表
- **WHEN** 发送 `PUT /api/budgets/1`，body 为 `{"name":"月度家庭","period":"monthly","commodity_id":1,"limits":[{"account_id":5,"amount":"3000"}]}`
- **THEN** 返回 HTTP 200，预算表名称和限额已更新

#### Scenario: 预算表不存在
- **WHEN** 更新不存在的预算表 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 验证失败时拒绝更新
- **WHEN** 更新请求中 limits 包含重复的 account_id
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 删除预算表
系统 SHALL 提供 `DELETE /api/budgets/:id` 端点，删除预算表及其所有限额。

#### Scenario: 成功删除预算表
- **WHEN** 删除一个存在的预算表
- **THEN** 返回 HTTP 200，预算表和所有限额均已删除

#### Scenario: 预算表不存在
- **WHEN** 删除不存在的预算表 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

### Requirement: 查询预算执行情况
系统 SHALL 提供 `GET /api/budgets/:id/status` 端点，接受可选 `date` 查询参数（默认当天），返回预算执行情况。

#### Scenario: 查询当月预算执行情况
- **WHEN** 请求 `GET /api/budgets/1/status`
- **THEN** 返回 HTTP 200，响应体包含 period_start、period_end、items（各账户执行情况）

#### Scenario: 查询指定日期的预算执行情况
- **WHEN** 请求 `GET /api/budgets/1/status?date=2025-12-15`
- **THEN** 返回 2025 年 12 月的预算执行情况

#### Scenario: 预算表不存在
- **WHEN** 查询不存在的预算表 ID 的执行情况
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 日期格式无效
- **WHEN** 请求 `GET /api/budgets/1/status?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: BudgetDto 响应结构
预算表响应 SHALL 包含字段：id (i64)、name (string)、period (string)、commodity_id (i64)。period 值为 "daily"、"weekly-sun"、"weekly-mon"、"monthly"、"yearly" 之一。

#### Scenario: 月度预算的 DTO 序列化
- **WHEN** 预算表 period 为 FinancePeriod::Monthly
- **THEN** 响应 JSON 中 period 字段值为 "monthly"

### Requirement: BudgetLimitDto 响应结构
限额响应 SHALL 包含字段：account_id (i64)、amount (string，Decimal 序列化)。

#### Scenario: 限额金额序列化
- **WHEN** 限额金额为 2000.00
- **THEN** 响应 JSON 中 amount 字段值为 "2000.00"

### Requirement: 预算执行情况响应结构
预算执行情况响应 SHALL 包含：budget (BudgetDto)、period_start (string)、period_end (string)、items (BudgetItemStatusDto 数组)。BudgetItemStatusDto SHALL 包含：account_id (i64)、limit_amount (string)、actual_amount (string)、remaining (string)、percentage (string)。

#### Scenario: 正常执行情况
- **WHEN** 限额 2000，实际支出 800
- **THEN** 对应 item 的 remaining 为 "1200"，percentage 为 "40"

#### Scenario: 超支情况
- **WHEN** 限额 500，实际支出 502.10
- **THEN** 对应 item 的 remaining 为 "-2.10"，percentage 为 "100.42"
