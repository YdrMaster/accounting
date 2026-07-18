# cli-natural-key-resolution (delta)

## MODIFIED Requirements

### Requirement: 账户路径解析规则
`resolve_account` SHALL 按 `:` 分割路径，逐级从根账户向下查找，返回最终账户的 ID。路径中不允许空段。每段 SHALL 在当前父级命名空间内对 `account_names` 做不区分大小写的命中，命中账户的任意名字（不限语言、不限显示名）即命中该账户。账户命名空间为父账户作用域，根账户之间为全局（见 `entity-names-i18n`）。

#### Scenario: 完整路径解析成功
- **WHEN** 解析路径 `"Expenses:Food:Snack"`
- **THEN** 依次查找 "Expenses" → "Food" → "Snack"，返回 Snack 账户的 ID

#### Scenario: 任意语言名字命中
- **WHEN** 解析路径 `"资产:现金"`，且 `Assets` 有中文名 `资产`、`Cash` 有中文名 `现金`
- **THEN** 返回 `Assets:Cash` 账户的 ID

#### Scenario: 大小写不敏感命中
- **WHEN** 解析路径 `"assets:cash"`
- **THEN** 返回 `Assets:Cash` 账户的 ID

#### Scenario: 路径为空段
- **WHEN** 解析路径 `"Expenses::Snack"`
- **THEN** 返回错误 "账户路径包含空段"

### Requirement: 唯一自然键的歧义处理
成员和预算的名字唯一性由 `member_names` / `budget_names` 的全局命名空间唯一性保证（创建时在应用层校验，见 `entity-names-i18n`）。解析器按名字查询时最多命中一条记录；若因数据异常出现多条，解析器 SHALL 报错而非随机选择。

#### Scenario: 名称唯一时正常解析
- **WHEN** `member_names` 表中仅有一条 name="Alice" 的记录
- **THEN** `resolve_member` 返回该记录的 MemberId

#### Scenario: 出现重复名称时安全失败
- **WHEN** 数据库中存在两条命中 "Alice" 的成员名字记录
- **THEN** `resolve_member` 返回错误 "成员名称 'Alice' 不唯一，请联系管理员"
