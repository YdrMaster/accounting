# channel-api 增量

## ADDED Requirements

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
