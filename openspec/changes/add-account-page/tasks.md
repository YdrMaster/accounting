## 1. 后端 AccountType 扩展

- [ ] 1.1 在 `accounting/src/account_type.rs` 中为 `AccountType` 枚举添加 `Import` 变体
- [ ] 1.2 更新 `FromStr` 实现，支持 "import" / "导入" 解析为 `Import`
- [ ] 1.3 更新 `display_name` 方法，添加 `account_type_import` 国际化 key
- [ ] 1.4 在 `accounting-api/src/dto.rs` 的 `AccountDto` 中确认 `account_type` 字段能正确序列化 `Import`
- [ ] 1.5 添加国际化文本（locales/zh-CN.yaml 和 locales/en.yaml 中的 `account_type_import`）

## 2. 前端类型和 API 层

- [ ] 2.1 在 `types/api.ts` 中添加 `AccountDto` 接口定义（id, name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day, owner_ids）
- [ ] 2.2 在 `api/client.ts` 中添加账户相关 API 调用函数（fetchAccounts, renameAccount, setAccountOwners, closeAccount, reopenAccount, deleteAccount）
- [ ] 2.3 创建 `stores/account.ts`，实现账户数据 store（加载账户列表、构建树形结构、提供按类型分组的方法）

## 3. 页面切换条组件

- [ ] 3.1 创建 `PageSwitcher.vue` 组件，接收页面列表、当前索引、可见数量等 props
- [ ] 3.2 实现切换条的圆角矩形标签渲染和高亮背景色框
- [ ] 3.3 实现切换条的拖动切换逻辑（touch/mouse 事件）
- [ ] 3.4 实现切换条的点击标签切换逻辑
- [ ] 3.5 实现桌面端左右箭头按钮（与切换条同一横排）
- [ ] 3.6 实现移动端箭头隐藏逻辑

## 4. ResponsiveShell 重构

- [ ] 4.1 修改 `useResponsiveLayout.ts`，在 `paneNames` 中添加 `accounts`，调整顺序为 transaction → assets → accounts → calendar → budget
- [ ] 4.2 在 `paneLabels` 中添加 `accounts: '账户'`
- [ ] 4.3 修改 `ResponsiveShell.vue`，用 `PageSwitcher` 替换 `WideHeader`
- [ ] 4.4 在 `componentMap` 中添加 `accounts: AccountsView`
- [ ] 4.5 移除底部圆点指示器相关代码
- [ ] 4.6 实现桌面端禁用内容区拖拽、移动端保留内容区拖拽的差异化逻辑
- [ ] 4.7 实现移动端内容区拖拽时切换条高亮框联动

## 5. 账户页面组件

- [ ] 5.1 创建 `AccountsView.vue`，实现按根账户类型分 5 栏竖排布局
- [ ] 5.2 实现账户卡片网格渲染（只显示名称）
- [ ] 5.3 实现根账户栏目标题展示（不可点击）
- [ ] 5.4 实现点击账户卡片的选中逻辑
- [ ] 5.5 实现子账户展开行渲染（高亮行显示在父账户下方）
- [ ] 5.6 实现选中/展开互斥逻辑（子树包含判断）
- [ ] 5.7 创建抽屉组件，从页面底部滑入
- [ ] 5.8 在抽屉中实现账户详情展示（名称、账单日、还款日、所有者、关闭状态）
- [ ] 5.9 在抽屉中实现 inline 编辑控件（名称编辑、账单日/还款日编辑、所有者选择、关闭/重开、删除）
- [ ] 5.10 实现根账户不可编辑的限制

## 6. 集成和验证

- [ ] 6.1 确保页面切换条与账户页面的交互正常（切换页面时抽屉关闭、选中状态清除）
- [ ] 6.2 验证桌面端和移动端的差异化交互
- [ ] 6.3 验证账户编辑操作后数据刷新
- [ ] 6.4 运行前端构建确认无编译错误
