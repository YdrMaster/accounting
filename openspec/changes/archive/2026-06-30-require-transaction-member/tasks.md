## 1. Schema 与核心模型

- [x] 1.1 在 `accounting-sql/src/schema.rs` 中将 `transactions.member_id` 改为 `INTEGER NOT NULL REFERENCES members(id)`
- [x] 1.2 在 `accounting/src/transaction.rs` 中将 `Transaction.member_id` 从 `Option<MemberId>` 改为 `MemberId`
- [x] 1.3 在 `accounting-sql/src/repo/transaction.rs` 中更新 `TransactionRow` 及 `TryFrom<TransactionRow>`，直接使用 `MemberId`

## 2. SQL Repository 与服务

- [x] 2.1 在 `accounting-sql/src/repo/transaction.rs` 中更新 `transaction_insert` 和 `transaction_update`，绑定 `tx.member_id.0` 而非 `Option`
- [x] 2.2 更新 `accounting-service/src/transaction_service.rs` 中的测试，使其构造交易时使用有效的 `MemberId`
- [x] 2.3 更新 `accounting-service/src/report/budget.rs`、`balance_sheet.rs`、`cash_flow.rs` 中的报表测试，提供非空的 `member_id`

## 3. CLI

- [x] 3.1 在 `accounting-cli/src/cmd/tx.rs` 中将 `TxAddArgs` 的 `--member` 改为必填的 `String` 参数
- [x] 3.2 更新 `parse_tx_args`，始终将成员名解析为 `MemberId`
- [x] 3.3 更新 `parse_tx_args_for_update`，当未提供 `--member` 时读取原交易的 `member_id` 并复用
- [x] 3.4 在 CLI 本地化文件中新增或更新缺少成员时的错误提示文案

## 4. Beancount 导入与导出

- [x] 4.1 在 `accounting-beancount/src/import.rs` 中，当交易缺少 `member` metadata 时报错
- [x] 4.2 在 `accounting-beancount/src/import.rs` 中，当 `member` 名字无法解析为已导入成员时报错
- [x] 4.3 在 `accounting-beancount/src/export.rs` 中适配非可选的 `tx.member_id`
- [x] 4.4 更新 beancount 解析器与导入测试，确保每条测试交易都包含 `member` metadata

## 5. API 编译修复

- [x] 5.1 在 `accounting-api/src/dto.rs` 中将 `CreateTransactionRequest.member_id` 从 `Option<i64>` 改为 `i64`
- [x] 5.2 在 `accounting-api/src/handlers/transaction.rs` 中更新 `create_transaction` 和 `update_transaction`，使用 `MemberId(req.member_id)` 构造 `Transaction`
- [x] 5.3 更新 `TransactionDto` 序列化：`member_id` 为 `i64`，`member_name` 为 `String`
- [x] 5.4 更新 `accounting-web/src/types/api.ts` 中的 `TransactionDto` 和 `CreateTransactionRequest` 类型，与 API 变更保持一致

## 6. 测试与验证

- [x] 6.1 更新 `accounting-sql/src/repo/transaction.rs` 中的测试（`sample_tx`），使用真实的 `MemberId`
- [x] 6.2 更新 `accounting-service/src/transaction_service.rs` 中的测试，创建或引用一个成员
- [x] 6.3 在整个 workspace 运行 `cargo test`，修复剩余的编译或测试失败
- [x] 6.4 对 `tx add` 和 `tx update` 做 CLI 冒烟测试，验证成员必填和保留行为
