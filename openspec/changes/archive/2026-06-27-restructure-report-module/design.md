## Context

当前 `accounting-service/src/` 中有两个报表相关的 service：
- `report_service.rs`：包含 `balance_sheet`、`income_statement`、`summary`、`get_balance`、`stats_by_tag/member/channel`
- `budget_service.rs`：包含预算 CRUD 和 `get_budget_status`

`accounting` 核心 crate 的 `budget.rs` 定义了 `BudgetPeriod` 枚举（Daily/WeeklyFromSunday/WeeklyFromMonday/Monthly/Yearly），提供 `period_range` 方法计算周期起止日期。

数据库层 `accounting-sql/src/repo/posting.rs` 已有 `posting_sum_by_account`（单账户余额）、`sum_by_account_with_descendants`（带后代聚合的周期汇总）等方法。账户类型通过闭包表 `account_ancestors` 关联到根账户判断。

## Goals / Non-Goals

**Goals:**
- 统一报表模块结构，3 个子模块职责清晰
- `BudgetPeriod` 重命名为 `FinancePeriod`，作为通用财务周期概念
- 资产负债表仅统计资产类账户，优化为单条 SQL
- 资金流量表和预算执行表共享周期聚合查询
- 移除冗余功能（损益表、收支汇总、多维度统计、单账户余额）
- 同步更新 API 和 CLI 层

**Non-Goals:**
- 实现账目筛选功能（后续独立变更）
- 报表数据可视化（前端图表）
- 多币种折算（资金流量表和预算执行表仍按原币种分组）

## Decisions

### 1. 模块结构

**决策**: 新建 `report/` 目录，包含 `mod.rs`、`balance_sheet.rs`、`cash_flow.rs`、`budget.rs`。

**理由**:
- 3 个报表类型职责独立，各自有清晰的数据结构和查询逻辑
- `mod.rs` 统一导出 `ReportService` 或分别导出 3 个 service struct
- 与现有的 `config/` 模块风格一致

**替代方案**: 保留 `report_service.rs` 单文件。但 3 个报表类型加上共享查询方法，单文件会超过 500 行，不利于维护。

### 2. FinancePeriod 命名和位置

**决策**: `BudgetPeriod` → `FinancePeriod`，保留在 `accounting/src/` 中，可能拆分为 `finance_period.rs`（周期定义）+ `budget.rs`（预算模型和验证）。

**理由**:
- `FinancePeriod` 更准确地反映其通用性，不再局限于预算
- 保留在核心 crate 中，因为 `accounting-sql` 和 `accounting-service` 都依赖它
- 拆分文件让周期逻辑和预算模型解耦

### 3. 资产负债表单条 SQL 优化

**决策**: 使用 `GROUP BY account_id, commodity_id` + `JOIN account_ancestors` + `JOIN accounts root` 的单条 SQL 统计所有资产账户余额，Rust 层按根账户名分类。

**理由**:
- 消除 N+1 查询问题（当前逐账户查询）
- SQLite 支持复杂的 `GROUP BY` + `JOIN`
- 通过闭包表找到根账户，判断是否为资产类

**SQL 示例**:
```sql
SELECT 
  p.account_id,
  p.commodity_id,
  SUM(p.amount) as balance
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN account_ancestors aa ON a.id = aa.account_id
JOIN accounts root ON aa.ancestor_id = root.id 
  AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
WHERE root.name IN ('Assets', '资产')
GROUP BY p.account_id, p.commodity_id
```

### 4. 资金流量表数据结构

**决策**: 输入参数为 `FinancePeriod` + `date`（确定具体周期），输出为每个资产账户的 `(流入, 流出, 净额)` + 总资产汇总行。

**数据结构**:
```rust
pub struct CashFlowItem {
    pub account: Account,
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

**理由**:
- 与预算执行表的输入参数风格一致（都是周期 + 日期）
- 总资产汇总行是用户最关心的数字
- 流入/流出分开统计，便于分析资金去向

### 5. 共享周期聚合查询

**决策**: 在 `accounting-sql/src/repo/posting.rs` 中提取 `posting_sum_by_period` 方法，接受 `account_ids`、`start_date`、`end_date`、`commodity_id`，返回 `Vec<(AccountId, Decimal)>`。

**理由**:
- `cash_flow` 和 `budget` 都需要「按周期 + 账户聚合分录金额」
- 底层 SQL 逻辑相同，只是上层数据结构和业务逻辑不同
- 减少重复代码，统一查询优化

### 6. 资产负债表移除权益类

**决策**: 资产负债表仅统计资产根账户下的账户，不包含权益类。

**理由**:
- 用户明确指定只关注资产
- 简化数据结构和查询逻辑
- 权益类账户可通过其他方式查看（如账户页面）

## Risks / Trade-offs

- **[SQL 复杂度]** 单条 SQL 涉及多表 JOIN 和子查询，可能影响可读性 → 添加详细注释，必要时拆分为视图
- **[FinancePeriod 重命名影响面]** 涉及 4 个 crate，需要批量替换 → 使用 IDE 重构功能或 `sed` 批量替换，确保编译通过
- **[API 破坏性变更]** 移除 `/api/reports/income-statement` 等端点，前端可能依赖 → 确认前端未使用这些端点，或同步更新前端代码
- **[资金流量表性能]** 大量分录时周期聚合查询可能较慢 → 数据库已有 `transaction_date` 索引，必要时添加 `account_id + date` 复合索引
