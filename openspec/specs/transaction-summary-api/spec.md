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

### Requirement: 按天收支汇总端点
系统 SHALL 提供 `GET /api/reports/daily-summary` 端点，接受 `from` 和 `to` 查询参数（ISO 8601 日期格式），按天返回范围内每个有交易日期的收支汇总。计算规则与周期汇总一致：income = 当日资产类分录正金额之和，expense = 当日资产类分录负金额绝对值之和。仅返回有交易的日期，无交易的日期不出现在结果中。

#### Scenario: 查询跨月范围的按天汇总
- **WHEN** 请求 `GET /api/reports/daily-summary?from=2026-01-26&to=2026-03-08`
- **THEN** 返回 JSON 数组，每个元素为 `{ "date": "YYYY-MM-DD", "income": "72.66", "expense": "340.92" }`，覆盖范围内所有有交易的日期（含 from/to 落在相邻月份的部分）

#### Scenario: 无交易的日期不返回
- **WHEN** 范围内某日没有任何交易
- **THEN** 结果数组中不包含该日期的元素

#### Scenario: 转账交易的按天汇总
- **WHEN** 某日有资产账户间转账（一正一负分录）
- **THEN** 该日 income 和 expense 分别计入正值和负值的绝对值

#### Scenario: 缺少或错误的日期参数
- **WHEN** 请求缺少 `from`/`to` 参数，或日期格式不是 `YYYY-MM-DD`
- **THEN** 返回 400 错误
