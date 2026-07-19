# account-drag-reposition 设计

## Context

账户树的父子关系由 `accounts.parent_id` 表示，另有闭包表 `account_ancestors (account_id, ancestor_id, depth)` 维护完整祖先链（创建时由 `account_create_with_closure` 写入）。账户类型**不是存储字段**，而是沿父链找到根账户后按根账户名推导（`AccountType::from_str`）。当前后端没有任何更新 `parent_id` 的方法，`UPDATE accounts` 仅用于 billing/repayment day、close/reopen。同级账户无排序字段，按 `ORDER BY id`（即创建顺序）返回。

前端账户页为 `AccountsView.vue` + `AccountGrid`/`AccountRowGroup`/`AccountCard` 递归卡片网格，展开态由 `expandedPath` 驱动，`utils/accountGrid.ts` 的 `compileRows`/`buildRowTree` 把展开的账户树编译成行结构。项目无拖放库，现有拖动手势（`PageSwitcher.vue`）为手写 pointer 事件。新建抽屉 `AccountCreateDrawer.vue` 目前用 `<select>` 选择父账户（前端自做的类型过滤），创建走 `POST /api/accounts`。

关键决策已在探索阶段与需求方确认：

- 根账户无卡片展示，天然不可拖动；`is_system` 账户禁止移动。
- 允许跨类型移动（前端确认弹窗兜底）；系统无关联状态，改类型后余额等重新计算即可。
- 同级顺序仅存 localStorage，不加数据库排序字段。
- 支持移动已关闭账户。
- 移动即整棵子树路径变更；移动前后视为不同账户，不做历史路径兼容。

## Goals / Non-Goals

**Goals:**

- 后端提供安全变更账户父节点的 API：防成环、同级不重名、事务内维护闭包表一致性。
- 前端拖放改父节点，乐观更新，失败回滚并提示。
- 同级拖动排序，顺序持久化在 localStorage（按用户/浏览器本地）。
- 新建账户流程简化：选中账户 → 新建为其子账户；无选中时引导用户先选择。

**Non-Goals:**

- 不做跨设备/跨浏览器的顺序同步（无服务端排序字段）。
- 不支持多选批量移动。
- 不改变账户删除、关户、导入等既有流程的语义。
- 不为移动操作记录审计历史（移动前后视为不同账户，无追溯需求）。

## Decisions

### D1: 新增 `PUT /api/accounts/{id}/parent` 端点，而非复用通用 update

请求体 `{ "parent_id": "<AccountId>" }`（必填，不允许 null——根账户是固定类型锚点，不允许新建/移动出新的根）。语义单一、校验集中，避免与 rename/fields/owner 等端点互相干扰。

备选：扩展 `/fields` 通用更新端点——否决，移动涉及闭包表重建和成环校验，与普通字段更新复杂度完全不同。

### D2: 校验规则（service 层，按序执行）

1. 被移动账户存在、非 `is_system`、非根账户（`parent_id IS NULL` 的直接/间接等价判断：根账户无父，禁止移动）。
2. 目标父账户存在。
3. 目标父账户 ≠ 自身，且不在被移动账户的后代集合中（防成环）。利用闭包表：`SELECT 1 FROM account_ancestors WHERE account_id = :target AND ancestor_id = :moved` 命中即成环。
4. 目标父账户下无同名账户（复用 `account_get_by_parent_and_name`；名字按当前语言解析，与重名校验的既有行为一致）。
5. **不校验账户类型**——允许跨类型，类型由父链重新推导。

已关闭账户不特殊处理（允许移动）。

### D3: 闭包表重建在单事务内完成

执行顺序：

1. `UPDATE accounts SET parent_id = :new_parent, updated_at = ... WHERE id = :id`
2. 删除被移动子树（自身 + 所有后代）在 `account_ancestors` 中的全部行
3. 按新父链重建子树每个节点的闭包行（复用/泛化已有 `account_rebuild_ancestors`，`repo/account.rs:251`）

