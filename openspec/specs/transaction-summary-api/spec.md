# transaction-summary-api

收支汇总 API——提供基于日期范围的收支汇总查询功能，返回周期内资产类分录的收入和支出汇总。

## Requirements

### Requirement: 收支汇总 Service 接口
ReportService SHALL 提供 `summary` 方法，接受日期范围参数，返回周期内资产类分录的收支汇总。

#### Scenario: 查询月汇总
- **WHEN** 调用 `summary(from=2026-02-01, to=2026-02-28)`
- **THEN** 返回该月资产类分录的正金额之和（income）和负金额绝对值之和（expense）

#### Scenario: 查询日汇总
- **WHEN** 调用 `summary(from=2026-02-28, to=2026-02-28)`
- **THEN** 返回该日资产类分录的正金额之和（income）和负金额绝对值之和（expense）

#### Scenario: 无交易的周期
- **WHEN** 查询的日期范围内没有交易记录
- **THEN** income 和 expense 均为 0

### Requirement: 收支汇总 API 端点
系统 SHALL 提供 `GET /api/reports/summary` 端点，接受 `from` 和 `to` 查询参数（ISO 8601 日期格式）。

#### Scenario: 成功查询月汇总
- **WHEN** 请求 `GET /api/reports/summary?from=2026-02-01&to=2026-02-28`
- **THEN** 返回 JSON `{ "income": "173692.17", "expense": "34611.83" }`

#### Scenario: 缺少日期参数
- **WHEN** 请求缺少 `from` 或 `to` 参数
- **THEN** 返回 400 错误

#### Scenario: 日期格式错误
- **WHEN** 请求的日期参数格式不是 `YYYY-MM-DD`
- **THEN** 返回 400 错误

### Requirement: 汇总计算规则
收支汇总 SHALL 基于资产类账户（asset + equity）的分录金额计算：
- income = 周期内所有资产类分录正金额之和
- expense = 周期内所有资产类分录负金额绝对值之和

#### Scenario: 支出交易
- **WHEN** 周期内有资产减少分录（amount < 0）
- **THEN** expense 增加该金额的绝对值

#### Scenario: 收入交易
- **WHEN** 周期内有资产增加分录（amount > 0）
- **THEN** income 增加该金额

#### Scenario: 转账交易
- **WHEN** 周期内有转账（资产账户间转移，一正一负）
- **THEN** income 和 expense 分别增加正值和负值的绝对值
