# Proposal: move-cashback-discounts-to-equity

## Why

内置账户 `Assets:Cashback`（返现）和 `Expenses:Discounts`（折扣）的归类有语义问题，并污染报表：返现是伴随消费产生、只能用于特定消费的受限返利，不是用户持有的资产；折扣/返利既不是用户的收入也不是用户的支出，挂在 `Assets` / `Expenses` 下会分别进入资产负债表（本系统资产负债表仅统计 Assets 根）和收支统计。它们与 `Equity:OpeningBalances` 同构——资金不经收支、直接计入净资产的调整项，应归入 `Equity` 根下。

## What Changes

- 内置子账户 `Assets:Cashback`（返现）改为 `Equity:Cashback`
- 内置子账户 `Expenses:Discounts`（折扣）改为 `Equity:Discounts`
- 两个账户的叶子名称（英文系统名 `Cashback` / `Discounts`、中文名 `返现` / `折扣`）不变，仅父账户变更
- **BREAKING**：种子账户树结构变化，不提供存量数据库迁移；开发阶段存量数据重新导入即可（与既有「不兼容存量数据库」原则一致）
- 行为副作用：两个账户的关闭规则由 Asset（余额须为 0）变为 Equity（无条件关闭）；其余额不再计入资产负债表与现金流报表（按根账户分类）

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `builtin-data-english-storage`: 系统子账户清单变更——`Expenses:Discounts`、`Assets:Cashback` 移至 Equity 根下，成为 `Equity:Discounts`、`Equity:Cashback`
- `transaction-summary-api`: 收支汇总口径变更——统计范围由「资产类（Assets + Equity 根）分录」收窄为「仅 Assets 根分录」，Equity 根分录（期初余额、返现、折扣）不再计入收支统计

## Impact

- 代码：`accounting-sql/src/schema.rs` 种子数据 `child_specs`（两处父引用）
- 规格/文档：`openspec/specs/builtin-data-english-storage/spec.md`（随归档同步）、`spec/sql.md`、`README.md`（内置节点清单）
- 报表：`balance_sheet`（仅统计 Assets）、`cash_flow`（按根账户名分类）的统计口径随账户归属自动变化，无需改代码
- 测试：seed 数据相关断言需同步更新
