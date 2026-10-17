# Design: account-container-drag-and-create

## Context

账户页现状（见 proposal.md - Why）：

- 根账户只是 `AccountGrid` 的 `<h3>` 标题文本，无 DOM 卡片；网格渲染的是根账户的孩子（`AccountsView.vue` `getChildrenOfType` 把根平铺掉）。
- 选中/创建链路：点击卡片 → `selectedAccountId` → 顶栏"新建账户"（`disabled: !selectedAccount`）→ `AccountCreateDrawer` 固定以选中账户为父。根永远无法被选中。
- 拖拽链路（`useAccountDrag.ts`）：命中测试只查 `.account-card[data-account-id]`；间隙插入的 `parentId` 锁死为 `dragged.parent_id`。任何手势都无法产生"移到根层级"。
- 后端无阻碍：`AccountService::create` 与 `reparent` 均允许以系统根账户为父级，支持跨类型移动；`reparent` 校验：存在、非系统、非根、防环、同级重名。
- 已知缺陷：`set_parent`（accounting-api `handlers/account.rs`）返回 `Result<_, String>`；axum 0.8 将 `Err(String)` 序列化为 **200 OK + text/plain**，前端 `moveAccount` 判定 `res.ok` 后走 `res.json()` 抛 `SyntaxError`——用户看到"json 错误"而非真实原因（如同级重名）。

## Goals / Non-Goals

**Goals:**

- 拖动落点两分类：命中卡片 → 追加为末位子账户；命中容纳框内部 → 插入指示线所示位置（含空容纳框）。
- 容纳框成为一等公民：可选中（不弹详情）、可作新建目标、可接收拖放。
- 保持现有视觉与行为：卡片网格、纸片叠放高亮、拖拽幽灵、悬停自动展开、拖放后选中被拖账户。
- 移动失败时用户可读的错误信息（重名场景明确报重名）。
- 列数断点降至 3/4。

**Non-Goals:**

- 不改 `reparent` 服务层校验规则与数据模型；不做服务端同级顺序存储（localStorage 方案延续）。
- 不重构其他 `Result<_, String>` 端点（通病仅在本变更涉及的端点修复）。
- 不做创建抽屉内的父账户选择器（新建目标 = 选中容器 owner，心智与拖放一致）。

## Decisions

### D1 命中测试统一为"卡片 → 最内层包含容器 → 容器内插槽"

`useAccountDrag.updateHitTest` 重构为三级：① 命中卡片 → `dropTargetId`（成为其最后一个子账户，现有语义保留）；② 未命中卡片 → 收集 `[data-container-id]` 矩形，取包含指针的**面积最小**（最内层）容器；③ 在该容器内按其直接子卡片（`parent_id === containerId`）的行/列几何计算最近插槽（行内左右端与卡片间隙、行间与框内空白归并到最近插槽；空容器整框即落点），产出 `InsertionIndicator`。现有"仅自己兄弟行"逻辑删除。

被否决备选：保留旧逻辑再叠加根落点特判——两套规则并存，边界行为（跨层间隙、空框）仍然缺失，复杂度更高。

防环：容器 owner 是被拖账户自身后代时，该容器与其中卡片一样不可作为落点（对齐现有 `isDescendant(card.id, draggedId)`）。

### D2 容器 DOM 契约：`data-container-account-id`

`AccountGrid.vue` 的分区网格标注 `data-container-account-id="<根账户 id>"`；`AccountRowGroup.vue` 的 `children-container` 标注 owner（即 `expandedAccountId`）。命中测试只依赖 data 属性 + `getBoundingClientRect`，与组件内部结构解耦，可单测。

被否决备选：以 Vue ref 注册容器清单——`useAccountDrag` 是与组件树无关的 composable，DOM 查询与其现有 `collectCards` 风格一致，改动最小。

### D3 分区按根账户拆分

`AccountsView` 从"每类型一个 `AccountGrid`（平铺该类型全部根的孩子）"改为"每根账户一个 `AccountGrid`"，容器与根一一对应（当前每类型恰有一个系统根，视觉不变）。多根类型不再有插入归属歧义。

被否决备选：维持按类型合并 + 单根假设——`groupedAccounts` 数据结构本就允许多根，合并语义在插入定位上无法自洽。

### D4 空容纳框渲染最小占位区

无一级子账户的根（如初始"收入"）其分区网格高度塌缩为 0，无法命中。空容纳框渲染一个最小高度的占位区（虚线样式），可选中、可接收拖放。有子账户但折叠的账户不需要占位——命中卡片即"追加为末位子账户"已覆盖。

### D5 "末位子账户"需要显式排序追缴

