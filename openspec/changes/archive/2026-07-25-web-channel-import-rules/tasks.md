# web-channel-import-rules 任务

## 1. mapping REST API（accounting-api）

- [x] 1.1 新增 `accounting-api/src/handlers/mapping.rs`：`MappingDto`（member_id, channel_id, category, account_id）、查询/设置/删除三个 handler，复用现有 `Arc<AppState>` + `Lang` + `Result<_, String>` 风格
- [x] 1.2 `GET /api/mappings?member_id=&channel_id=`：调用 `MappingService::list` 返回映射数组
- [x] 1.3 `PUT /api/mappings`：校验 `account_id` 对应账户存在（不存在返回错误），然后 upsert 映射
- [x] 1.4 `DELETE /api/mappings?member_id=&channel_id=&category=`：调用 `MappingService::delete`，映射不存在返回错误
- [x] 1.5 在 API 路由注册处挂载 mapping router
- [x] 1.6 为三个端点编写 handler 集成测试（参照 `handlers/account.rs` 既有测试模式），`cargo test -p accounting-api` 通过

## 2. 前端数据层（accounting-web）

- [x] 2.1 `types/api.ts` 新增 `MappingDto` 类型
- [x] 2.2 `api/client.ts` 新增 `fetchMappings` / `upsertMapping` / `deleteMapping`（DELETE 用 `URLSearchParams` 编码 category）
- [x] 2.3 新增 `stores/mapping.ts`：按 `(member_id, channel_id)` 缓存映射，提供 `load / set / remove`，沿用 channel store 的 error 处理模式

## 3. 配置面板 UI（accounting-web）

- [x] 3.1 `ConfigPanel.vue` 渠道卡片删除按钮加 `v-if="!channel.is_system"`（折叠头部）
- [x] 3.2 新增 `components/layout/ChannelMappingSection.vue`：成员切换 `<select>`（memberStore，默认第一个成员）、映射列表（category + 账户名 + 删除按钮）、添加行（分类输入 + AccountPicker + 添加按钮）
- [x] 3.3 账户名按 `account_id` 从 account store 解析，账户不存在时回退显示原始 ID
- [x] 3.4 将 `ChannelMappingSection` 嵌入渠道卡片展开区（关联账户字段下方），切换成员/渠道时重新加载映射
- [x] 3.5 locales（zh-CN / en）新增导入规则相关文案（标题、分类 placeholder 提示 `"<role>:<分类>"` 格式、添加按钮等）
- [x] 3.6 为 mapping store 和 ChannelMappingSection 补充前端测试（参照现有 `__tests__` 模式）

## 4. 验证

- [x] 4.1 `cargo test -p accounting-api` 通过
- [x] 4.2 `npm run test`（accounting-web）通过
- [x] 4.3 手动验证：配置抽屉中内置渠道（支付宝）无删除按钮；展开渠道卡片可切换成员、添加/删除映射，刷新后数据保持

## 5. 导入规则区块仅限适配器渠道

- [x] 5.1 accounting-sql：新增 `channel_names_by_id`（返回渠道全部语言名字），含单测
- [x] 5.2 accounting-api：`ChannelDto` 新增 `has_import_adapter`（任一名字匹配 `find_adapter` 即 true），含 handler 测试（内置支付宝 true / 用户渠道 false）
- [x] 5.3 前端：`ChannelDto` 类型加 `has_import_adapter`，`ConfigPanel.vue` 按此字段条件渲染 `ChannelMappingSection`，补组件测试
- [x] 5.4 `POST /api/channels` 重名拒绝：创建前 `channel_resolve_by_name` 预检（NOCASE 全语言），重名返回 `channel_name_exists` 错误且不改既有渠道；修复 upsert 劫持内置渠道的问题；含 2 个 handler 测试
