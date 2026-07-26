## 1. PanelAction 多按钮支持

- [x] 1.1 扩展 `panelAction.ts`：PanelAction 增加 `icon?: string` 字段，注入类型改为 `Ref<PanelAction[]>`
- [x] 1.2 改造 `ViewPanel.vue`：按数组顺序从右到左渲染按钮，icon 字段存在时显示图标替代 `+` 前缀
- [x] 1.3 适配现有 view（TransactionView、AccountsView、CalendarView、BudgetView、AssetsView）的 panelAction 注册为数组形式

## 2. Store 筛选状态与请求参数

- [x] 2.1 在 `types/api.ts` 定义 `TxFilters` 接口（from/to/accounts/members/tags/channels/keyword/reimbursable）
- [x] 2.2 新增 `buildTxQuery(filter, extra)` 工具函数：将 TxFilters 序列化为 URLSearchParams（多值用 append），编写单元测试
- [x] 2.3 `api/client.ts` 的 fetchTransactions 改为接受 URLSearchParams
- [x] 2.4 transaction store 增加 `activeFilter` 状态、`setFilter()`/`clearFilter()` action，setFilter 触发 resetList + loadInitial
- [x] 2.5 改造 `loadInitial`：合并 activeFilter 参数；筛选激活时 expandSameDay 翻倍上限 3 次（limit ≤ 800）
- [x] 2.6 改造 `loadMore`：携带 activeFilter 参数；loadedRange.from <= filter.from 时停止翻页
- [x] 2.7 增加 requestId 竞态保护：loadInitial 响应回来时比对 requestId，过期则丢弃

## 3. 筛选抽屉组件

- [x] 3.1 创建 `TransactionFilterDrawer.vue`：底部抽屉结构（absolute 定位、60% 高度、滑入动画、内容独立滚动）
- [x] 3.2 时间范围区域：快捷预设 chips（本月/上月/近三月/今年/全部）+ 自定义日期范围输入，预设匹配时高亮
- [x] 3.3 账户多选区域：从 accountStore 加载账户列表，chip 多选切换
- [x] 3.4 标签多选区域：从 tagStore 加载标签，chip 多选切换
- [x] 3.5 渠道多选区域：从 channelStore 加载渠道，chip 多选切换
- [x] 3.6 成员多选区域：从 memberStore 加载成员，chip 多选切换
- [x] 3.7 备注搜索输入框 + 可报销 toggle
- [x] 3.8 底部操作栏：重置（无条件时禁用）+ 完成按钮
- [x] 3.9 条件变化 → 300ms debounce → 调用 store.setFilter()

## 4. TransactionView 集成

- [x] 4.1 注册筛选 panel action（漏斗图标），activeFilter 非空时高亮样式，表单 overlay 打开时隐藏
- [x] 4.2 集成 drawer：showFilterDrawer 状态，与表单 overlay 互斥（打开表单时收起 drawer）
- [x] 4.3 Hero 区域：筛选激活时月份标签旁显示"已筛选"标记

## 5. 国际化

- [x] 5.1 `locales/zh-CN.ts` 和 `locales/en.ts` 添加筛选相关文案（按钮、预设、区域标题、操作按钮、已筛选标记）

## 6. 验证

- [x] 6.1 运行现有单元测试确认无回归（buildTxQuery 新测试 + 既有 sunburst/cashFlowList 测试）
- [x] 6.2 启动 dev server 手动验证：抽屉展开/收起动画、各条件即时刷新、翻页携带参数、from 终止翻页、面板切换状态保持、表单互斥
