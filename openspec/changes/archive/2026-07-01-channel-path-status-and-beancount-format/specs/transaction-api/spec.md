## MODIFIED Requirements

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

### Requirement: CreateTransactionRequest 支持渠道路径
CreateTransactionRequest 可包含 `channel_path: Vec<ChannelPathNodeRequest>` 字段；若未提供，系统使用空路径。

#### Scenario: 创建交易时指定渠道路径
- **当** 用户提交 CreateTransactionRequest 且 `channel_path: [{position:0, channel_id:1, status:"verified"}]`
- **那么** 系统创建交易并关联对应 status=verified 的 ChannelPath 节点

#### Scenario: 创建交易时未提供渠道路径
- **当** 用户提交 CreateTransactionRequest 且未包含 `channel_path`
- **那么** 系统创建空渠道路径的交易

## NEW Requirements

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