三步包在一个事务里，失败整体回滚。子树规模在个人账户系统中很小（几十到几百节点），全量重建子树闭包比增量 diff 简单且足够快。

备选：增量计算新旧祖先差集——否决，实现复杂、易错，子树小时无收益。

### D4: 前端拖放——手写 pointer 手势，不引入拖放库

项目无拖放库依赖，且已有手写手势先例（`PageSwitcher.vue`）。卡片网格 + 递归展开行的模型与列表排序库（sortablejs 等）的假设不匹配，引入库仍需大量适配。手写方案：

- 卡片上 `pointerdown` + 移动阈值（~6px）后进入拖动态，原卡片半透明，浮层跟随指针。
- 拖动中命中检测目标卡片：落在卡片上 → 高亮为"成为其子账户"的放置目标。
- 同级卡片之间的间隙命中 → 显示插入指示线，松手后仅更新前端同级顺序。
- 悬停在未展开的目标卡片上 ~600ms 自动展开其下一级（复用现有 `expandedPath` 机制）。
- 松手后：改父节点 → 乐观更新本地树 + 调 API，失败则回滚并 toast；同级排序 → 只更新本地顺序状态。

HTML5 DnD 备选——否决：移动端不支持，且项目有触屏需求（PageSwitcher 存在即证据）。

### D5: 同级顺序存 localStorage

键如 `account-sibling-order`，值为 `{ [parentId]: AccountId[] }` 的映射。渲染时在 `buildRowTree`/`compileRows` 前对每组同级账户按此映射排序（映射中缺失的账户按 id 顺序追加在尾部，映射中多余的 id 忽略）。父节点变更时无需清理——被移动账户自然从新父的映射缺失态按 id 顺序落位。

备选：纯内存——需求方接受 localStorage（刷新不丢）；服务端排序字段——否决，schema 内联 `CREATE TABLE IF NOT EXISTS` 无迁移框架，为本地偏好加列不值。

### D6: 跨类型移动确认

放置目标与被拖账户根类型不一致时，松手先弹确认（"移动后该账户及其子账户将变为 XX 类型"），确认后才发 API。前端已有根类型推导工具（`AccountCreateDrawer` 的 `findRootType` 逻辑可提取复用）。

### D7: 新建抽屉简化

- `AccountCreateDrawer` 移除父账户 `<select>` 与账户类型字段，改为只读提示"将作为「X」的子账户创建"。
- `AccountsView` 的新建按钮：无 `selectedAccountId` 时不禁用但点击后 toast 提示"请先选择一个账户"（保持可发现性）。
- 创建请求体直接带 `parent_id = selectedAccountId`，后端逻辑不变。

## Risks / Trade-offs

- [闭包表重建遗漏导致祖先链错乱] → 单事务 + 复用已验证的 `account_rebuild_ancestors`；为含多级子树的移动写集成测试，校验移动后每个后代节点的祖先链。
- [乐观更新与并发修改冲突（另一标签页改了树）] → 失败后回滚 + 重新拉取账户列表；单用户本地系统，冲突概率低。
- [拖动手势与点击选中/展开的手势冲突] → 移动阈值区分点击与拖动；拖动中屏蔽 click 冒泡。
- [网格布局中命中检测复杂（子行嵌套、展开动画）] → 拖动期间用 `getBoundingClientRect` 快照 + 指针坐标手动命中，不依赖 DOM 结构冒泡。
- [localStorage 顺序与账户增删脱节（残留 id）] → 渲染排序时忽略不存在的 id，缺失账户追加尾部，无需主动清理。
- [跨类型移动后报表/关户校验口径变化出乎用户意料] → D6 确认弹窗明确提示；系统无关联状态，余额重算即可。

## Open Questions

- 拖动经过父卡片自动展开的延迟（600ms）与动画细节，实现时微调。
- i18n 文案（toast、确认弹窗、抽屉提示）在实现时补入 locales。
