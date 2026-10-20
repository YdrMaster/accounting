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

### Requirement: PostingDto 包含账户 ID
PostingDto SHALL 包含 `account_id: i64` 字段，值为分录所属账户的内部 ID，与账户 API 返回的 `AccountDto.id` 一致。该字段为新增字段，不影响既有 `account`（账户路径名）字段。

#### Scenario: 查询交易返回分录账户 ID
- **WHEN** 通过 GET /api/transactions/:id 查询一笔包含分录的交易
- **THEN** 每条分录的 PostingDto.account_id 为该分录账户的内部 ID，且与 GET /api/accounts 中对应账户的 id 相同

#### Scenario: 账户 ID 与账户名指向同一账户
- **WHEN** 客户端拿到某条分录的 account_id 和 account
- **THEN** 通过 account_id 查到的账户，其路径名与 account 字段一致

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

### Requirement: TransactionDto 包含转账标记
TransactionDto SHALL 包含只读布尔字段 `is_transfer`，由服务端在组装 DTO 时按分录结构实时判定该交易是否为转账（判定规则见 `transaction-list-ui`）。该字段 SHALL NOT 落盘存储，SHALL 不改变既有字段的语义。

#### Scenario: 转账交易返回 true
- **WHEN** 查询一笔资产账户间的转账交易
- **THEN** TransactionDto.is_transfer 为 true

#### Scenario: 非转账交易返回 false
- **WHEN** 查询一笔普通收支交易
- **THEN** TransactionDto.is_transfer 为 false

#### Scenario: 期初余额返回 false
- **WHEN** 查询一笔资产账户与 `Equity:OpeningBalances` 配对的交易
- **THEN** TransactionDto.is_transfer 为 false

### Requirement: 整日模式（full_days）
`GET /api/transactions` SHALL 支持布尔查询参数 `full_days`。当其为 `true` 时，系统 SHALL 返回满足筛选、且位于 `[D 当天起点, to 当天终点]` 内的全部交易，其中 D 是最新 `limit` 笔匹配交易中最早一笔所在的那一天；返回条数 SHALL 不少于 `limit`（匹配交易总数少于 `limit` 时返回全部），排序 SHALL 与默认一致（`date_time`、`id` 倒序）。`full_days` 缺省为 `false`，此时行为不变（最多返回 `limit` 笔）。

#### Scenario: 页尾对齐整天
- **WHEN** 请求 `GET /api/transactions?to=2026-07-05&limit=100&full_days=true`，第 100 笔交易落在 07-05，且 07-05 共有 30 笔
- **THEN** 响应包含 07-05 全部 30 笔，以及 07-05 之前各天的全部匹配交易
- **THEN** 返回条数不少于 100
- **THEN** 最旧一笔是其所在那天的最早一笔（该天完整）

#### Scenario: 匹配交易不足 limit
- **WHEN** 满足筛选且 `date <= to` 的交易共 30 笔
- **THEN** 响应返回这 30 笔
- **THEN** 不因 `full_days` 额外扩展或报错

#### Scenario: 单日超过 limit
- **WHEN** 某天有 250 笔交易，请求 `GET /api/transactions?limit=100&full_days=true`，第 100 笔落在该天
- **THEN** 响应返回该天的全部 250 笔（可能还包含更晚的天）

#### Scenario: 缺省行为不变
- **WHEN** 请求不带 `full_days`
- **THEN** 系统按原语义最多返回 `limit` 笔

### Requirement: 交易读接口失败返回非 2xx 状态码
交易读接口（`GET /api/transactions`、`GET /api/transactions/{id}`、`GET /api/postings/{id}`）在失败时 SHALL 返回非 2xx HTTP 状态码：参数非法（如非整数的 `limit`/`offset`、非法日期、引用的标签不存在）→ 400；数据库或内部故障 → 5xx。系统 SHALL NOT 以 2xx 状态码携带纯文本错误体返回失败结果。

#### Scenario: 非法 limit 返回 400 且非 200
- **WHEN** 请求 `GET /api/transactions?limit=abc`
- **THEN** 响应状态码为 400（非 200）
- **AND** 响应体为按请求语言本地化的错误文案

#### Scenario: 非法 offset 或日期返回 400
- **WHEN** 请求 `GET /api/transactions?offset=1.5` 或 `GET /api/transactions?from=2026/07/01`
- **THEN** 响应状态码为 400

#### Scenario: 数据库故障返回 5xx
- **WHEN** 查询过程中发生数据库故障
- **THEN** 响应状态码为 5xx（非 200）

#### Scenario: 成功请求仍返回 200 JSON
- **WHEN** 请求 `GET /api/transactions?limit=100` 正常返回交易列表
- **THEN** 响应状态码为 200
- **AND** 响应体为合法 JSON 数组，可被客户端直接解析
