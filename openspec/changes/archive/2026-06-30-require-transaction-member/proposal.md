## 为什么

目前通过 CLI `tx add` 和核心服务/API 创建交易时，`member_id`（记账人）可以为空，这违反了“每笔交易必须有记账人”的业务规则。把该字段改为必填可以在数据库和模型层面防止无效数据进入系统，并与 `spec/core.md` 中“记录者”的语义保持一致。

## 变更内容

- **破坏性变更**：SQLite schema 中 `transactions.member_id` 从可空改为 `NOT NULL`。
- **破坏性变更**：`accounting::transaction::Transaction.member_id` 从 `Option<MemberId>` 改为 `MemberId`。
- **破坏性变更**：`accounting-cli tx add` 的 `--member` 参数改为必填。
- `accounting-cli tx update` 在不传 `--member` 时保留原交易的 `member_id`。
- Beancount 导入时，交易必须包含 `member` metadata；缺失或无法解析为已知成员时报错。
- 更新所有当前使用 `member_id: None` 的测试、示例数据和报表 fixture。
- 为保证编译通过，对 API handler/DTO 做最小化调整。

## 能力

### 新增能力

_无_

### 变更能力

- `transaction-api`：`member_id` 不再可选，每笔交易必须关联一个已存在的成员。
- `beancount-import`：导入的交易必须包含 `member` metadata，且能解析为已知成员。

## 影响

- `accounting-sql/src/schema.rs`
- `accounting/src/transaction.rs`
- `accounting-sql/src/repo/transaction.rs`
- `accounting-service/src/transaction_service.rs`
- `accounting-cli/src/cmd/tx.rs`
- `accounting-beancount/src/import.rs`
- `accounting-beancount/src/export.rs`
- `accounting-api/src/dto.rs` 与 `accounting-api/src/handlers/transaction.rs`（最小编译修复）
- `accounting-sql`、`accounting-service` 及报表模块中的测试数据
