# cli-natural-key-resolution

## Purpose

定义命令行工具如何将人类可读的名称、路径、符号解析为内部数据库 ID，并统一错误输出。（TBD：根据实现细化各实体的解析策略与错误码）

## Requirements

### Requirement: 统一的自然键解析层
`accounting-cli` SHALL 在 `cmd/resolver.rs` 中提供异步解析函数，将自然键转换为内部 ID：
- `resolve_member(db, name) -> Result<MemberId, AccountingError>`
- `resolve_account(db, path) -> Result<AccountId, AccountingError>`
- `resolve_channel(db, name) -> Result<ChannelId, AccountingError>`
- `resolve_commodity(db, symbol) -> Result<CommodityId, AccountingError>`
- `resolve_budget(db, name) -> Result<BudgetId, AccountingError>`

解析失败时返回包含实体类型和自然键值的错误信息，如 `成员 'Alice' 不存在`。

#### Scenario: 通过成员名称定位成员
- **WHEN** 调用 `resolve_member(db, "Alice")` 且数据库中存在 name="Alice" 的成员
- **THEN** 返回该成员的 MemberId

#### Scenario: 通过账户路径定位账户
- **WHEN** 调用 `resolve_account(db, "Assets:Cash")` 且该路径存在
- **THEN** 返回对应 AccountId

#### Scenario: 自然键不存在时返回可读错误
- **WHEN** 调用 `resolve_member(db, "Bob")` 且数据库中不存在该成员
- **THEN** 返回错误信息 `成员 'Bob' 不存在`

### Requirement: 账户路径解析规则
`resolve_account` SHALL 按 `:` 分割路径，逐级从根账户向下查找，返回最终账户的 ID。路径中不允许空段。

#### Scenario: 完整路径解析成功
- **WHEN** 解析路径 `"Expenses:Food:Snack"`
- **THEN** 依次查找 "Expenses" → "Food" → "Snack"，返回 Snack 账户的 ID

#### Scenario: 路径为空段
- **WHEN** 解析路径 `"Expenses::Snack"`
- **THEN** 返回错误 "账户路径包含空段"

### Requirement: 渠道名称分隔符限制
由于 CLI 使用 `->` 和 `&` 作为渠道路径语法分隔符，`channels.name` 及新增渠道输入 SHALL 禁止包含子串 `->` 或 `&`。

#### Scenario: 创建含分隔符的渠道名被拒绝
- **WHEN** 用户尝试创建名为 `支付宝&微信` 的渠道
- **THEN** 系统返回错误 "渠道名称不能包含 & 或 ->"

#### Scenario: 创建普通渠道名成功
- **WHEN** 用户创建名为 `支付宝` 的渠道
- **THEN** 创建成功

### Requirement: 唯一自然键的歧义处理
`members.name` 和 `budgets.name` 在 schema 层声明 `UNIQUE`。解析器按名称查询时最多返回一条记录；若因 schema 异常出现多条，解析器 SHALL 报错而非随机选择。

#### Scenario: 名称唯一时正常解析
- **WHEN** `members` 表中仅有一条 name="Alice" 的记录
- **THEN** `resolve_member` 返回该记录的 MemberId

#### Scenario: 出现重复名称时安全失败
- **WHEN** 数据库中存在两条 name="Alice" 的成员记录
- **THEN** `resolve_member` 返回错误 "成员名称 'Alice' 不唯一，请联系管理员"
