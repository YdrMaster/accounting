# Tasks

## 1. 后端：repo 层（accounting-sql）

- [x] 1.1 在 `repo/account.rs` 新增 `account_update_parent`：单事务内更新 `accounts.parent_id`（含 `updated_at`），删除被移动子树（自身+后代）在 `account_ancestors` 的全部行，按新父链重建子树闭包行（复用/泛化 `account_rebuild_ancestors`）
- [x] 1.2 新增/复用查询：判断目标是否在被移动账户后代中（闭包表单条查询），用于防成环
- [x] 1.3 写 repo 层测试：移动含多级子树的账户后，校验每个后代节点的祖先链正确；移动失败时事务回滚

## 2. 后端：service + API（accounting-service / accounting-api）

- [x] 2.1 service 新增移动校验方法，按序校验：账户存在、非根、非 `is_system`；目标父存在；目标非自身/非后代；目标父下无同名（复用 `account_get_by_parent_and_name`）；不校验类型
- [x] 2.2 新增路由 `PUT /api/accounts/{id}/parent`，请求体 `{ parent_id }`，返回更新后的账户；校验失败返回明确错误信息
- [x] 2.3 写 API 层测试：移动成功、根账户拒绝、系统账户拒绝、目标不存在拒绝、成环拒绝、同级重名拒绝、跨类型移动成功、移动已关闭账户成功

## 3. 前端：API 与新建流程（accounting-web）

- [x] 3.1 `api/client.ts` 新增 `moveAccount(id, parentId)` 封装 `PUT /api/accounts/{id}/parent`，`types/api.ts` 补类型
- [x] 3.2 `AccountCreateDrawer.vue` 移除父账户 `<select>` 和账户类型字段，改为只读提示「将作为 X 的子账户创建」；创建请求带 `parent_id = selectedAccountId`
- [x] 3.3 `AccountsView.vue` 新建按钮：无 `selectedAccountId` 时点击 toast 提示「请先选择一个账户」
- [x] 3.4 补 i18n 文案（locales）：抽屉提示、toast、确认弹窗

## 4. 前端：拖放重定位

- [x] 4.1 实现拖动手势（手写 pointer 事件，参考 `PageSwitcher.vue`）：移动阈值 ~6px 进入拖动态，原卡片半透明，浮层跟随指针，拖动中屏蔽 click 冒泡；系统账户卡片不响应拖动
- [x] 4.2 命中检测：落在目标卡片上 → 高亮为"成为其子账户"；落在同级卡片间隙 → 显示插入指示线；悬停未展开父卡片 ~600ms 自动展开（复用 `expandedPath`）
- [x] 4.3 松手执行改父节点：跨类型先弹确认（提取复用根类型推导逻辑）；乐观更新本地树 + 调 `moveAccount`，失败回滚并 toast
- [x] 4.4 同级排序：松手在间隙时只更新本地顺序；顺序以 `{ [parentId]: AccountId[] }` 存 localStorage；渲染排序在 `buildRowTree`/`compileRows` 前应用（缺失 id 按创建顺序追加尾部，多余 id 忽略）

## 5. 验证

- [x] 5.1 `cargo test` 全 workspace 通过
- [x] 5.2 前端构建通过（`cd accounting-web && npm run build`）
- [ ] 5.3 手动验证：拖到卡片上改父节点（含跨类型确认）、同级排序刷新后保持、未选中点新建的提示、新建子账户流程
