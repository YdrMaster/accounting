# transaction-api

## Purpose

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
TransactionDto SHALL 包含 `member_name: String` 字段，返回交易关联的成员名称。

#### Scenario: 查询交易成员名
- **当** 查询任意一笔交易时
- **那么** TransactionDto.member_name 为该交易关联的成员名称

### Requirement: TransactionDto 包含渠道路径
TransactionDto SHALL 包含 `channel_path: Vec<ChannelPathNodeDto>` 字段，返回该交易关联的渠道路径节点。

#### Scenario: 交易有单渠道路径
- **当** 查询一笔渠道路径为 `[{position:0, channel_id:1, channel_name:"支付宝", status:"pending"}]` 的交易
- **那么** TransactionDto.channel_path 应返回对应节点，status 为字符串 "pending"

#### Scenario: 交易无渠道路径
- **当** 查询一笔没有渠道路径的交易
- **那么** TransactionDto.channel_path 为空数组

#### Scenario: 渠道状态返回 verified
- **当** 交易渠道路径节点的 status 为 verified
- **那么** TransactionDto.channel_path[0].status 为 "verified"

### Requirement: CreateTransactionRequest 必须提供成员
CreateTransactionRequest SHALL 包含 `member_id: i64` 字段，且该成员 SHALL 已存在。

#### Scenario: 创建交易时提供有效成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向一个已存在成员时
- **那么** 系统创建交易并关联该成员

#### Scenario: 创建交易时未提供成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 缺失时
- **那么** 系统拒绝请求并返回成员必填错误

#### Scenario: 创建交易时成员不存在
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向不存在的成员时
- **那么** 系统拒绝请求并返回成员不存在错误

### Requirement: CreateTransactionRequest 支持渠道路径
CreateTransactionRequest 可包含 `channel_path: Vec<ChannelPathNodeRequest>` 字段；若未提供，系统 SHALL 使用空路径。

#### Scenario: 创建交易时指定渠道路径
- **当** 用户提交 CreateTransactionRequest 且 `channel_path: [{position:0, channel_id:1, status:"verified"}]`
- **那么** 系统创建交易并关联对应 status=verified 的 ChannelPath 节点

#### Scenario: 创建交易时未提供渠道路径
- **当** 用户提交 CreateTransactionRequest 且未包含 `channel_path`
- **那么** 系统创建空渠道路径的交易

### Requirement: PostingDto 包含账户类型
PostingDto SHALL 包含 `account_type: String` 字段，值为 "asset"、"equity"、"income" 或 "expense"。

#### Scenario: 资产类分录
- **WHEN** 查询一笔资产类账户的分录
- **THEN** PostingDto.account_type 为 "asset"

#### Scenario: 收支类分录
- **WHEN** 查询一笔支出类账户的分录
- **THEN** PostingDto.account_type 为 "expense"

### Requirement: ChannelPathNodeDto 字段
系统 SHALL 定义 `ChannelPathNodeDto`，包含：
- `position: i32`
- `channel_id: i64`
- `channel_name: String`
- `status: String`（取值为 "default"、"pending"、"verified" 之一）

#### Scenario: DTO 序列化
- **当** 将 ChannelPathNodeDto 序列化为 JSON
- **那么** 输出包含 position、channel_id、channel_name、status 字段

### Requirement: ChannelPathNodeRequest 字段
系统 SHALL 定义 `ChannelPathNodeRequest`，包含：
- `position: i32`
- `channel_id: i64`
- `status: String`（可选，缺省为 "default"）

#### Scenario: 请求缺省 status
- **当** 请求体仅包含 position 和 channel_id
- **那么** 系统按 status=default 处理

#### Scenario: 请求无效 status
- **当** 请求 status 不是 "default"、"pending"、"verified" 之一
- **那么** 系统返回 400 Bad Request
