# Proposal: PostingDto 增加 account_id，修复编辑交易时分录账户不显示

## Why

在交易详情页编辑交易时，分录的账户选择器永远显示占位符"选择账户"，即使分录实际已有账户。根因是后端 `PostingDto` 只返回账户路径名（`account`），不返回 `account_id`，前端编辑表单只能把 `accountId` 硬编码为 `null`（`TransactionFormOverlay.vue` 中已有注释说明此限制）。这导致用户每次编辑都被迫重新点选所有账户，且 `canSubmit` 校验 `accountId` 非空，直接保存会被阻止。

## What Changes

- 后端 `PostingDto`（`accounting-api/src/dto.rs`）新增 `account_id: i64` 字段，`posting_to_dto`（`accounting-api/src/handlers/transaction.rs`）填充分录的账户 id。纯新增字段，向后兼容。
- 前端 `PostingDto` 类型（`accounting-web/src/types/api.ts`）同步增加 `account_id: number`。
- 前端编辑表单 `loadTransaction`（`accounting-web/src/components/layout/TransactionFormOverlay.vue`）用 `p.account_id` 填充 `accountId`，使账户选择器正确回显，直接保存不再被校验阻止。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `transaction-api`: PostingDto 新增 `account_id` 字段的要求。
- `transaction-form`: 编辑表单打开时分录账户选择器回显已有账户的要求。

## Impact

- **代码**: `accounting-api/src/dto.rs`、`accounting-api/src/handlers/transaction.rs`、`accounting-web/src/types/api.ts`、`accounting-web/src/components/layout/TransactionFormOverlay.vue`
- **API**: `GET /api/transactions/:id` 等返回 PostingDto 的端点响应增加 `account_id` 字段（向后兼容的新增）
- **依赖**: 无新增依赖
