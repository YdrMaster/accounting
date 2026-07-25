# add-web-import 设计

## Context

导入链路现状：CLI（`accounting-cli import`）→ `ImportService::import(data, source, member_id)` → 适配器解析 + 映射/fallback 账户 + 待处理标签。服务层有完整测试（含真实支付宝文件 300+ 条）。Web 端缺失的是：HTTP 端点与前端入口。

已确认的关键事实：

- 渠道列表 handler（`accounting-api/src/handlers/channel.rs:28-48`）已用 `channel_names_by_id` 拿全语言名字匹配适配器，计算 `has_import_adapter` —— 导入端点可复用同一逻辑把渠道 id 转成 `ImportService` 需要的 `source` 字符串，对用户改名免疫（内置渠道种子英文名始终留在名字表中）
- `/api/me`（`handlers/me.rs`）从 settings 表读 `current_member_id`，fallback 第一个成员 —— 导入端点复用该逻辑，前端无需传 member_id
- axum 0.8 未开启 `multipart` feature；默认 body 上限 2MB，真实支付宝账单约 184KB，但多年账单可能超限

## Goals / Non-Goals

**Goals:**

- Web 端可从配置面板渠道卡片一键/拖拽导入账单文件
- 端点对渠道改名免疫（按 id 定位，服务端反查适配器匹配名）
- 导入结果以摘要 toast 反馈（成功/跳过数 + 可展开的跳过原因）
- 零新增第三方依赖

**Non-Goals:**

- 不改动 `ImportService`、适配器、映射规则（现成复用）
- 不支持多文件批量导入、不支持导入进度展示（导入为同步请求，万行级账单秒级完成）
- 不做导入后跳转账单列表/待处理筛选的引导（仅 toast 提示）
- 不为 CLI 增加新能力

## Decisions

### D1: 端点按渠道 id 定位，而非前端传渠道名

`POST /api/channels/{id}/import?member_id=<成员id>`。服务端 `channel_names_by_id(id)` 取全语言名字，按大小写不敏感匹配 `builtin_adapters()` 的 `names()`，取第一个命中名作为 `source` 传给 `ImportService`。

- 备选：前端传显示名（`POST /api/import?source=xxx`）。被否决——用户把渠道显示名改成非适配器名（如"我的支付宝"）后，前端传来的名字既解析不到适配器也可能解析不到渠道，导入必坏。id 定位对改名完全免疫。
- 无匹配适配器时返回 400 错误（该渠道不支持导入）。

### D2: 裸字节 body，不启用 multipart

请求 body 即文件字节，axum 用 `Bytes` extractor 接收。前端拖放/文件选择拿到的 `File` 对象可直接 `fetch(url, { method: 'POST', body: file })` 发送。

- 备选：启用 axum `multipart` feature。被否决——单文件上传无需 multipart 的字段语义，省一个 feature 依赖，前后端都更简单。
- 该路由单独挂 `DefaultBodyLimit::max(32 * 1024 * 1024)`（32MB），避免多年账单触发默认 2MB 上限；不影响其他路由。

### D3: 成员由调用方显式指定

`member_id` 为必填 query 参数，服务端校验成员存在（缺失或不存在返回 4xx）。系统不存在"当前用户"概念（`GET/PUT /api/me` 从未被前端使用，另行移除），导入成员必须在 UI 上显式选择。

- 备选：复用 me.rs 的 settings `current_member_id` 解析。被否决——该 settings 键无任何写入方，永远 fallback 第一个成员，是伪概念。

### D4: 响应 DTO 镜像 ImportResult

```json
{
  "imported": 320,
  "skipped": 2,
  "pending_tag_name": "pending",
  "errors": [{ "row": 15, "detail": "..." }]
}
```

`errors[].detail` 为 `AdaptError` 的人类可读描述（服务端已本地化）。不返回 `transaction_ids`——Web 端没有逐条确认的场景，摘要不需携带数百个 id。

### D5: 前端交互——卡片即 drop zone，按钮替代删除位

- `has_import_adapter=true` 的卡片头部，在原删除按钮位置渲染"导入"按钮；`is_system=false` 的渠道维持删除按钮；无适配器渠道该位置为空
- 整个卡片（`channel-card`）作为 drop zone：`dragover` 时高亮边框，`drop` 时取 `dataTransfer.files[0]`；点击"导入"按钮触发隐藏 `<input type="file">`
- 文件选定/拖入后**先弹出成员确认对话框**：成员下拉（默认第一项，可改）+ 确认/取消；确认才携带 `member_id` 发起导入，取消则放弃
- 导入中按钮呈 loading 态并忽略重复拖放/点击
- 完成后 toast："导入 N 条，跳过 M 条"；有跳过时 toast 内提供可展开的逐行原因列表
- toast 组件：项目内若无现成 toast，实现一个最小的单条提示（固定定位、数秒自动消失），不引入组件库

### D6: 状态归属

channel store 新增 `importFile(channelId, file): Promise<ImportResultDto>` action 与 `importingChannelId` 状态；API 细节封装在 `client.ts`。ConfigPanel 只处理拖放事件与 toast 展示。

## Risks / Trade-offs

- [同步请求导入大文件可能耗时较长，前端 fetch 长时间挂起] → 账单量级（万行内）实测秒级完成；toast 前先显示导入中 loading。若未来出现超时问题再考虑异步任务化
- [用户把非账单文件拖入卡片] → 适配器解析失败，`ImportService` 返回 Parse 错误，toast 显示失败原因；不产生脏数据（解析失败即整体失败，逐行错误则跳过并计入 skipped）
- [重复导入同一文件产生重复交易] → 与 CLI 行为一致（`ImportService` 目前无去重）；本变更不引入去重，保持与既有能力语义相同
- [toast 组件与现有 UI 风格不一致] → 复用 ConfigPanel 现有样式变量，最小实现
