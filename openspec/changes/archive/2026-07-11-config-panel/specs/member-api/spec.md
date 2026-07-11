## 新增需求

### 需求：列出成员
API SHALL 提供 `GET /api/members`，返回包含所有成员的 JSON 数组，每个成员包含 `id` 和 `name` 字段。

#### 场景：成功列出成员
- **当** 向 `/api/members` 发起 GET 请求
- **则** 响应为 `{ id: number, name: string }` 对象的 JSON 数组

### 需求：创建成员
API SHALL 提供 `POST /api/members`，接受包含 `name` 字段的 JSON body。SHALL 返回创建的成员。

#### 场景：成功创建成员
- **当** 向 `/api/members` 发起 POST 请求，body 为 `{ "name": "张三" }`
- **则** 创建新成员，响应为 `{ "id": <new_id>, "name": "张三" }`，状态码 200

### 需求：重命名成员
API SHALL 提供 `PUT /api/members/{id}`，接受包含 `name` 字段的 JSON body。SHALL 更新成员名称并返回更新后的成员。

#### 场景：成功重命名成员
- **当** 向 `/api/members/1` 发起 PUT 请求，body 为 `{ "name": "李四" }`
- **则** 成员名称更新，响应为 `{ "id": 1, "name": "李四" }`

#### 场景：重命名不存在的成员
- **当** 向 `/api/members/999` 发起 PUT 请求，body 为 `{ "name": "test" }`
- **则** 响应为错误及相应状态码

### 需求：删除成员
API SHALL 提供 `DELETE /api/members/{id}` 删除成员。

#### 场景：成功删除成员
- **当** 向 `/api/members/1` 发起 DELETE 请求
- **则** 成员被删除，响应状态码 200
