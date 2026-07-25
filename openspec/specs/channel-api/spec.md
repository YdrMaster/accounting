# 渠道 API

## Purpose

渠道用于标记交易的来源（如支付宝、微信支付），并可关联到具体账户。本能力提供渠道实体的 REST API，支持列出、创建、更新和删除操作，供前端管理与维护渠道数据。

## Requirements

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

### Requirement: 更新渠道
API SHALL 提供 `PUT /api/channels/{id}`，接受包含可选 `name`、可选 `description` 和可选 `account_id` 字段的 JSON body。SHALL 仅更新提供的字段。

#### Scenario: 更新渠道名称和描述
- **WHEN** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "name": "新支付宝", "description": "更新后的描述" }`
- **THEN** 渠道的名称和描述更新

#### Scenario: 仅更新渠道 account_id
- **WHEN** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "account_id": 5 }`
- **THEN** 仅更新渠道的 account_id（与现有行为向后兼容）

#### Scenario: 更新所有渠道字段
- **WHEN** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "name": "新名称", "description": "新描述", "account_id": 3 }`
- **THEN** 所有三个字段更新

### Requirement: 删除渠道
API SHALL 提供 `DELETE /api/channels/{id}` 删除渠道。如果渠道被交易 channel_paths 引用，SHALL 拒绝删除。

#### Scenario: 成功删除渠道
- **WHEN** 向 `/api/channels/1` 发起 DELETE 请求，且渠道未被使用
- **THEN** 渠道被删除，响应状态码 200

#### Scenario: 删除正在使用的渠道
- **WHEN** 向 `/api/channels/1` 发起 DELETE 请求，且渠道被交易引用
- **THEN** 响应为错误，指示渠道正在使用中

### Requirement: 导入账单文件
API SHALL 提供 `POST /api/channels/{id}/import?member_id=<成员id>`，请求 body 为账单文件原始字节（非 multipart、非 JSON），query 参数 `member_id` 显式指定导入成员（必填）。服务端 SHALL 通过 `channel_names_by_id` 取该渠道的全语言名字，按大小写不敏感匹配内置账单适配器（`builtin_adapters()` 的 `names()`），以第一个命中名作为 source 调用 `ImportService` 执行导入。响应 SHALL 为 JSON：`{ imported, skipped, pending_tag_name, errors: [{ row, detail }] }`，其中 `detail` 为本地化的人类可读错误描述；响应 SHALL NOT 携带逐条交易 id。该路由 SHALL 放宽默认请求体大小限制以容纳多年账单文件（至少 32MB）。

#### Scenario: 成功导入支付宝账单
- **WHEN** 向 `/api/channels/{支付宝渠道id}/import?member_id=1` 发起 POST 请求，body 为支付宝导出的 CSV 文件字节
- **THEN** 服务端以命中适配器的渠道名调用 `ImportService`，响应 `imported` 等于成功导入条数，`skipped` 等于跳过条数，`errors` 逐行给出跳过原因

#### Scenario: 渠道无匹配适配器
- **WHEN** 向 `/api/channels/{无适配器渠道id}/import?member_id=1` 发起 POST 请求
- **THEN** 返回 400 错误，提示该渠道不支持导入，不产生任何交易

#### Scenario: 渠道不存在
- **WHEN** 向 `/api/channels/{不存在的id}/import?member_id=1` 发起 POST 请求
- **THEN** 返回错误，提示渠道不存在

#### Scenario: 改名后的内置渠道仍可导入
- **WHEN** 用户已将内置渠道"支付宝"的中文显示名改为其他名字，向该渠道 id 的 import 端点发起 POST 请求（带合法 member_id）
- **THEN** 服务端仍能通过名字表中保留的其他语言名字匹配到适配器，导入成功

#### Scenario: 导入成员为显式指定的成员
- **WHEN** 向 `/api/channels/{支付宝渠道id}/import?member_id={成员B的id}` 发起 POST 请求
- **THEN** 导入产生的交易 `member_id` 全部为成员 B

#### Scenario: member_id 缺失或成员不存在
- **WHEN** 请求缺少 `member_id` 参数，或 `member_id` 指向不存在的成员
- **THEN** 返回 4xx 错误，不产生任何交易

#### Scenario: 解析失败的文件
- **WHEN** 向 import 端点 POST 一个非账单格式（或编码无法识别）的文件
- **THEN** 返回错误，提示解析失败，不产生任何交易
