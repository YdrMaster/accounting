# saving-plan-api

## MODIFIED Requirements

### Requirement: 查询攒钱计划状态
系统 SHALL 提供 `GET /api/saving-plans/:id/status` 端点，接受可选 `date` 查询参数（格式 "YYYY-MM-DD"，默认当天），返回攒钱计划状态。状态基于全局资金分配计算（同币种有效计划按检查点顺序占用资金）。

#### Scenario: 查询攒钱计划状态
- **WHEN** 请求 `GET /api/saving-plans/1/status`
- **THEN** 返回 HTTP 200，响应体包含 expired、period_start、period_end、target_amount、current_balance、gap、met、allocated、satisfaction、accounts

#### Scenario: 查询指定日期的状态
- **WHEN** 请求 `GET /api/saving-plans/1/status?date=2026-09-30`
- **THEN** 返回截至 2026-09-30 的攒钱计划状态

#### Scenario: 共享账户的计划满足率反映占用
- **WHEN** 计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,E} 目标 2000），A 3000、B 1000、E 500
- **THEN** 计划 1 的 satisfaction 为 "100"，计划 2 的 satisfaction 为 "75"

#### Scenario: 过期计划返回 200 且 expired 为 true
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-10-15 的状态
- **THEN** 返回 HTTP 200，响应体 expired 字段为 true，其余字段正常返回

#### Scenario: 攒钱计划不存在
- **WHEN** 查询不存在的攒钱计划 ID 的状态
- **THEN** 返回 HTTP 404，响应体包含错误信息

#### Scenario: 日期格式无效
- **WHEN** 请求 `GET /api/saving-plans/1/status?date=invalid`
- **THEN** 返回 HTTP 400，响应体包含错误信息

### Requirement: 攒钱计划状态响应结构
攒钱计划状态响应 SHALL 包含：plan (SavingPlanDto)、expired (bool)、period_start (string|null)、period_end (string|null)、target_amount (string)、current_balance (string)、gap (string)、met (bool)、allocated (string，Decimal)、satisfaction (string，Decimal)、accounts (AccountAllocationDto 数组)。AccountAllocationDto SHALL 包含：account_id (i64)、balance (string)、occupied_by_earlier (string)、allocated (string)。

#### Scenario: 未达标状态
- **WHEN** 目标金额 5000，当前余额 3200
- **THEN** gap 为 "1800"，met 为 false

#### Scenario: 已达标状态
- **WHEN** 目标金额 5000，当前余额 5300
- **THEN** gap 为 "-300"，met 为 true

#### Scenario: 一次性计划的周期字段为 null
- **WHEN** 查询 period 为 None 的攒钱计划状态
- **THEN** period_start 和 period_end 均为 null

#### Scenario: 账户明细序列化
- **WHEN** 计划关联账户 A（余额 3000，被更早计划占用 2000，本计划分配 1000）
- **THEN** accounts 中 A 对应项为 balance="3000"、occupied_by_earlier="2000"、allocated="1000"
