# add-web-import

## Why

账单导入能力目前只有 CLI 入口（`accounting-cli import`），Web 端用户无法导入交易。后端 `ImportService` 已完整就绪（适配器解析、账户映射、fallback 账户、待处理标签），渠道列表 API 也已返回 `has_import_adapter` 标志，缺的只是 HTTP 端点和前端入口。同时，配置面板中内置渠道（`is_system=true`）不显示删除按钮，该位置正好可用于放置导入入口。

## What Changes

- 新增 `POST /api/channels/{id}/import` 端点：请求 body 为账单文件原始字节，服务端按渠道 id 反查全语言名字、匹配内置适配器后调用 `ImportService`，成员取当前用户（复用 `/api/me` 的 `current_member_id` 解析逻辑），返回导入摘要（成功数、跳过数、错误明细、待处理标签名）
- 配置面板渠道卡片：对 `has_import_adapter=true` 的渠道，在原删除按钮位置显示"导入"按钮；无适配器的渠道该位置保持为空（内置渠道仍不显示删除按钮）
- 渠道卡片支持拖拽导入：将账单文件拖到卡片上高亮提示，松手即触发导入；点击"导入"按钮打开系统文件选择框，选择即导入
- 导入完成后显示摘要 toast（"导入 N 条，跳过 M 条"），跳过原因可展开查看
- 前端 `client.ts` 与 channel store 新增导入调用，zh-CN / en 新增相关文案

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `channel-api`: 新增账单导入端点 `POST /api/channels/{id}/import`，接收原始文件字节，按渠道 id 定位适配器并执行导入，返回导入摘要
- `config-panel`: 渠道卡片新增导入入口——`has_import_adapter=true` 的卡片显示导入按钮并作为文件拖放目标，导入后显示摘要反馈

## Impact

- **accounting-api**：`handlers/channel.rs` 新增导入 handler 与路由；`dto.rs` 新增导入结果 DTO；成员解析逻辑从 `me.rs` 抽取复用；该路由需放宽 axum 默认 body 大小限制
- **accounting-web**：`ConfigPanel.vue`（导入按钮、拖放交互、toast）、`stores/channel.ts`、`api/client.ts`、`types/api.ts`、`locales/zh-CN.ts`、`locales/en.ts`
- **不改动**：`accounting-service` 的 `ImportService`、账单适配器、账户映射逻辑（全部现成复用）
- **依赖**：无新增第三方依赖（裸字节 body，无需 axum multipart feature）
