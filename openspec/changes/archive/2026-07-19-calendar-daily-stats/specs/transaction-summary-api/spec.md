# transaction-summary-api delta

## ADDED Requirements

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
