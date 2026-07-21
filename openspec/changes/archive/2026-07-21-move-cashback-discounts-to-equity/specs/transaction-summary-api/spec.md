# transaction-summary-api (delta)

## MODIFIED Requirements

### Requirement: 收支汇总 Service 接口
ReportService SHALL 提供 `summary` 方法，接受日期范围参数，返回周期内资产根（Assets）分录的收支汇总。

#### Scenario: 查询月汇总
- **WHEN** 调用 `summary(from=2026-02-01, to=2026-02-28)`
- **THEN** 返回该月资产根分录的正金额之和（income）和负金额绝对值之和（expense）

#### Scenario: 查询日汇总
- **WHEN** 调用 `summary(from=2026-02-28, to=2026-02-28)`
- **THEN** 返回该日资产根分录的正金额之和（income）和负金额绝对值之和（expense）

#### Scenario: 无交易的周期
- **WHEN** 查询的日期范围内没有交易记录
- **THEN** income 和 expense 均为 0

### Requirement: 汇总计算规则
收支汇总 SHALL 基于资产根（Assets）账户的分录金额计算，SHALL NOT 计入权益根（Equity）及其他根账户的分录：
- income = 周期内所有资产根分录正金额之和
- expense = 周期内所有资产根分录负金额绝对值之和

权益账户（期初余额、返现、折扣等）是净资产的直接调整项，其分录不代表收入或支出。资产与权益之间的划转 SHALL 仅体现资产侧的变动：资产侧为正时表现为「无因的收入」，为负时表现为「无因的支出」，权益侧分录不重复计入。

> **终局决策，不再复议**：收支统计的统计范围是且仅是 Assets 根，永不包含 Equity 根本分录。历史上曾按 Assets+Equity 双边统计，导致资产↔权益划转（期初余额、返现、折扣）在 income/expense 两侧毛额虚增；该口径已废弃，且同时消除了按天汇总与周期汇总之间曾存在的口径不一致。任何后续变更 SHALL NOT 重新引入将 Equity 计入收支统计的提议或实现。

#### Scenario: 支出交易
- **WHEN** 周期内有资产减少分录（amount < 0）
- **THEN** expense 增加该金额的绝对值

#### Scenario: 收入交易
- **WHEN** 周期内有资产增加分录（amount > 0）
- **THEN** income 增加该金额

#### Scenario: 转账交易
- **WHEN** 周期内有转账（资产账户间转移，一正一负）
- **THEN** income 和 expense 分别增加正值和负值的绝对值

#### Scenario: 期初余额入账不双计
- **WHEN** 周期内有期初余额入账（资产账户与 `Equity:OpeningBalances` 一正一负配对）
- **THEN** 仅资产侧分录计入汇总（资产增加表现为 income），权益侧分录不计入，收支总额不因权益分录而双计

#### Scenario: 权益账户分录不计入收支
- **WHEN** 周期内某笔交易包含 Equity 根下账户（如 `Equity:Cashback`）的分录
- **THEN** 该分录既不计入 income 也不计入 expense

### Requirement: 按天收支汇总端点
系统 SHALL 提供 `GET /api/reports/daily-summary` 端点，接受 `from` 和 `to` 查询参数（ISO 8601 日期格式），按天返回范围内每个有交易日期的收支汇总。计算规则与周期汇总一致：income = 当日资产根（Assets）分录正金额之和，expense = 当日资产根分录负金额绝对值之和，不计入权益根（Equity）分录。仅返回有交易的日期，无交易的日期不出现在结果中。

#### Scenario: 查询跨月范围的按天汇总
- **WHEN** 请求 `GET /api/reports/daily-summary?from=2026-01-26&to=2026-03-08`
- **THEN** 返回 JSON 数组，每个元素为 `{ "date": "YYYY-MM-DD", "income": "72.66", "expense": "340.92" }`，覆盖范围内所有有交易的日期（含 from/to 落在相邻月份的部分）

#### Scenario: 无交易的日期不返回
- **WHEN** 范围内某日没有任何交易
- **THEN** 结果数组中不包含该日期的元素

#### Scenario: 转账交易的按天汇总
- **WHEN** 某日有资产账户间转账（一正一负分录）
- **THEN** 该日 income 和 expense 分别计入正值和负值的绝对值

#### Scenario: 权益账户分录不计入按天汇总
- **WHEN** 某日某笔交易包含 Equity 根下账户的分录
- **THEN** 该分录不计入当日 income 或 expense

#### Scenario: 缺少或错误的日期参数
- **WHEN** 请求缺少 `from`/`to` 参数，或日期格式不是 `YYYY-MM-DD`
- **THEN** 返回 400 错误
