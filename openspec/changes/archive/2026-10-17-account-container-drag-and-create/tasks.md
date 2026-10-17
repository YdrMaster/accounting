# Tasks: account-container-drag-and-create

## 1. 拖拽修复：容器化命中测试与 drop 分流

- [x] 1.1 `AccountGrid.vue` 接收根账户 prop 并在分区网格标注 `data-container-account-id`；`AccountRowGroup.vue` 在 `children-container` 标注 owner id（design D2）
- [x] 1.2 `AccountsView.vue` 分区从按类型平铺改为按根账户逐个渲染 `AccountGrid`，每个根一个容纳框（design D3）
- [x] 1.3 空容纳框渲染最小高度占位区（虚线样式），可命中、可选中（design D4）
- [x] 1.4 `useAccountDrag.updateHitTest` 重构为三级命中：卡片 → 最内层包含容器（面积最小）→ 容器内最近插槽（行内间隙/左右端/行间归并；空容器整框落点）；删除"仅自己兄弟行"限制（design D1）
- [x] 1.5 命中校验：容器 owner 为被拖账户自身后代时拒绝落点，容器选中/指示线状态对齐现有 `dropTargetId`/`insertion` 渲染（design D1）
- [x] 1.6 `AccountsView.handleDrop` 的 `insert` 分支按同父/跨父分流：同父走 `handleSiblingReorder`；跨父走 `handleMoveAccount`（含跨类型确认、回滚），成功后按插入位置写入目标父级 `siblingOrder`（design D5、D6）
- [x] 1.7 drop-on-card 与末位插入后将 moved id 追加到目标父级 `siblingOrder` 末尾，保证视觉末位（design D5）
- [x] 1.8 `AccountGrid.vue` 列数断点 600px→450px、900px→675px（design D8）
- [x] 1.9 为 `useAccountDrag` 命中测试补充单测：卡片命中、跨层间隙、空容器、自身后代拒绝、末位插槽

## 2. 容纳框选中与新建统一

- [x] 2.1 容纳框空白处点击选中 owner（排除 `event.target.closest('.account-card')` 的卡片事件），不弹详情抽屉；选中容器渲染可见标识（design D7）
- [x] 2.2 顶栏"新建账户"按钮以选中容器 owner 为父打开 `AccountCreateDrawer`，创建结果渲染在该容器末位；无选中仍禁用（design D7）
- [x] 2.3 更新 `AccountsViewRefresh.spec.ts`、`AccountGrid.spec.ts` 等既有前端测试以覆盖分区按根拆分与容器选中行为

## 3. 移动失败报错修复（后端）

- [x] 3.1 `set_parent` 返回类型改为 `Result<Json<AccountDto>, (StatusCode, String)>`：`AccountAlreadyExists` → 409（本地化文案含冲突账户名）、`AccountNotFound`/`InvalidTransaction` → 400、`DatabaseError` → 500（design D9）
- [x] 3.2 通过 router（axum test client）补 HTTP 层测试：重名失败返回 409 + 可读文案，校验失败状态码均非 2xx
- [x] 3.3 手动验证拖动到同名子账户存在的目标父级时，页面弹窗显示明确重名提示而非 JSON 解析错误

## 4. 回归验证

- [x] 4.1 前端测试套件与类型检查全绿（`npm run test` / `vue-tsc`）
- [x] 4.2 浏览器手测主路径：向空的"收入"根拖入首个一级账户、卡片间空隙跨层插入、同级重排、点根容纳框新建一级账户、拖到卡片追加为末位子账户、跨类型移动确认弹窗
