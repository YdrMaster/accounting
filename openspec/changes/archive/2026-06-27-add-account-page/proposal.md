## Why

当前主界面缺少账户管理入口，用户无法在 Web 端查看和编辑账户信息（名称、账单日、还款日、所有者、关闭/重开、删除）。同时，页面导航方式单一——固定的"记账"标题和左右箭头在页面增多后不够直观，需要更灵活的页面切换机制。

## What Changes

- 新增"账户"页面，展示所有账户的卡片网格，按根账户类型分栏（资产、收入、支出、权益、导入），支持选中账户、展开子账户、从下方拉起抽屉进行 inline 编辑
- 将主界面顶部固定的"记账"标题替换为可拖动的页面切换条（圆角矩形标签横排），高亮背景色框住当前屏幕可见的几个标签
- 页面顺序调整为：交易 → 资产 → 账户 → 日历 → 预算
- 桌面端：保留左右箭头按钮（与切换条同一横排），内容区不可拖拽切换页面
- 移动端：内容区可拖拽切换页面，切换条联动高亮，移除底部圆点指示器
- 后端 `AccountType` 枚举新增 `Import` 变体，支持"导入"类型的根账户

## Capabilities

### New Capabilities

- `page-switcher`: 页面切换条组件，替代原有固定标题 + 箭头导航，支持拖动/点击切换、高亮可见标签、桌面与移动端差异化交互
- `account-page`: 账户管理页面，包含卡片网格展示、选中/展开互斥逻辑、抽屉式 inline 编辑面板
- `account-type-import`: 后端 AccountType 新增 Import 变体，支持导入类型根账户

### Modified Capabilities

- `transaction-list-ui`: 页面顺序调整，在 paneNames 中插入 accounts 并调整顺序

## Impact

- **前端**: `ResponsiveShell.vue` 重构导航区域，新增 `PageSwitcher.vue`、`AccountsView.vue`、`stores/account.ts`，修改 `useResponsiveLayout.ts`
- **后端**: `accounting/src/account_type.rs` 新增 Import 变体，`accounting-api` 相关 handler 和 DTO 可能需要适配
- **API**: 前端需新增账户列表获取和各项编辑操作的 API 调用封装（后端 API 已存在）
