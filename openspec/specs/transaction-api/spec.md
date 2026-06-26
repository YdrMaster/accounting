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
TransactionDto SHALL 包含 `member_name: Option<String>` 字段，返回交易关联的成员名称。

#### Scenario: 交易有关联成员
- **WHEN** 查询一笔有成员的交易
- **THEN** TransactionDto.member_name 为 Some(成员名)

#### Scenario: 交易无关联成员
- **WHEN** 查询一笔没有成员的交易
- **THEN** TransactionDto.member_name 为 None

### Requirement: PostingDto 包含账户类型
PostingDto SHALL 包含 `account_type: String` 字段，值为 "asset"、"equity"、"income" 或 "expense"。

#### Scenario: 资产类分录
- **WHEN** 查询一笔资产类账户的分录
- **THEN** PostingDto.account_type 为 "asset"

#### Scenario: 收支类分录
- **WHEN** 查询一笔支出类账户的分录
- **THEN** PostingDto.account_type 为 "expense"
