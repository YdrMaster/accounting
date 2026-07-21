# 成员 API

## Purpose

提供成员实体的 REST API，支持列出、创建、重命名和删除操作。成员代表参与记账的个人，用于在交易与共享账本中标识资金归属，本能力为前端与服务端提供统一的成员管理接口。

## Requirements

### Requirement: 列出成员
API SHALL 提供 `GET /api/members`，返回包含所有成员的 JSON 数组，每个成员包含 `id` 和 `name` 字段。

#### Scenario: 成功列出成员
- **WHEN** 向 `/api/members` 发起 GET 请求
- **THEN** 响应为 `{ id: number, name: string }` 对象的 JSON 数组

### Requirement: 创建成员
API SHALL 提供 `POST /api/members`，接受包含 `name` 字段的 JSON body。SHALL 返回创建的成员。

#### Scenario: 成功创建成员
- **WHEN** 向 `/api/members` 发起 POST 请求，body 为 `{ "name": "张三" }`
- **THEN** 创建新成员，响应为 `{ "id": <new_id>, "name": "张三" }`，状态码 200

### Requirement: 重命名成员
API SHALL 提供 `PUT /api/members/{id}`，接受包含 `name` 字段的 JSON body。SHALL 更新成员名称并返回更新后的成员。

#### Scenario: 成功重命名成员
- **WHEN** 向 `/api/members/1` 发起 PUT 请求，body 为 `{ "name": "李四" }`
- **THEN** 成员名称更新，响应为 `{ "id": 1, "name": "李四" }`

#### Scenario: 重命名不存在的成员
- **WHEN** 向 `/api/members/999` 发起 PUT 请求，body 为 `{ "name": "test" }`
- **THEN** 响应为错误及相应状态码

### Requirement: 删除成员
API SHALL 提供 `DELETE /api/members/{id}` 删除成员。

#### Scenario: 成功删除成员
- **WHEN** 向 `/api/members/1` 发起 DELETE 请求
- **THEN** 成员被删除，响应状态码 200
