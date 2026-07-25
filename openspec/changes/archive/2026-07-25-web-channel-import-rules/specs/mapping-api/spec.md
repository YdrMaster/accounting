# mapping-api

## ADDED Requirements

### Requirement: 映射查询 API
系统 SHALL 提供 `GET /api/mappings?member_id=<id>&channel_id=<id>`，返回指定 (成员, 渠道) 下的全部账户映射。每条映射 SHALL 包含 `member_id`、`channel_id`、`category`、`account_id` 字段。账户显示名 SHALL 由前端按 `account_id` 从账户数据本地解析，API 不拼接名称。

#### Scenario: 查询映射列表
- **WHEN** 请求 `GET /api/mappings?member_id=1&channel_id=2`，且该 (成员, 渠道) 下存在映射 `"Expenses:餐饮美食" → account_id=42`
- **THEN** 返回 JSON 数组，包含 `{ "member_id": 1, "channel_id": 2, "category": "Expenses:餐饮美食", "account_id": 42 }`

#### Scenario: 查询无映射的渠道
- **WHEN** 请求 `GET /api/mappings?member_id=1&channel_id=2`，且该 (成员, 渠道) 下无映射
- **THEN** 返回空 JSON 数组

### Requirement: 映射设置 API
系统 SHALL 提供 `PUT /api/mappings`，请求体包含 `member_id`、`channel_id`、`category`、`account_id`，对 (成员, 渠道, category) 执行 upsert。`account_id` 对应的账户不存在时 SHALL 返回错误。

#### Scenario: 设置映射成功
- **WHEN** 请求 `PUT /api/mappings`，body 为 `{ "member_id": 1, "channel_id": 2, "category": "Expenses:餐饮美食", "account_id": 42 }`，且账户 42 存在
- **THEN** 映射表中插入或更新记录 `(1, 2, "Expenses:餐饮美食") → 42`

#### Scenario: 重复设置覆盖
- **WHEN** 对同一 (成员, 渠道, category) 先后设置 account_id=42 和 account_id=50
- **THEN** 最终该键的 account_id 为 50

#### Scenario: 目标账户不存在
- **WHEN** 请求设置映射，body 中 `account_id` 对应的账户不存在
- **THEN** 返回错误，不创建映射记录

### Requirement: 映射删除 API
系统 SHALL 提供 `DELETE /api/mappings?member_id=<id>&channel_id=<id>&category=<urlencoded>`，删除指定映射。映射不存在时 SHALL 返回错误。

#### Scenario: 删除映射成功
- **WHEN** 请求 `DELETE /api/mappings?member_id=1&channel_id=2&category=Expenses%3A餐饮美食`，且该映射存在
- **THEN** 对应映射记录被删除

#### Scenario: 删除不存在的映射
- **WHEN** 请求删除一个 (成员, 渠道, category) 组合不存在的映射
- **THEN** 返回映射不存在的错误
