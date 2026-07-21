# cash-flow-report

## Purpose

资金流量表——按财务周期统计每个资产账户及总资产在指定周期内的流入、流出与净额。该报表帮助用户直观了解各账户在周期内的资金进出情况，排除不计预算的标签分录，为现金流分析提供数据支撑。

## Requirements

### Requirement: 资金流量表输入参数
资金流量表 SHALL 接受以下输入：
- `date: NaiveDate`：确定具体周期的日期
- `period: FinancePeriod`：周期类型

#### Scenario: 查询月度资金流量
- **WHEN** 调用 `cash_flow_report(2026-06-15, FinancePeriod::Monthly)`
- **THEN** 统计 2026-06-01 至 2026-06-30 期间的资金流量

### Requirement: 资金流量表数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct CashFlowItem {
    pub account: Account,
    pub inflow: Decimal,   // 正金额之和
    pub outflow: Decimal,  // 负金额绝对值之和
    pub net: Decimal,      // 净额 = inflow - outflow
}

pub struct CashFlowTotal {
    pub inflow: Decimal,
    pub outflow: Decimal,
    pub net: Decimal,
}

pub struct CashFlowReport {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub items: Vec<CashFlowItem>,
    pub total: CashFlowTotal,
}
```

#### Scenario: 资金流量表包含周期范围和明细
- **WHEN** 生成资金流量表
- **THEN** 返回包含 `period_start`、`period_end`、`items`（每个资产账户）、`total`（总资产汇总）

### Requirement: 计算每个资产账户的流入流出
系统 SHALL 对每个资产账户，统计周期内的：
- `inflow`：正金额分录之和
- `outflow`：负金额分录的绝对值之和
- `net`：inflow - outflow

#### Scenario: 资产账户有流入和流出
- **WHEN** "Assets:Bank" 在周期内有分录 +5000 和 -3000
- **THEN** 该账户的 CashFlowItem 为 inflow=5000, outflow=3000, net=2000

### Requirement: 计算总资产汇总行
系统 SHALL 汇总所有资产账户的流入、流出、净额，生成 `total` 字段。

#### Scenario: 总资产汇总
- **WHEN** 有两个资产账户：Bank(inflow=5000, outflow=3000) 和 Cash(inflow=1000, outflow=800)
- **THEN** total 为 inflow=6000, outflow=3800, net=2200

### Requirement: 使用共享的周期聚合查询
资金流量表 SHALL 使用 `posting_sum_by_period` 共享查询方法获取数据。

#### Scenario: 调用共享查询
- **WHEN** 生成资金流量表
- **THEN** 调用 `db.posting_sum_by_period(account_ids, start_date, end_date, ...)`

### Requirement: 排除不计预算的标签
资金流量表 SHALL 排除带有 "exclude-from-budget" 或 "不计预算" 标签的分录。

#### Scenario: 排除特定标签
- **WHEN** 某分录带有 "exclude-from-budget" 标签
- **THEN** 该分录的金额不计入资金流量统计
