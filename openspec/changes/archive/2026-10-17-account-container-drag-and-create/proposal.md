# Proposal: account-container-drag-and-create

## Why

账户页面无法创建根账户下的一级子账户：根账户（资产、收入等）在页面上只是静态栏目标题，没有 DOM 存在，既不能被选中（新建账户按钮依赖选中态），也没有拖放落点；同时拖放命中测试只认卡片和被拖账户自己的兄弟行，导致任何手势都无法把账户移到根层级。全新数据库下页面完全无法起步（没有任何一级账户时无入口）。

## What Changes

- 将账户页交互模型从"仅卡片可选中/可落点"升级为"容纳框（容器）为一等公民"：
  - 每个类型分区按根账户拆分，分区网格成为根账户的容纳框（带明确 owner），空分区渲染可见的占位区域。
  - 拖动落点从 1 种扩展为 2 种：① 命中卡片 → 被拖账户成为该卡片最后一个子账户（现有行为，保证末位）；② 命中容纳框内部（卡片间空隙、框内空白、空框）→ 显示插入指示线，松手插入该位置。
  - 容纳框内插入按父级分流：同父维持现有 localStorage 同级排序；跨层级/跨到根走 `PUT /api/accounts/{id}/parent` reparent（复用跨类型确认弹窗与防环校验），并把插入位置持久化到新父级的同级顺序。
- 容纳框可选中：点击容纳框空白处选中其 owner（根账户或展开的父账户），不弹账户详情抽屉；新建账户按钮以选中的容器 owner 为父级创建，新账户追加到该容器末位。新建与拖放统一为"向容器放入/创建"。
- 视觉效果与现有行为逻辑保持：卡片网格、层级纸片叠放高亮、拖拽幽灵、插入指示线、悬停自动展开、拖放后选中被拖账户等均不变。
- 调低卡片网格的响应式列数断点至现有的 3/4（600px→450px 进 3 列、900px→675px 进 4 列），让每行容纳更多卡片。
- 修复跨父级移动失败的报错：`PUT /api/accounts/{id}/parent` 当前把服务端错误以 `Err(String)` 返回（axum 将其序列化为 200 OK 纯文本），前端误判成功后在 JSON 解析处抛出 `Unexpected token ... is not valid JSON` 之类的内部信息。改为结构化错误响应（状态码 + 本地化文案），同级重名时明确报出目标父账户下已存在同名子账户。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `account-page`: 根账户分区成为可选中、可放置的容纳框；拖放落点扩展到容器内任意位置（含空容器）并支持跨层级插入；新建账户以选中的容器 owner 为父级，根容器可创建一级账户。
- `account-move`: 移动校验失败的 HTTP 错误响应形态收紧——同级重名等校验失败 MUST 返回明确的状态码与本地化错误文案（重名时报出冲突账户），不再以 200 OK 纯文本返回。

## Impact

- 前端（accounting-web）：
  - `src/composables/useAccountDrag.ts` —— 命中测试重构：卡片命中 → 最内层包含容器 → 容器内最近插槽；空容器支持；防环校验对齐容器 owner。
  - `src/components/layout/AccountGrid.vue` / `AccountRowGroup.vue` —— 容器 DOM 打 `data-*` 标识（owner 账户 id）、空分区占位态、分区按根拆分。
  - `src/views/AccountsView.vue` —— 分区按根渲染、`handleDrop` 同父/跨父分流、选中模型扩展到容器。
  - `src/components/layout/AccountCreateDrawer.vue` —— 基本不变（已按 `parentAccount` 参数化，天然支持根账户作父级）。
- 后端（accounting-api，仅错误映射）：`src/handlers/account.rs` 的 `set_parent` 由 `Result<_, String>` 改为 `Result<_, (StatusCode, String)>`，按错误变体映射状态码与本地化文案（重名报出冲突账户名）；`reparent` 服务层校验规则本身不变（见 `account-move`），系统根账户作父级、跨类型移动均已支持。
- 涉及测试：`AccountGrid.spec.ts`、`AccountsViewRefresh.spec.ts` 等既有前端测试需随行为扩展更新，并为拖拽命中测试逻辑补充单测；`set_parent` 补充 HTTP 层错误形态测试。