无服务端顺序存储；渲染顺序 = localStorage `siblingOrder` + 缺席者按 id 升序兜底。拖到卡片 B 上 / 插入容器末位后，若 moved id 的数值排在中间，视觉上不满足"最后一个"。故 drop-on-card 与末位插入完成后，MUST 将 moved id 追加到目标父级的 `siblingOrder` 末尾（`saveSiblingOrder`）。中间位置插入同理写入对应序。

### D6 drop 分流：同父排序 vs 跨父移动

`handleDrop` 的 `insert` 分支：`ins.parentId === dragged.parent_id` → 现有 `handleSiblingReorder`（纯 localStorage）；否则 → `handleMoveAccount`（复用跨类型确认弹窗、防环校验、乐观更新与回滚），成功后按 D5 持久化插入位置。跨类型确认取消则无任何副作用。

被否决备选：跨父插入也只本地排序——会导致 UI 树与数据库父链不一致，破坏（见 `account-move`）的整树迁移语义。

### D7 选中模型扩展：容纳框可选中，卡片选中不变

`selectedAccountId` 语义不变（新建目标 = 该账户的子账户层），但赋值入口从"仅卡片点击"扩展为"卡片点击 + 容器空白点击"。容器点击处理排除来自卡片的事件（`event.target.closest('.account-card')`），并按 D2 的 data 属性定位容器 owner。容器选中不弹详情抽屉；卡片点击维持现状（选中 + 非 root/非 system 弹详情）。选中容器有可见标识（如高亮描边）。新建按钮的禁用条件不变（无选中即禁用）。

被否决备选：新建按钮常亮 + 抽屉内选父级——与"选中即目标"的现有心智和拖放模型冲突，交互面更大。

### D8 列数断点降至 3/4

`AccountGrid.vue` 容器查询断点 600px→450px（进 3 列）、900px→675px（进 4 列）。`useGridColumns` 只读取 `--grid-columns`，无需改动；`compileRows`/命中测试按实际列数工作，无耦合。纯 CSS 参数调整。

### D9 `set_parent` 错误映射修复

`set_parent` 改为 `Result<Json<AccountDto>, (StatusCode, String)>`（对齐同文件 `rename_account` 的既有模式）：

- `AccountingError::AccountAlreadyExists` → `409 CONFLICT` + 本地化文案（含冲突账户名，Display 已本地化，复用 `account_already_exists` 词条）；
- `AccountNotFound` / `InvalidTransaction`（根/系统账户、防环）→ `400 BAD_REQUEST` + 本地化文案；
- `DatabaseError` → `500 INTERNAL_SERVER_ERROR`。

前端 `moveAccount` 已按 `!res.ok` 抛错并经 `alertDialog(t('accounts.moveFailed', ...))` 展示 message，无需改动即可从"json 解析错误"变为可读原因。补 HTTP 层（router 级）错误形态测试——现有 handler 直调测试只断言 `is_err()`，覆盖不到响应形态。

被否决备选：客户端预检重名（与后端权威校验重复；显示名随请求语言变化，本地副本不可靠）；前端解析 200 文本容错（治标，错在响应形态）。

## Risks / Trade-offs

- [命中测试每 move 事件全量查询容器与卡片矩形] → 数量级为几十个节点，与现状 `collectCards` 相同的一次性 `querySelectorAll` + `getBoundingClientRect`，无布局抖动放大；必要时缓存矩形（展开/滚动才失效）。
- [容器嵌套（分区框 ⊃ 子层级框）导致命中歧义] → 以"包含指针的面积最小"取最内层，嵌套语义唯一确定；子层级容器的空隙属于该层级，分区级空隙只在分区网格自身区域命中。
- [拖拽中 hover 自动展开改变 DOM 使指针下容器突变] → 复用现有 `scheduleExpand` 时序；展开后下一 move 事件重新命中，指示线自然落到新容器。
- [`set_parent` 错误响应形态变化（200→4xx）影响其他调用方] → 前端唯一调用方 `moveAccount` 本就按状态码分支；无其他已知消费方，风险 contained。
- [占位区让"空分区"从不可交互变为可交互] → 视觉上引入新的虚线占位样式，需与"加载完成但为空"和"加载中"区分，避免误认为加载失败。

## Migration Plan

1. 先落拖拽侧（D1–D5、D8）：容器 data 契约、命中测试重构、drop 分流、末位追缴、断点调整。
2. 再落容器选中与新建（D7）：容器点击选中、选中标识、新建抽屉以容器 owner 为父。
3. 后端错误映射（D9）独立提交，可独立回滚。
4. 无数据迁移；回滚即 revert 对应提交，localStorage 格式不变。

## Open Questions

（无）
