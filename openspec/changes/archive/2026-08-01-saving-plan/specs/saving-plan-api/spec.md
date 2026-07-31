# saving-plan-api

## Purpose

攒钱计划系统的 HTTP REST API——提供攒钱计划的增删改查和按日期的状态查询端点，让用户为一组资产账户设定共享目标金额，并跟踪实时余额相对目标的缺口与达标情况。错误信息的显示语言由 Lang extractor 处理。

## ADDED Requirements

### Requirement: 列出所有攒钱计划
系统 SHALL 提供 `GET /api/saving-plans` 端点，返回所有攒钱计划列表。

#### Scenario: 成功列出攒钱计划
- **WHEN** 数据库中有 2 个攒钱计划
- **THEN** 返回 HTTP 200，响应体为包含 2 个 SavingPlanDto 的 JSON 数组

#### Scenario: 无攒钱计划时返回空数组
- **WHEN** 数据库中无任何攒钱计划
- **THEN** 返回 HTTP 200，响应体为空 JSON 数组 `[]`

### Requirement: 创建攒钱计划
系统 SHALL 提供 `POST /api/saving-plans` 端点，接受 name、period（可选）、deadline（可选，"YYYY-MM-DD"）、commodity_id、target_amount 和 account_ids 数组参数，创建新攒钱计划。

#### Scenario: 成功创建攒钱计划
- **WHEN** 发送 `POST /api/saving-plans`，body 为 `{"name":"旅行基金","period":null,"deadline":"2026-09-30","commodity_id":1,"target_amount":"5000","account_ids":[5,6]}`
- **THEN** 返回 HTTP 201，响应体为新攒钱计划的 SavingPlanDto

#### Scenario: 名称为空时拒绝创建
- **WHEN** 发送创建请求，name 为空字符串
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 账户集合为空时拒绝创建
- **WHEN** 发送创建请求，account_ids 为空数组
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 挂非资产账户时拒绝创建
- **WHEN** 发送创建请求，account_ids 中包含 Expenses 子树的账户
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 币种不存在时拒绝创建
- **WHEN** 发送创建请求，commodity_id 不存在
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 获取攒钱计划详情
系统 SHALL 提供 `GET /api/saving-plans/:id` 端点，返回攒钱计划详情及其账户 ID 列表。

#### Scenario: 成功获取详情
- **WHEN** 请求一个关联 3 个账户的攒钱计划
- **THEN** 返回 HTTP 200，响应体包含 plan 字段和 account_ids 数组（3 项）

#### Scenario: 攒钱计划不存在
- **WHEN** 请求不存在的攒钱计划 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

### Requirement: 更新攒钱计划
系统 SHALL 提供 `PUT /api/saving-plans/:id` 端点，接受 name、period、deadline、commodity_id、target_amount 和 account_ids 参数，更新已有攒钱计划。

#### Scenario: 成功更新攒钱计划
- **WHEN** 发送 `PUT /api/saving-plans/1`，body 为 `{"name":"欧洲旅行基金","period":null,"deadline":"2026-12-31","commodity_id":1,"target_amount":"8000","account_ids":[5]}`
- **THEN** 返回 HTTP 200，攒钱计划名称、目标金额和账户集合已更新

#### Scenario: 攒钱计划不存在
- **WHEN** 更新不存在的攒钱计划 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 验证失败时拒绝更新
- **WHEN** 更新请求中 account_ids 包含重复的账户 ID
- **THEN** 返回 HTTP 400，响应体包含错误信息

#### Scenario: 挂非资产账户时拒绝更新
- **WHEN** 更新请求中 account_ids 包含 Expenses 子树的账户
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 删除攒钱计划
系统 SHALL 提供 `DELETE /api/saving-plans/:id` 端点，删除攒钱计划及其所有账户关联。

#### Scenario: 成功删除攒钱计划
- **WHEN** 删除一个存在的攒钱计划
- **THEN** 返回 HTTP 200，攒钱计划和所有账户关联均已删除

#### Scenario: 攒钱计划不存在
- **WHEN** 删除不存在的攒钱计划 ID
- **THEN** 返回 HTTP 404，响应体包含错误信息

### Requirement: 查询攒钱计划状态
系统 SHALL 提供 `GET /api/saving-plans/:id/status` 端点，接受可选 `date` 查询参数（格式 "YYYY-MM-DD"，默认当天），返回攒钱计划状态。

#### Scenario: 查询攒钱计划状态
- **WHEN** 请求 `GET /api/saving-plans/1/status`
- **THEN** 返回 HTTP 200，响应体包含 expired、period_start、period_end、target_amount、current_balance、gap、met

#### Scenario: 查询指定日期的状态
- **WHEN** 请求 `GET /api/saving-plans/1/status?date=2026-09-30`
- **THEN** 返回截至 2026-09-30 的攒钱计划状态

#### Scenario: 过期计划返回 200 且 expired 为 true
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-10-15 的状态
- **THEN** 返回 HTTP 200，响应体 expired 字段为 true，其余字段正常返回

#### Scenario: 攒钱计划不存在
- **WHEN** 查询不存在的攒钱计划 ID 的状态
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 日期格式无效
- **WHEN** 请求 `GET /api/saving-plans/1/status?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: SavingPlanDto 响应结构
攒钱计划响应 SHALL 包含字段：id (i64)、name (string)、period (string|null，值为 "daily"、"weekly-sun"、"weekly-mon"、"monthly"、"yearly" 之一或 null)、deadline (string|null，"YYYY-MM-DD")、commodity_id (i64)、target_amount (string，Decimal 序列化)、account_ids (i64 数组)。

#### Scenario: 循环计划的 DTO 序列化
- **WHEN** 攒钱计划 period 为 Some(FinancePeriod::Monthly)，deadline 为 None
- **THEN** 响应 JSON 中 period 字段值为 "monthly"，deadline 字段值为 null

#### Scenario: 一次性计划的 DTO 序列化
- **WHEN** 攒钱计划 period 为 None，deadline 为 2026-09-30
- **THEN** 响应 JSON 中 period 字段值为 null，deadline 字段值为 "2026-09-30"

### Requirement: 攒钱计划状态响应结构
攒钱计划状态响应 SHALL 包含：plan (SavingPlanDto)、expired (bool)、period_start (string|null)、period_end (string|null)、target_amount (string)、current_balance (string)、gap (string)、met (bool)。

#### Scenario: 未达标状态
- **WHEN** 目标金额 5000，当前余额 3200
- **THEN** gap 为 "1800"，met 为 false

#### Scenario: 已达标状态
- **WHEN** 目标金额 5000，当前余额 5300
- **THEN** gap 为 "-300"，met 为 true

#### Scenario: 一次性计划的周期字段为 null
- **WHEN** 查询 period 为 None 的攒钱计划状态
- **THEN** period_start 和 period_end 均为 null
