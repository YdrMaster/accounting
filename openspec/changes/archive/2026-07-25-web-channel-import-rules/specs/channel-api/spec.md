# channel-api 增量

## MODIFIED Requirements

### Requirement: 列出渠道
API SHALL 提供 `GET /api/channels`，返回包含所有渠道的 JSON 数组，每个渠道包含 `id`、`name`、`description`、`account_id`、`is_system` 和 `has_import_adapter` 字段。`has_import_adapter` 为 `true` 当且仅当该渠道的任一语言名字（`channel_names` 表）能匹配某个内置账单适配器（`builtin_adapters()` 中任一适配器的 `names()`，大小写不敏感）。

#### Scenario: 成功列出渠道
- **WHEN** 向 `/api/channels` 发起 GET 请求
- **THEN** 响应为渠道对象的 JSON 数组

#### Scenario: 内置渠道标记适配器关联
- **WHEN** 列出渠道，内置渠道"支付宝"（英文名 "Alipay"）在结果中
- **THEN** 其 `has_import_adapter` 为 `true`

#### Scenario: 普通渠道无适配器关联
- **WHEN** 列出渠道，用户创建的渠道（如"云闪付"）在结果中
- **THEN** 其 `has_import_adapter` 为 `false`

### Requirement: 创建渠道
API SHALL 提供 `POST /api/channels`，接受包含 `name`、可选 `description` 和可选 `account_id` 的 JSON body。SHALL 返回创建的渠道 id。若 `name` 与既有渠道的任一语言名字重名（不区分大小写），API SHALL 拒绝创建并返回错误，不得修改既有渠道。

#### Scenario: 成功创建渠道
- **WHEN** 向 `/api/channels` 发起 POST 请求，body 为 `{ "name": "云闪付", "description": "日常支付" }`，且该名字未被使用
- **THEN** 创建新渠道，响应为新渠道 id

#### Scenario: 重名拒绝（精确同名）
- **WHEN** 向 `/api/channels` 发起 POST 请求，body 中 `name` 为 "支付宝"（内置渠道既有名字）
- **THEN** 返回"渠道名称已存在"错误，内置渠道不被修改

#### Scenario: 重名拒绝（大小写变体）
- **WHEN** 向 `/api/channels` 发起 POST 请求，body 中 `name` 为 "ALIPAY"（与内置渠道英文名 "Alipay" 仅大小写不同）
- **THEN** 返回"渠道名称已存在"错误，不创建新渠道
