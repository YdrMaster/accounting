## ADDED Requirements

### Requirement: 资产趋势报表输入参数
资产趋势报表 SHALL 接受以下输入：
- `period: FinancePeriod`：时间桶粒度，仅支持 WeeklyFromMonday、Monthly、Yearly
- `commodity_id: CommodityId`：统计币种

#### Scenario: 查询月度趋势
- **WHEN** 调用 `net_worth_trend(WeeklyFromMonday 以外的 Monthly, CommodityId(1))`
- **THEN** 按月分桶统计全部历史的资产与负债序列

#### Scenario: 周粒度使用周一起始
- **WHEN** period 为 WeeklyFromMonday
- **THEN** 每个桶的起始日为周一，与 FinancePeriod::WeeklyFromMonday 的 period_range 一致

### Requirement: 按账户与日期聚合增量
系统 SHALL 查询 Assets 根下所有账户在每一天的分录净额增量（GROUP BY account_id, date），排除 Equity 根账户。

#### Scenario: 日增量聚合
- **WHEN** 某账户在 2026-06-15 有分录 +5000 和 -2000
- **THEN** 该账户该日的增量为 3000

#### Scenario: 排除权益根
- **WHEN** Equity 根下的账户有分录
- **THEN** 该分录不计入趋势统计

### Requirement: 时间桶归并与前缀累计
系统 SHALL 将每日增量按 `FinancePeriod::period_range(day).0` 归入对应桶起始日，按 (桶, 账户) 累加后对每个账户按桶序计算前缀和，得到每个桶结束时各账户的累计余额。

#### Scenario: 跨桶累计
- **WHEN** 某账户 6 月桶增量 +3000，7 月桶增量 -1000
- **THEN** 6 月桶结束时余额 3000，7 月桶结束时余额 2000

#### Scenario: 桶内无分录的账户延续余额
- **WHEN** 某账户在某桶内无增量
- **THEN** 该桶结束时余额等于上一桶的累计余额

### Requirement: 按符号拆分总资产与总负债
系统 SHALL 对每个桶，将所有账户累计余额按符号拆分：总资产 = 正余额之和，总负债 = 负余额绝对值之和。

#### Scenario: 资产负债拆分
- **WHEN** 某桶结束时账户余额为 Bank +50000、CreditCard -8000、Cash +2000
- **THEN** 该桶 assets = 52000，liabilities = 8000

### Requirement: 资产趋势 API 端点
系统 SHALL 提供 `GET /api/reports/net-worth-trend`，接受查询参数 `period`（weekly|monthly|yearly，缺省 monthly）与 `commodity`（缺省 1），返回全量历史序列。

#### Scenario: 正常响应
- **WHEN** 请求 `GET /api/reports/net-worth-trend?period=monthly`
- **THEN** 返回 `{ "period": "monthly", "points": [{ "date": "<桶起始日>", "assets": "<字符串>", "liabilities": "<字符串>" }, ...] }`，points 按日期升序

#### Scenario: 非法周期参数
- **WHEN** 请求参数 period 为不支持的值（如 daily）
- **THEN** 返回 400 状态码

#### Scenario: 无数据
- **WHEN** 数据库中无任何分录
- **THEN** 返回 points 为空数组
