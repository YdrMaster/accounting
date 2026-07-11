## 新增需求

### 需求：列出标签
API SHALL 提供 `GET /api/tags`，返回包含所有标签的 JSON 数组，每个标签包含 `id`、`name`、`description` 和 `is_system` 字段。

#### 场景：成功列出标签
- **当** 向 `/api/tags` 发起 GET 请求
- **则** 响应为标签对象的 JSON 数组

### 需求：创建标签
API SHALL 提供 `POST /api/tags`，接受包含 `name` 和可选 `description` 的 JSON body。如果同名标签已存在，SHALL 返回现有标签。否则 SHALL 返回创建的标签。

#### 场景：成功创建标签
- **当** 向 `/api/tags` 发起 POST 请求，body 为 `{ "name": "餐饮", "description": "日常吃饭" }`
- **则** 创建新标签，响应为标签对象

#### 场景：创建重复标签
- **当** 向 `/api/tags` 发起 POST 请求，name 已存在
- **则** 返回现有标签，不创建重复

### 需求：更新标签
API SHALL 提供 `PUT /api/tags/{id}`，接受包含可选 `name` 和可选 `description` 字段的 JSON body。SHALL 仅更新提供的字段并返回更新后的标签。

#### 场景：更新标签名称
- **当** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "name": "新名称" }`
- **则** 标签名称更新，响应为更新后的标签对象

#### 场景：更新标签描述
- **当** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "description": "新描述" }`
- **则** 标签描述更新

#### 场景：同时更新名称和描述
- **当** 向 `/api/tags/1` 发起 PUT 请求，body 为 `{ "name": "新名称", "description": "新描述" }`
- **则** 两个字段都更新

### 需求：删除标签
API SHALL 提供 `DELETE /api/tags/{id}` 删除标签。系统标签 SHALL 不可删除。

#### 场景：成功删除标签
- **当** 向 `/api/tags/1` 发起 DELETE 请求，且标签不是系统标签
- **则** 标签被删除，响应状态码 200

#### 场景：删除系统标签
- **当** 对系统标签发起 DELETE 请求
- **则** 响应为错误，指示系统标签不可删除
