# entity-names-i18n

## Purpose

定义账户、标签、渠道、币种、成员、预算六类实体的名字多语言存储模型：名字独立于实体表按语言存储、每语言显示名与回退链、命名空间唯一性、输入命中语义、系统名字保护，以及 API/Web 层的语言参数与本地化行为。

## Requirements

### Requirement: 名字按语言存储
`accounts`、`tags`、`channels`、`commodities`、`members`、`budgets` 六类实体 SHALL NOT 在自身表保存名字；所有名字 SHALL 存储于各自的名字表（`account_names`、`tag_names`、`channel_names`、`commodity_names`、`member_names`、`budget_names`）。每条名字记录 SHALL 标注语言（`en`、`zh-CN`、其他 BCP-47 标签或 `und` 未标注）、`is_system`、以及是否为该实体该语言的显示名。币种符号（`commodities.symbol`）保留在实体表，固定英文大写，不参与多语言。

#### Scenario: 创建账户附带名字
- **WHEN** 用户以中文显示语言创建账户并输入名字 `餐饮`
- **THEN** `accounts` 表新增无名字的实体行，`account_names` 表新增一条 `lang=zh-CN`、`name=餐饮`、`is_display=1` 的记录

#### Scenario: 实体删除级联删除名字
- **WHEN** 一个实体被删除
- **THEN** 其名字表中的所有名字记录一并删除

### Requirement: 每语言显示名与回退链
每个实体的每种语言 SHALL 至多有一条显示名；该语言存在名字时 SHALL 恰有一条显示名。按语言取显示名时 SHALL 依次回退：所选语言的显示名 → 英文显示名 → 中文显示名 → 其余名字按插入序第一条。

#### Scenario: 命中所选语言
- **WHEN** 账户有中文显示名 `现金` 和英文显示名 `Cash`，以中文查询显示名
- **THEN** 返回 `现金`

#### Scenario: 回退到英文
- **WHEN** 账户只有英文显示名 `Cash`，以中文查询显示名
- **THEN** 返回 `Cash`

#### Scenario: 回退到插入序
- **WHEN** 账户只有一条 `und` 语言的名字 `餐饮美食`，以英文查询显示名
- **THEN** 返回 `餐饮美食`

### Requirement: 命名空间唯一性
名字 SHALL 在命名空间内不区分大小写唯一：账户的命名空间为父账户（根账户之间为全局）；标签、渠道、币种、成员、预算为全局。英文名字 SHALL 保存输入的大小写形式用于显示，匹配 SHALL 不区分大小写。创建、改名、新增名字时 SHALL 校验唯一性，撞系统内置名（任何语言）同样拒绝。

#### Scenario: 不同父级下允许同名
- **WHEN** `Assets` 下存在 `Cash`，用户在 `Expenses` 下创建名为 `Cash` 的账户
- **THEN** 创建成功

#### Scenario: 同命名空间撞名被拒绝
- **WHEN** `Assets` 下已存在名字 `Cash`，用户在 `Assets` 下创建名为 `cash` 的账户或为其添加别名 `cash`
- **THEN** 系统拒绝并提示名字已存在

#### Scenario: 撞系统内置名被拒绝
- **WHEN** 用户尝试创建名字为 `现金`（系统账户 `Assets:Cash` 的中文系统名）的账户
- **THEN** 系统拒绝，无论目标父账户是什么语言环境

### Requirement: 输入命中语义
解析实体名字时 SHALL 不区分显示名与语言，命中实体的任意名字即命中该对象。账户路径按 `:` 切段，逐段在当前父级命名空间内大小写不敏感命中。

#### Scenario: 任意语言名字命中
- **WHEN** 账户有名字 `Cash`（en）和 `现金`（zh-CN），用户输入 `现金`
- **THEN** 命中同一账户

#### Scenario: 大小写不敏感命中
- **WHEN** 标签存在名字 `pending`，用户输入 `Pending`
- **THEN** 命中该标签

#### Scenario: 账户路径逐段命中
- **WHEN** 用户输入路径 `资产:现金`，且 `Assets` 有中文名 `资产`、`Cash` 有中文名 `现金`
- **THEN** 解析到 `Assets:Cash` 账户

### Requirement: 系统名字保护
系统内置实体 SHALL 为每种受支持语言持有系统名字（`is_system=1`）。系统名字 SHALL NOT 可删除、SHALL NOT 可修改文本，但 SHALL 允许被设为非显示名。实体级删除保护沿用 `is_system` 规则。

#### Scenario: 删除系统名字被拒绝
- **WHEN** 用户尝试删除系统账户的英文系统名 `Cash`
- **THEN** 系统拒绝

#### Scenario: 系统名字设为非显示
- **WHEN** 用户将 `Cash` 设为非显示名，并将自定义名字 `钱包`（en 语言）设为英文显示名
- **THEN** 以英文查询显示名返回 `钱包`，输入 `Cash` 仍命中该账户

### Requirement: API 语言参数
实体列表与详情接口 SHALL 接受语言参数，按该语言解析显示名后返回。错误信息等用户可见文案 SHALL 按每次请求的语言渲染。CLI SHALL 以 `--lang` 决定显示语言并透传。数据库 SHALL NOT 存储显示语言设置；未提供语言参数时默认 `en`。

#### Scenario: 按参数语言返回显示名
- **WHEN** 以 `lang=zh-CN` 请求账户列表
- **THEN** 响应中的账户名为中文显示名，无中文名的实体按回退链解析

#### Scenario: 默认语言
- **WHEN** 请求未携带语言参数
- **THEN** 按 `en` 解析显示名与文案

#### Scenario: 错误信息按请求语言渲染
- **WHEN** 以 `lang=zh-CN` 发起的请求触发错误
- **THEN** 错误信息以中文渲染，与进程内其他请求的语言互不影响

### Requirement: Web 语言切换与文案本地化
Web 前端 SHALL 使用 vue-i18n 管理 UI 文案，提供语言切换入口并持久化选择。实体名 SHALL 全部来自 API 按所选语言返回的显示名，前端 SHALL NOT 维护内置名映射。既有硬编码 UI 文案 SHALL 迁移至 i18n 资源。

#### Scenario: 切换语言
- **WHEN** 用户将显示语言从中文切换为英文
- **THEN** UI 文案与实体显示名均以英文呈现，刷新后选择保持
