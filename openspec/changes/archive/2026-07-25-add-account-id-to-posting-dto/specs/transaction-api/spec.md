# Delta: transaction-api

## ADDED Requirements

### Requirement: PostingDto 包含账户 ID
PostingDto SHALL 包含 `account_id: i64` 字段，值为分录所属账户的内部 ID，与账户 API 返回的 `AccountDto.id` 一致。该字段为新增字段，不影响既有 `account`（账户路径名）字段。

#### Scenario: 查询交易返回分录账户 ID
- **WHEN** 通过 GET /api/transactions/:id 查询一笔包含分录的交易
- **THEN** 每条分录的 PostingDto.account_id 为该分录账户的内部 ID，且与 GET /api/accounts 中对应账户的 id 相同

#### Scenario: 账户 ID 与账户名指向同一账户
- **WHEN** 客户端拿到某条分录的 account_id 和 account
- **THEN** 通过 account_id 查到的账户，其路径名与 account 字段一致
