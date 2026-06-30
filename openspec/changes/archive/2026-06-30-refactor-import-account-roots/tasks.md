## 1. Core Types & Schema

- [x] 1.1 从 `accounting/src/account_type.rs` 移除 `AccountType::Import` 枚举变体，同步更新 `from_str`、`display_name`、`close_conditions` 及对应测试。
- [x] 1.2 在 `accounting/src/posting_role.rs` 中处理映射 key：普通分录使用 `Assets` / `Income` / `Expenses`，退款作为支出使用 `Expenses`；按金额符号和退款标记区分 `IncomeExpense` 的 key，更新 `to_key`、`from_key`、`import_segment` 及测试。
- [x] 1.3 在 `accounting-sql/src/schema.rs` 中移除 `SEED_ACCOUNTS_ROOT_EN` 和 `SEED_ACCOUNTS_ROOT_ZH` 里的 `Import/导入` 根账户，并统一使用英文根名 `Assets / Equity / Income / Expenses`（不再按语言区分）。
- [x] 1.4 更新 `accounting-sql/src/schema.rs` 中的种子数据测试，移除对 `Import/导入` 根账户的断言。

## 2. Import & Mapping Services

- [x] 2.1 修改 `accounting-service/src/import_service.rs` 的 `resolve_account_id`：无映射时按 `role + amount 符号 + 退款标记` 构建 fallback 路径，退款作为支出走 `Expenses:Import:<channel>:<cat>`（金额为负），普通收入/支出分别走 `Income:Import:<channel>:<cat>` / `Expenses:Import:<channel>:<cat>`，资产侧走 `Assets:Import:<channel>:<cat>`。
- [x] 2.2 删除 `ImportService::resolve_import_root` 方法及 `ImportError::ImportRootNotFound` 枚举变体。
- [x] 2.3 更新 `accounting-service/src/import_service.rs` 内联测试：验证新 fallback 路径，移除对 `Import` 根账户的检查。
- [x] 2.4 更新 `accounting-service/src/mapping_service.rs` 内联测试，使用映射 key 前缀 `Assets:` / `Income:` / `Expenses:`（退款也使用 `Expenses:`）。
- [x] 2.5 更新 `accounting-service/src/account_service.rs` 中 `ensure_cascading` 相关测试，改用 `Assets:Import:...` / `Expenses:Import:...` 等路径。

## 3. CLI

- [x] 3.1 在 `accounting-cli/src/cmd/import.rs` 中移除 `ImportRootNotFound` 错误映射。
- [x] 3.2 在 `accounting-cli/src/cmd/mapping.rs` 中把 `--category` 帮助示例改为映射 key 形式 `Assets:...` / `Income:...` / `Expenses:...`（退款也使用 `Expenses:`）。
- [x] 3.3 检查并清理 `accounting-cli/locales/*.yaml` 中与 `import_root_not_found` 相关的文案。

## 4. Beancount Import/Export

- [x] 4.1 修改 `accounting-beancount/src/export.rs`：删除 `Import → Equity:Import` 路径转换，直接按根账户名判断 `Asset / Income / Expense`；移除 `Import` 相关的 account_type 分支。
- [x] 4.2 修改 `accounting-beancount/src/import.rs`：删除 `resolve_account_path` 中对 `Equity:Import:` / `account_type: Import` 的特殊处理。
- [x] 4.3 更新 `accounting-beancount/tests/integration_tests.rs` 中断言，确保导出的账户路径为标准 beancount 路径。

## 5. Tests & Documentation

- [x] 5.1 更新 `accounting-cli/tests/natural_keys.rs` 中的 `mapping` 测试，使用新的 `Expenses:餐饮美食` key。
- [x] 5.2 更新 CLI 文档：`accounting-cli/docs/commands/mapping.md`、`import.md`、`beancount.md` 以及 `examples/03-import-and-reconcile.md` 中的路径与 key 示例。
- [x] 5.3 更新 `openspec/specs/account-mapping/spec.md`、`bill-import/spec.md`、`beancount-import/spec.md`、`beancount-export/spec.md`、`account-type-import/spec.md` 主 spec（非 delta），或在归档本次 change 时同步回写。

## 6. Verification

- [x] 6.1 运行 `cargo test --workspace` 并修复所有失败用例。
- [x] 6.2 运行一次端到端导入 smoke test：初始化中文数据库 → 导入支付宝 CSV → 验证生成 `Expenses:Import:支付宝:餐饮美食`、`Assets:Import:支付宝:蚂蚁宝藏信用卡`、`Expenses:Import:支付宝:退款` 等账户。
