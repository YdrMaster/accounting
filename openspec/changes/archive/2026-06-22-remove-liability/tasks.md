# Tasks: remove-liability

> 回溯归档：以下任务均已实现（commit `4b9fa057` ~2026-06-22），checkbox 标记为 `[x]` 复盘。

## 1. Domain 层（`accounting`）

- [x] 1.1 `account_type.rs`：删 `Liability`，重编号 `Asset=1, Equity=2, Income=3, Expense=4`
- [x] 1.2 `account_type.rs`：删 `is_permanent()` 方法及测试
- [x] 1.3 `account_type.rs`：`close_conditions()` 改为仅 `Asset` 要求余额为零；删 `display_name`/`from_prefix` 的 Liability 分支
- [x] 1.4 `validation.rs`：`validate_account_close()` 删 Liability match arm；测试去 Liability 断言

## 2. 数据库层（`accounting-sql`）

- [x] 2.1 `schema.rs`：`accounts.account_type` CHECK 从 `BETWEEN 1 AND 5` 改 `BETWEEN 1 AND 4`
- [x] 2.2 `schema.rs`：种子 `SEED_ACCOUNTS_ROOT_EN/ZH` 编号重排（Equity 3→2、Income 4→3、Expense 5→4）+ 子账户同步
- [x] 2.3 `repo/account.rs`：`map_account` 类型映射重编（1→Asset、2→Equity、3→Income、4→Expense）
- [x] 2.4 `repo/posting.rs`：测试辅助插入编号 4/5 改 3/4

## 3. Service / API / CLI / 前端

- [x] 3.1 `report_service.rs`：`BalanceSheet` 删 `liabilities` 字段；`balance_sheet()` 删 Liability 分支；统计分支编号 4/5 改 3/4
- [x] 3.2 `accounting-api/handlers/report.rs`：`BalanceSheetResponse` 删 `liabilities`；handler 去映射
- [x] 3.3 `accounting-cli/cmd/mod.rs`：`AccountTypeArg` 删 Liability；`cmd/report.rs` `bs` 输出去 `liabilities` 块；README 去 Liability 示例
- [x] 3.4 `accounting-web`：确认仅 4 个 tab，无需改动；README 更新

## 4. i18n 与文档

- [x] 4.1 `accounting/locales/{zh-CN,en}.yaml`：删 `account_type_liability` 键
- [x] 4.2 `spec/core.md` / `spec/service.md`：删 Liability 枚举，报表描述改 Asset/Equity
- [x] 4.3 历史文档（`plan/phase1-3`、`cli-design`、`docs/superpowers/*`、`spec/refund-reimbursement-design.md`）：仅顶部加废弃标注，不改正文（见 D6）

## 5. 验证

- [x] 5.1 `cargo fmt` / `cargo test --workspace` / `cargo clippy --all-targets` 全通过
- [x] 5.2 `accounting-web && npm run build` 前端构建通过
