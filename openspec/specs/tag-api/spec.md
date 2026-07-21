# 标签 API

## Purpose

提供标签实体的 REST API，支持列出、创建、更新和删除操作。标签用于为交易打标分类（如“餐饮”“待处理”），本能力解决前端及外部调用方对标签数据的增删改查需求；创建时同名标签去重复用，系统标签受保护不可删除。

## Requirements

### Requirement: 列出标签
API SHALL 提供 `GET /api/tags`，返回包含所有标签的 JSON 数组，每个标签包含 `id`、`name`、`description` 和 `is_system` 字段。

#### Scenario: 成功列出标签
- **WHEN** 向 `/api/tags` 发起 GET 请求
- **THEN** 响应为标签对象的 JSON 数组

### Requirement: 创建标签
API SHALL 提供 `POST /api/tags`，接受包含 `name` 和可选 `description` 的 JSON body。如果同名标签已存在，SHALL 返回现有标签。否则 SHALL 返回创建的标签。

#### Scenario: 成功创建标签
- **WHEN** 向 `/api/tags` 发起 POST 请求，body 为 `{ "name": "餐饮", "description": "日常吃饭" }`
- **THEN** 创建新标签，响应为标签对象

#### Scenario: 创建重复标签
- **WHEN** 向 `/api/tags` 发起 POST 请求，name 已存在
- **THEN** 返回现有标签，不创建重复

### Requirement: 更新标签
API SHALL 提供 `PUT /api/tags/{id}`，接受包含可选 `name` 和可选 `description` 字段的 JSON body。SHALL 仅更新提供的字段并返回更新后的标签。

#### Scenario: 更新标签名称
- **WHEN** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "name": "新名称" }`
- **THEN** 标签名称更新，响应为更新后的标签对象

#### Scenario: 更新标签描述
- **WHEN** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "description": "新描述" }`
- **THEN** 标签描述更新

#### Scenario: 同时更新名称和描述
- **WHEN** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "name": "新名称", "description": "新描述" }`
- **THEN** 两个字段都更新

### Requirement: 删除标签
API SHALL 提供 `DELETE /api/tags/{id}` 删除标签。系统标签 SHALL 不可删除。

#### Scenario: 成功删除标签
- **WHEN** 向 `/api/tags/1` 发起 DELETE 请求，且标签不是系统标签
- **THEN** 标签被删除，响应状态码 200

#### Scenario: 删除系统标签
- **WHEN** 对系统标签发起 DELETE 请求
- **THEN** 响应为错误，指示系统标签不可删除
