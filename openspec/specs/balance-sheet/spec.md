# balance-sheet

## Purpose

资产负债表，统计资产根账户下所有账户的余额（全生命周期分录金额之和）。

## Requirements

### Requirement: 仅统计资产类账户
资产负债表 SHALL 仅统计资产根账户（root name 为 "Assets" 或 "资产"）下的账户，不包含权益类账户。

#### Scenario: 资产账户包含在报表中
- **WHEN** 存在资产类账户 "Assets:Bank"，余额为 10000
- **THEN** 资产负债表的 assets 列表中包含该账户

#### Scenario: 权益账户不包含在报表中
- **WHEN** 存在权益类账户 "Equity:OpeningBalances"，余额为 -5000
- **THEN** 资产负债表中不包含该账户

### Requirement: 单条 SQL 统计所有资产账户余额
系统 SHALL 使用单条 SQL 查询统计所有资产账户的余额，消除 N+1 查询问题。

#### Scenario: SQL 查询结构
- **WHEN** 调用 `balance_sheet()` 方法
- **THEN** 执行一条包含 `GROUP BY account_id, commodity_id` 和 `JOIN account_ancestors` 的 SQL

### Requirement: 资产负债表数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct AccountBalance {
    pub account: Account,
    pub balances: Vec<(CommodityId, Decimal)>,
}

pub struct BalanceSheet {
    pub assets: Vec<AccountBalance>,
}
```

#### Scenario: 资产负债表仅包含 assets 字段
- **WHEN** 创建 BalanceSheet 实例
- **THEN** 仅有 `assets` 字段，无 `equity` 字段

### Requirement: 移除零余额账户
资产负债表 SHALL 过滤掉余额为零的账户。

#### Scenario: 零余额账户不包含
- **WHEN** 存在资产类账户 "Assets:Unused"，余额为 0
- **THEN** 资产负债表的 assets 列表中不包含该账户
