## ADDED Requirements

### Requirement: 收支分类明细输入参数
收支分类明细 SHALL 接受以下输入：
- `date: NaiveDate`：确定具体周期的日期
- `period: FinancePeriod`：周期类型
- `commodity_id: CommodityId`：统计币种

#### Scenario: 查询月度分类明细
- **WHEN** 调用分类明细 (2026-06-15, Monthly, CommodityId(1))
- **THEN** 统计 2026-06-01 至 2026-06-30 期间 Income 与 Expenses 根下各账户的层级汇总

### Requirement: 层级汇总计算
系统 SHALL 分别以 Income 根与 Expenses 根调用 `sum_by_account_with_descendants`，获得每个有分录的账户及其所有祖先的周期汇总金额。

#### Scenario: 多层级汇总
- **WHEN** "Expenses:餐饮:外卖" 在周期内有分录合计 500
- **THEN** 结果包含 ("Expenses:餐饮:外卖", 500)、("Expenses:餐饮", 500)、("Expenses", 500) 三行（假设无其他分录）

### Requirement: 金额绝对值归一
系统 SHALL 将所有汇总金额取绝对值后返回：Income 侧分录为负值取反，Expenses 侧分录取正值不变。

#### Scenario: 收入金额归一
- **WHEN** "Income:工资" 周期内分录合计 -15000
- **THEN** 返回金额为 15000

### Requirement: 排除不计预算的标签
收支分类明细 SHALL 排除带有 "exclude-from-budget" 或 "不计预算" 标签的分录，与资金流量表口径一致。

#### Scenario: 排除特定标签
- **WHEN** 某分录带有 "exclude-from-budget" 标签
- **THEN** 该分录金额不计入分类明细

### Requirement: 收支分类明细 API 端点
系统 SHALL 提供 `GET /api/reports/category-breakdown`，接受查询参数 `date`（缺省今日）、`period`（缺省 monthly）、`commodity`（缺省 1），一次返回收支两棵树的扁平明细。

#### Scenario: 正常响应
- **WHEN** 请求 `GET /api/reports/category-breakdown?date=2026-07-15&period=monthly`
- **THEN** 返回 `{ "period_start": "...", "period_end": "...", "income": [{ "account": "<显示路径>", "amount": "<字符串>" }], "expense": [...] }`，账户路径按请求语言解析

#### Scenario: 非法周期参数
- **WHEN** 请求参数 period 为不支持的值
- **THEN** 返回 400 状态码
