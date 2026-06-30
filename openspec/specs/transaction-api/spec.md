# transaction-api

交易 API 的 DTO 扩展——为交易和分录数据传输对象添加标签、成员名和账户类型字段，支持前端展示需求。

## Requirements

### Requirement: TransactionDto 包含标签名列表
TransactionDto SHALL 包含 `tags: Vec<String>` 字段，返回该交易关联的所有标签名称。

#### Scenario: 交易有标签
- **WHEN** 查询一笔有标签的交易
- **THEN** TransactionDto.tags 包含所有标签名称

#### Scenario: 交易无标签
- **WHEN** 查询一笔没有标签的交易
- **THEN** TransactionDto.tags 为空数组

### Requirement: TransactionDto 包含成员名
TransactionDto 必须包含 `member_name: String` 字段，返回交易关联的成员名称。

#### Scenario: 查询交易成员名
- **当** 查询任意一笔交易时
- **那么** TransactionDto.member_name 为该交易关联的成员名称

### Requirement: CreateTransactionRequest 必须提供成员
CreateTransactionRequest 必须包含 `member_id: i64` 字段，且该成员必须已存在。

#### Scenario: 创建交易时提供有效成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向一个已存在成员时
- **那么** 系统创建交易并关联该成员

#### Scenario: 创建交易时未提供成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 缺失时
- **那么** 系统拒绝请求并返回成员必填错误

#### Scenario: 创建交易时成员不存在
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向不存在的成员时
- **那么** 系统拒绝请求并返回成员不存在错误

### Requirement: PostingDto 包含账户类型
PostingDto SHALL 包含 `account_type: String` 字段，值为 "asset"、"equity"、"income" 或 "expense"。

#### Scenario: 资产类分录
- **WHEN** 查询一笔资产类账户的分录
- **THEN** PostingDto.account_type 为 "asset"

#### Scenario: 收支类分录
- **WHEN** 查询一笔支出类账户的分录
- **THEN** PostingDto.account_type 为 "expense"
