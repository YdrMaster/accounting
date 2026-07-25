# web-channel-import-rules

## Why

配置抽屉的渠道 tab 对所有渠道（含系统内置渠道）都显示删除按钮，用户点击后才会吃到后端报错——后端（`channel_force_delete_by_id`）本就拒绝删除内置渠道，前端应直接隐藏入口，与标签 tab 的既有模式对齐。

此外，系统内置渠道（支付宝）关联着账单导入适配器，导入时的"分类→账户"映射（`account_mappings`）目前只能通过 CLI `mapping` 子命令维护，Web 端完全没有入口。需要在配置抽屉的渠道卡片内提供导入规则（映射）配置界面。

## What Changes

- 渠道卡片：系统内置渠道（`is_system=true`）不显示删除按钮；改名、描述、关联账户编辑保持开放（后端 `channel_rename` 本就是安全设计）
- 新增 `mapping` REST API：包装现有 `MappingService`，提供映射的 list / set / delete
- 渠道卡片展开区新增"导入规则"区块：内置成员切换器（映射按 (成员, 渠道) 维度），映射列表（分类→账户）展示、添加（分类输入 + AccountPicker）、删除；对所有渠道开放（不限内置渠道）

## Capabilities

### New Capabilities

- `mapping-api`: 账户映射的 HTTP API——`GET /api/mappings`、`PUT /api/mappings`、`DELETE /api/mappings`，包装 `MappingService`，按 (member_id, channel_id) 查询、按 category 设置/删除映射

### Modified Capabilities

- `config-panel`: 渠道卡片对系统内置渠道隐藏删除按钮；展开的渠道卡片新增"导入规则"区块（成员切换 + 映射 CRUD），仅对关联了导入适配器的渠道显示
- `channel-api`: `GET /api/channels` 的 ChannelDto 新增 `has_import_adapter` 字段，标记渠道是否关联内置账单适配器

## Impact

- `accounting-api`：新增 `handlers/mapping.rs` 路由与 DTO；`ChannelDto` 新增 `has_import_adapter`；复用 `accounting-service` 的 `MappingService`，无数据模型改动
- `accounting-sql`：新增 `channel_names_by_id` 查询（渠道的任一语言名字），用于适配器匹配
- `accounting-web`：`api/client.ts` 新增 mapping 调用、新增 `stores/mapping.ts`、`ConfigPanel.vue` 渠道卡片改动（删除按钮条件渲染 + 导入规则区块按 `has_import_adapter` 显示）、locales 新增文案
- 无 schema 迁移；`accounting` / `accounting-service` 零改动
