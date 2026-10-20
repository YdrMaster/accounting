## ADDED Requirements

### Requirement: 整日模式（full_days）
`GET /api/transactions` SHALL 支持布尔查询参数 `full_days`。当其为 `true` 时，系统 SHALL 返回满足筛选、且位于 `[D 当天起点, to 当天终点]` 内的全部交易，其中 D 是最新 `limit` 笔匹配交易中最早一笔所在的那一天；返回条数 SHALL 不少于 `limit`（匹配交易总数少于 `limit` 时返回全部），排序 SHALL 与默认一致（`date_time`、`id` 倒序）。`full_days` 缺省为 `false`，此时行为不变（最多返回 `limit` 笔）。

#### Scenario: 页尾对齐整天
- **WHEN** 请求 `GET /api/transactions?to=2026-07-05&limit=100&full_days=true`，第 100 笔交易落在 07-05，且 07-05 共有 30 笔
- **THEN** 响应包含 07-05 全部 30 笔，以及 07-05 之前各天的全部匹配交易
- **THEN** 返回条数不少于 100
- **THEN** 最旧一笔是其所在那天的最早一笔（该天完整）

#### Scenario: 匹配交易不足 limit
- **WHEN** 满足筛选且 `date <= to` 的交易共 30 笔
- **THEN** 响应返回这 30 笔
- **THEN** 不因 `full_days` 额外扩展或报错

#### Scenario: 单日超过 limit
- **WHEN** 某天有 250 笔交易，请求 `GET /api/transactions?limit=100&full_days=true`，第 100 笔落在该天
- **THEN** 响应返回该天的全部 250 笔（可能还包含更晚的天）

#### Scenario: 缺省行为不变
- **WHEN** 请求不带 `full_days`
- **THEN** 系统按原语义最多返回 `limit` 笔

### Requirement: 交易读接口失败返回非 2xx 状态码
交易读接口（`GET /api/transactions`、`GET /api/transactions/{id}`、`GET /api/postings/{id}`）在失败时 SHALL 返回非 2xx HTTP 状态码：参数非法（如非整数的 `limit`/`offset`、非法日期、引用的标签不存在）→ 400；数据库或内部故障 → 5xx。系统 SHALL NOT 以 2xx 状态码携带纯文本错误体返回失败结果。

#### Scenario: 非法 limit 返回 400 且非 200
- **WHEN** 请求 `GET /api/transactions?limit=abc`
- **THEN** 响应状态码为 400（非 200）
- **AND** 响应体为按请求语言本地化的错误文案

#### Scenario: 非法 offset 或日期返回 400
- **WHEN** 请求 `GET /api/transactions?offset=1.5` 或 `GET /api/transactions?from=2026/07/01`
- **THEN** 响应状态码为 400

#### Scenario: 数据库故障返回 5xx
- **WHEN** 查询过程中发生数据库故障
- **THEN** 响应状态码为 5xx（非 200）

#### Scenario: 成功请求仍返回 200 JSON
- **WHEN** 请求 `GET /api/transactions?limit=100` 正常返回交易列表
- **THEN** 响应状态码为 200
- **AND** 响应体为合法 JSON 数组，可被客户端直接解析
