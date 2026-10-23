# Tasks: unify-income-expense-stats

## 1. 后端：daily-summary 改为逐交易净额口径

- [x] 1.1 改写 `accounting-sql/src/repo/posting.rs` 的 `posting_daily_summary`：内层子查询按 `(transaction_id, commodity_id)` 求资产根分录净额 `tx_net`，外层按 `(day, commodity_id)` 用 `CASE` 拆分为 income/expense，保留 `load_precisions` + `from_db_amount` 精度处理与同日合并；同步更新函数与 `DailySummaryRow` 的文档注释（把「资产根分录正/负金额之和」改为「资产根分录逐交易净额」）。验证：`cargo test -p accounting-sql` 通过，新增/改写单测覆盖纯转账净额 0 与含手续费转账。
- [x] 1.2 更新 `accounting-service/src/report/daily_summary.rs`：修正模块与 `DailySummaryItem` 的文档注释，并把 `test_daily_summary_basic_transfer_and_cross_month` 的转账日期望由 income 600 改为 500。验证：`cargo test -p accounting-service daily_summary` 通过。
- [x] 1.3 为 daily-summary 口径补充后端单测：纯转账当日不返回金额、含手续费转账按净额计入 expense、同日支出与纯转账混合只计支出、权益分录不计入。验证：`cargo test -p accounting-service daily_summary` 新增用例全绿。

## 2. 后端：删除死代码

- [x] 2.1 删除 `accounting-sql/src/repo/posting.rs` 的 `PostingSummary` 结构与 `posting_summary` 函数、`accounting-sql/src/repo/facade.rs` 中的 `posting_summary` facade 声明，以及单测 `test_summary_and_assets_not_doubled` 内针对 `posting_summary` 的两条断言（保留 `posting_sum_all_assets` 不翻倍的断言）。验证：`cargo test -p accounting-sql` 通过。
- [x] 2.2 全仓确认无残留引用并编译。验证：`grep -rn "posting_summary\|PostingSummary" --include=*.rs .`（排除 target）无输出，且 `cargo check --workspace` 通过。

## 3. 前端：月收支汇总统一口径

- [x] 3.1 改写 `accounting-web/src/views/TransactionView.vue` 的 `monthlyExpense`/`monthlyIncome`/`monthlyBalance`，改用 `sumAssetAmount` 逐交易计算（收入 `Σmax(S,0)`、支出 `Σmax(-S,0)`、结余 `ΣS`），移除按 `account_type` 求和的逻辑。验证：`npm run type-check`（或 `vue-tsc`）通过。
- [x] 3.2 为月收支汇总补充前端单测（新增 `TransactionView` 相关用例或复用现有测试骨架）：当月含一笔支出与一笔纯转账时，月支出 = 支出额、月收入 = 0、月结余 = 负支出额，且与日分组头一致。验证：`npm test` 通过。

## 4. 集成验证

- [x] 4.1 端到端一致性验证：构造同一月份含普通收支 + 纯转账 + 含手续费转账的数据，确认日历格子逐日金额与交易列表日分组头相加结果一致。验证：后端 `GET /api/reports/daily-summary` 与前端日头计算结果对照一致。
- [x] 4.2 规格校验。验证：`openspec validate unify-income-expense-stats --strict` 通过。

## Workflow follow-up

- 归档变更（含 spec sync）：`openspec archive unify-income-expense-stats`。
