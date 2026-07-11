## 1. 后端 API - 成员改名

- [x] 1.1 在 `accounting-api/src/handlers/member.rs` 中添加 `PUT /api/members/{id}` 端点，接受 `{ name }` body，更新成员名称并返回更新后的 MemberDto
- [x] 1.2 如尚未存在，在数据库 trait 和 SQLite 实现中添加 `member_rename` 或 `member_update` 方法

## 2. 后端 API - 标签更新

- [x] 2.1 在 `accounting-api/src/handlers/tag.rs` 中添加 `PUT /api/tags/{id}` 端点，接受 `{ name, description }` body，更新标签字段并返回更新后的 TagDto
- [x] 2.2 在数据库 trait 和 SQLite 实现中添加 `tag_update` 方法

## 3. 后端 API - 渠道更新扩展

- [x] 3.1 扩展 `accounting-api/src/handlers/channel.rs` 中的 `PUT /api/channels/{id}` handler，支持可选的 `name` 和 `description` 字段（ alongside 现有 `account_id`）
- [x] 3.2 更新 `UpdateChannelRequest` DTO 和 `channel_update` 数据库方法，支持 name 和 description 字段

## 4. 前端 API 客户端

- [x] 4.1 在 `accounting-web/src/api/client.ts` 中添加 `renameMember(id, name)` 函数，调用 `PUT /api/members/{id}`
- [x] 4.2 在 `accounting-web/src/api/client.ts` 中添加 `updateTag(id, data)` 函数，调用 `PUT /api/tags/{id}`
- [x] 4.3 在 `accounting-web/src/api/client.ts` 中更新 `updateChannel(id, data)` 函数（或新增），支持 `PUT /api/channels/{id}` 中的 name 和 description 字段
- [x] 4.4 在 `accounting-web/src/api/client.ts` 中添加 `createChannel(data)` 函数，调用 `POST /api/channels`

## 5. 前端 Store

- [x] 5.1 在 `accounting-web/src/stores/member.ts` 的 `useMemberStore` 中添加 `create`、`rename`、`remove` action
- [x] 5.2 在 `accounting-web/src/stores/tag.ts` 的 `useTagStore` 中添加 `create`、`update`、`remove` action
- [x] 5.3 在 `accounting-web/src/stores/channel.ts` 的 `useChannelStore` 中添加 `create`、`update`、`remove` action

## 6. ConfigPanel 组件

- [x] 6.1 在 `accounting-web/src/components/layout/` 中创建 `ConfigPanel.vue`，包含底部抽屉结构（背景 + 抽屉容器，max-height 66vh，上滑动画）
- [x] 6.2 实现 tab 导航（成员/渠道/标签），带响应式激活 tab 状态
- [x] 6.3 实现成员 tab：列表带行内改名（点击名称 → 输入框 → 回车/失焦保存）、行内新增输入框、删除按钮
- [x] 6.4 实现标签 tab：列表带行内改名、行内新增输入框、删除按钮
- [x] 6.5 实现渠道 tab：可展开卡片列表，单展开行为，展开时显示名称/描述/账户字段
- [x] 6.6 实现渠道卡片行内编辑名称、描述字段
- [x] 6.7 实现渠道关联账户："选择"按钮打开 AccountPickerOverlay，选中的账户通过 API 更新渠道

## 7. 集成

- [x] 7.1 在 `PageSwitcher.vue` 中添加齿轮图标按钮，触发 `openConfig` 事件
- [x] 7.2 在 `ResponsiveShell.vue` 中连接齿轮图标，切换 ConfigPanel 可见性
- [x] 7.3 在 `ResponsiveShell.vue` 模板中添加 ConfigPanel，带 v-if 可见性控制
- [x] 7.4 确保从 ConfigPanel 内部打开 AccountPickerOverlay 时，ConfigPanel 关闭（或管理 z-index 层级）
