## Why

交易列表目前只能按时间倒序浏览，无法按账户、标签、渠道、成员或备注关键词查找交易。后端 `GET /transactions` 已支持全部筛选参数，前端从未接入。随着交易量增长，用户定位特定交易的成本越来越高。

## What Changes

- 交易页 ViewPanel header 新增筛选按钮（位于新建交易按钮左侧），有激活筛选条件时按钮高亮
- 新增半屏底部抽屉（FilterDrawer），展示所有筛选条件：时间范围（快捷预设 + 自定义）、账户多选、标签多选、渠道多选、成员多选、备注模糊搜索、可报销开关
- 筛选条件变化后即时（debounce 300ms）刷新交易列表，抽屉保持展开，用户可边调边看
- 筛选状态存储在 Pinia store 中，翻 panel 不丢失，页面刷新后重置
- `loadInitial` / `loadMore` 携带当前筛选参数请求后端；有 `from` 日期时翻页到 `from` 即止
- 顶部月度汇总（Hero）跟随筛选结果变化
- 表单覆盖层打开时抽屉自动收起

## Capabilities

### New Capabilities
- `transaction-filter`: 交易筛选抽屉 UI、筛选状态管理与即时刷新行为

### Modified Capabilities
- `transaction-list-ui`: 新增筛选按钮入口（header 双按钮）、筛选激活态高亮、Hero 跟随筛选结果
- `lazy-load-transactions`: loadInitial/loadMore 携带筛选参数；同一天膨胀逻辑加上限；有 from 时翻页提前终止

## Impact

- **前端**: `accounting-web` — stores/transaction.ts（filter state + 请求参数序列化）、TransactionView.vue（按钮注册 + drawer 集成）、ViewPanel.vue / panelAction.ts（多按钮 + icon 支持）、新增 TransactionFilterDrawer.vue 组件、locales（中英文文案）
- **后端**: 无变更（API 已就绪）
- **API 客户端**: fetchTransactions 需支持多值参数（URLSearchParams 重复 key）
