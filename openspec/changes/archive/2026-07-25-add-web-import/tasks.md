# add-web-import 任务

## 1. 后端：导入端点

- [x] 1.1 在 `accounting-api/src/handlers/me.rs` 中把"读 settings 的 `current_member_id` → fallback 第一个成员"逻辑抽为共享函数（供 `get_me` 与导入 handler 复用），并补充单元测试
- [x] 1.2 在 `accounting-api/src/dto.rs` 新增导入结果 DTO（`imported`、`skipped`、`pending_tag_name`、`errors: [{ row, detail }]`），错误 detail 本地化
- [x] 1.3 在 `accounting-api/src/handlers/channel.rs` 新增 `import_bill` handler：`Path(id)` + `Bytes` body；`channel_names_by_id` 反查适配器命中名（无命中返回 400，渠道不存在返回错误）；调用 `ImportService::import`；映射 `ImportError` 为 HTTP 错误
- [x] 1.4 在 channel 路由注册 `POST /api/channels/{id}/import`，并对该路由挂 `DefaultBodyLimit::max(32MB)`
- [x] 1.5 编写 handler 测试：成功导入（种子支付宝渠道 + CSV 字节）、无适配器渠道 400、不存在的渠道、改名后仍可导入、成员取当前用户、解析失败
- [x] 1.6 `cargo test -p accounting-api` 与 `cargo clippy` 通过

## 2. 前端：API 与状态

- [x] 2.1 `accounting-web/src/types/api.ts` 新增 `ImportResultDto` 类型
- [x] 2.2 `accounting-web/src/api/client.ts` 新增 `importBill(channelId, file)`：以 `File` 为 body POST 到 `/api/channels/{id}/import`
- [x] 2.3 `accounting-web/src/stores/channel.ts` 新增 `importingChannelId` 状态与 `importFile(channelId, file)` action（含错误处理），补充 store 测试

## 3. 前端：配置面板交互

- [x] 3.1 `ConfigPanel.vue` 渠道卡片头部操作位按类型渲染：`is_system=false` → 删除按钮；`has_import_adapter=true` → 导入按钮；其余 → 无按钮
- [x] 3.2 实现点击导入：隐藏 `<input type="file">` 触发文件选择，选中即调用 store 导入
- [x] 3.3 实现卡片 drop zone：`dragover`/`dragleave`/`drop` 事件与高亮样式，仅 `has_import_adapter` 卡片响应；导入中忽略重复触发
- [x] 3.4 实现最小 toast：成功摘要"导入 N 条，跳过 M 条"（有跳过时可展开逐行行号+原因）、失败错误提示、数秒自动消失
- [x] 3.5 `locales/zh-CN.ts` 与 `locales/en.ts` 新增导入相关文案（按钮、toast、loading）
- [x] 3.6 更新/新增组件测试：按钮按渠道类型渲染、点击触发选择、拖放触发导入、导入中防重、toast 摘要与展开；`ConfigPanel.spec.ts` 等相关测试通过

## 4. 验证

- [x] 4.1 `cargo test --workspace` 通过
- [x] 4.2 前端测试与 lint 通过（`npm run test` / `npm run lint`，以 accounting-web 实际脚本为准）
- [x] 4.3 端到端手测：启动 API + 前端，用 `resources/支付宝交易明细(20260325-20260625).csv` 在配置面板拖入支付宝渠道卡片，确认 toast 摘要正确、交易带待处理标签

## 5. 返工：成员显式指定（移除"当前用户"依赖）

- [x] 5.1 后端：`import_bill` 改为必填 query 参数 `member_id` 并校验成员存在，删除 `resolve_current_member_id` 调用；更新 handler 测试（显式成员、成员不存在 4xx）
- [x] 5.2 前端：`client.importBill(channelId, memberId, file)` 与 store `importFile` 增加 memberId 参数；更新 store 测试
- [x] 5.3 前端：ConfigPanel 文件选定/拖入后弹成员确认对话框（下拉默认第一项 + 确认/取消），确认才导入；i18n 文案；组件测试（确认发起、取消不发、可改成员）
- [x] 5.4 回归：`cargo test --workspace`、前端 `npx vitest run && npm run lint`、e2e curl 冒烟（带/不带 member_id）
