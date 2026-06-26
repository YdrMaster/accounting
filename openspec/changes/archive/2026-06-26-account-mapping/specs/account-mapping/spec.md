## ADDED Requirements

### Requirement: 账户映射数据模型
系统 SHALL 提供 `account_mappings` 表，存储绑定在 `(member_id, channel_id)` 上的分类字符串→账户编号映射。表结构包含 `member_id`、`channel_id`、`category`、`account_id`、`created_at`、`updated_at` 字段，主键为 `(member_id, channel_id, category)`。`member_id` 和 `channel_id` 分别外键引用 `members(id)` 和 `channels(id)`，声明 `ON DELETE CASCADE`。`account_id` 外键引用 `accounts(id)`，不声明 `ON DELETE`（默认 RESTRICT）。

#### Scenario: 创建映射记录
- **WHEN** 为成员 1、渠道"支付宝"设置 `"收支:餐饮美食" → AccountId(42)`
- **THEN** `account_mappings` 表中插入一行 `(member_id=1, channel_id=1, category="收支:餐饮美食", account_id=42)`

#### Scenario: 同一 (成员, 渠道, category) 重复设置覆盖
- **WHEN** 对同一 `(member_id=1, channel_id=1, category="收支:餐饮美食")` 先后设置 account_id=42 和 account_id=50
- **THEN** 最终映射表中该键的 account_id 为 50（覆盖而非报错）

#### Scenario: 删除成员时级联清理映射
- **WHEN** 删除成员 1
- **THEN** `account_mappings` 表中所有 `member_id=1` 的记录被自动删除

#### Scenario: 删除渠道时级联清理映射
- **WHEN** 删除渠道"支付宝"
- **THEN** `account_mappings` 表中所有 `channel_id` 为该渠道的记录被自动删除

### Requirement: 映射 category 格式
`category` 字段 SHALL 使用 `"角色:原始分类"` 格式，角色为 `"收支"` 或 `"资产"`。收支侧映射 key 格式为 `"收支:<分类名>"`，资产侧映射 key 格式为 `"资产:<付款方式名>"`。

#### Scenario: 收支侧映射 key
- **WHEN** 适配器输出 `role=IncomeExpense, category="餐饮美食"`
- **THEN** 映射 key 为 `"收支:餐饮美食"`

#### Scenario: 资产侧映射 key
- **WHEN** 适配器输出 `role=Asset, category="蚂蚁宝藏信用卡"`
- **THEN** 映射 key 为 `"资产:蚂蚁宝藏信用卡"`

### Requirement: 映射 CRUD 服务
系统 SHALL 在 `accounting-service` crate 中提供 `MappingService`，支持映射的创建/更新（set）、查询（list）、删除（delete）操作。set 操作 SHALL 接受 `(member_id, channel_id, category, account_path: String)` 参数，根据 account_path 查找对应的 AccountId，账户不存在时返回错误。

#### Scenario: 设置映射成功
- **WHEN** 调用 `mapping_set(member_id=1, channel_id=1, category="收支:餐饮美食", account_path="Expenses:餐饮")`
- **THEN** 系统查找 "Expenses:餐饮" 的 AccountId，插入或更新映射记录

#### Scenario: 设置映射目标账户不存在
- **WHEN** 调用 `mapping_set(category="收支:餐饮美食", account_path="Expenses:不存在")`
- **THEN** 系统返回 AccountNotFound 错误，不创建映射记录

#### Scenario: 列出映射
- **WHEN** 调用 `mapping_list(member_id=1, channel_id=1)`
- **THEN** 返回该 (成员, 渠道) 下所有映射记录

#### Scenario: 删除映射
- **WHEN** 调用 `mapping_delete(member_id=1, channel_id=1, category="收支:餐饮美食")`
- **THEN** 删除对应的映射记录

#### Scenario: 删除不存在的映射
- **WHEN** 调用 `mapping_delete(member_id=1, channel_id=1, category="收支:不存在")`
- **THEN** 系统返回映射不存在的错误

### Requirement: CLI mapping 子命令
系统 SHALL 在 CLI 中新增 `mapping` 子命令，支持 `set`、`list`、`delete` 三个子操作。`set` 接受 `--member`、`--channel`、`--category`、`--account` 参数；`list` 接受 `--member`、`--channel` 参数；`delete` 接受 `--member`、`--channel`、`--category` 参数。

#### Scenario: CLI 设置映射
- **WHEN** 用户执行 `accounting mapping set --member 1 --channel 支付宝 --category "收支:餐饮美食" --account "Expenses:餐饮"`
- **THEN** 系统创建映射并输出成功信息

#### Scenario: CLI 列出映射
- **WHEN** 用户执行 `accounting mapping list --member 1 --channel 支付宝`
- **THEN** 系统以表格形式输出该 (成员, 渠道) 下所有映射

#### Scenario: CLI 删除映射
- **WHEN** 用户执行 `accounting mapping delete --member 1 --channel 支付宝 --category "收支:餐饮美食"`
- **THEN** 系统删除映射并输出成功信息

### Requirement: 映射引用检查
删除账户时，系统 SHALL 检查 `account_mappings` 表中是否有引用该账户的映射记录。若有，SHALL 拒绝删除并返回错误。

#### Scenario: 删除被映射引用的账户
- **WHEN** 尝试删除 AccountId(42)，且 `account_mappings` 中存在 `account_id=42` 的记录
- **THEN** 系统拒绝删除，返回"该账户被账户映射引用，请先删除相关映射"错误

#### Scenario: 删除未被映射引用的账户
- **WHEN** 尝试删除 AccountId(42)，且 `account_mappings` 中不存在 `account_id=42` 的记录
- **THEN** 系统正常执行后续删除检查