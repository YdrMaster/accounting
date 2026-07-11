## Why

前端 5 个页面中有 3 个（资产、日历、预算）使用硬编码数据，交易页只读无增删改，账户页缺少新建功能。后端 API 已基本完备（budget-api 已在上个 change 完成），需要将前端全面接入真实数据，使应用具备完整的记账能力。

## What Changes

- 交易页改为连续滚动的交易流，支持上下滚动自动加载历史/新增月份数据
- 新增交易表单覆盖层，支持创建、编辑、删除交易（完整复式记账，仅 normal 模式）
- 交易卡片增加增删改交互（双击/左滑编辑、右滑删除）
- 日历页增加月历控件，点选日期筛选交易，与交易页共享交易列表组件
- 资产页接入 balance-sheet API，只读展示总资产/总负债/净资产
- 账户页增加新建账户功能
- 预算页接入 budget API，支持预算表 CRUD 和执行情况查看
- 新增账户选择覆盖层组件，在交易表单中选择账户时复用现有账户网格

## Capabilities

### New Capabilities
- `transaction-form`: 交易表单覆盖层——完整复式记账的创建/编辑表单，包含分录管理、自动配平、账户选择覆盖层、标签输入、渠道链路输入
- `calendar-view`: 日历页面——月历网格控件 + 按日筛选的交易列表，与交易页共享交易列表和交易卡片组件
- `assets-view`: 资产页面——接入 balance-sheet API，展示总资产/总负债/净资产及各账户余额明细
- `budget-view`: 预算页面——接入 budget API，展示预算列表、执行情况，支持预算表 CRUD 抽屉

### Modified Capabilities
- `transaction-list-ui`: 从按月分页改为连续滚动加载；抽取为共享组件供交易页和日历页复用
- `transaction-entry-display`: 交易卡片增加增删改交互入口（双击/左滑编辑、右滑删除）
- `account-page`: 增加新建账户按钮和创建流程

## Impact

- **前端组件**: 新增 TransactionList、TransactionCard、TransactionFormOverlay、AccountPickerOverlay、CalendarGrid、ChannelPathInput 等共享组件；改造 5 个 View
- **API 客户端**: `api/client.ts` 需补充 transaction CRUD、balance-sheet、budget CRUD、commodities、channels 等函数
- **Pinia stores**: 新增 budget store；改造 transaction store 支持分页加载和 CRUD actions；补充 report store 的 balance-sheet
- **类型定义**: `types/api.ts` 需补充 BalanceSheetDto、BudgetDto、BudgetDetailDto、BudgetStatusDto 等
- **后端依赖**: 无新增后端需求，所有 API 端点已就绪
- **无 breaking changes**: 现有功能保持不变，仅扩展
