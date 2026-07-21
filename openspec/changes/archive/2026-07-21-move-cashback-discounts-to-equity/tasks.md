# Tasks: move-cashback-discounts-to-equity

## 1. 种子数据

- [x] 1.1 修改 `accounting-sql/src/schema.rs` 的 `child_specs`：`Discounts` 的父引用由 `expenses_id` 改为 `equity_id`，`Cashback` 的父引用由 `assets_id` 改为 `equity_id`

## 2. 测试

- [x] 2.1 全仓检索 `Assets:Cashback` 与 `Expenses:Discounts`（含分写形式 `Cashback` / `Discounts`）的 Rust 代码与测试引用，同步更新为新归属
- [x] 2.2 新增/更新 seed 断言：初始化后存在 `Equity:Cashback`、`Equity:Discounts`，且不存在 `Assets:Cashback`、`Expenses:Discounts`（对应 delta spec「返现与折扣账户挂在 Equity 根下」场景）
- [x] 2.3 运行 `cargo test --workspace`，修复所有因种子结构变化失败的测试

## 3. 文档与规格

- [x] 3.1 更新 `README.md` 内置节点清单（6 个子账户的新归属）
- [x] 3.2 更新 `spec/sql.md` 中内置账户表（`Expenses:Discounts`、`Assets:Cashback` 两行）
- [x] 3.3 运行 `openspec validate move-cashback-discounts-to-equity --strict` 确认变更通过校验

## 4. 验证

- [x] 4.1 新建一个临时数据库，确认种子账户树为 `Equity:OpeningBalances`、`Equity:Cashback`、`Equity:Discounts`、`Expenses:Fees`、`Expenses:InstallmentFees`、`Assets:Cash`
- [x] 4.2 构造一笔含 `Equity:Cashback` posting 的交易，确认资产负债表与收支统计均不包含该 posting

## 5. 收支统计口径收窄（排除 Equity）

- [x] 5.1 修改 `accounting-sql/src/repo/posting.rs` 的 `posting_daily_summary`：统计范围由 `IN ('Assets', 'Equity')` 收窄为仅 `Assets`，并更新函数注释
- [x] 5.2 在 `daily_summary` 测试中补充权益分录用例：含 `Equity:OpeningBalances`（或 `Equity:Cashback`）posting 的交易，其权益侧分录不计入 income/expense（对应 delta spec「权益账户分录不计入收支」场景）
- [x] 5.3 运行 `cargo test --workspace`，确认全部通过
- [x] 5.4 用临时库实测：返现交易（Assets:Cash +10 / Equity:Cashback -10）的按天汇总为 income 10 / expense 0，而非双边各计 10
