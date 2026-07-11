# 渠道 API

## 目的

提供渠道实体的 REST API，支持列出、创建、更新和删除操作。

## 需求

### 需求：列出渠道
API SHALL 提供 `GET /api/channels`，返回包含所有渠道的 JSON 数组，每个渠道包含 `id`、`name`、`description`、`account_id` 和 `is_system` 字段。

#### 场景：成功列出渠道
- **当** 向 `/api/channels` 发起 GET 请求
- **则** 响应为渠道对象的 JSON 数组

### 需求：创建渠道
API SHALL 提供 `POST /api/channels`，接受包含 `name`、可选 `description` 和可选 `account_id` 的 JSON body。SHALL 返回创建的渠道 id。

#### 场景：成功创建渠道
- **当** 向 `/api/channels` 发起 POST 请求，body 为 `{ "name": "支付宝", "description": "日常支付" }`
- **则** 创建新渠道，响应为新渠道 id

### 需求：更新渠道
API SHALL 提供 `PUT /api/channels/{id}`，接受包含可选 `name`、可选 `description` 和可选 `account_id` 字段的 JSON body。SHALL 仅更新提供的字段。

#### 场景：更新渠道名称和描述
- **当** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "name": "新支付宝", "description": "更新后的描述" }`
- **则** 渠道的名称和描述更新

#### 场景：仅更新渠道 account_id
- **当** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "account_id": 5 }`
- **则** 仅更新渠道的 account_id（与现有行为向后兼容）

#### 场景：更新所有渠道字段
- **当** 向 `/api/channels/1` 发起 PUT 请求，body 为 `{ "name": "新名称", "description": "新描述", "account_id": 3 }`
- **则** 所有三个字段更新

### 需求：删除渠道
API SHALL 提供 `DELETE /api/channels/{id}` 删除渠道。如果渠道被交易 channel_paths 引用，SHALL 拒绝删除。

#### 场景：成功删除渠道
- **当** 向 `/api/channels/1` 发起 DELETE 请求，且渠道未被使用
- **则** 渠道被删除，响应状态码 200

#### 场景：删除正在使用的渠道
- **当** 向 `/api/channels/1` 发起 DELETE 请求，且渠道被交易引用
- **则** 响应为错误，指示渠道正在使用中
