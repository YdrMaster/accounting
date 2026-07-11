## 1. API 层与类型定义

- [x] 1.1 在 types/api.ts 中补充 BalanceSheetDto、AccountBalanceItemDto、BudgetDto、BudgetDetailDto、BudgetStatusDto、BudgetItemStatusDto、BudgetLimitDto、CreateTransactionData、PostingInput、ChannelPathNodeInput 等类型
- [x] 1.2 在 api/client.ts 中补充交易 CRUD 函数：fetchTransactions(params)、createTransaction(data)、updateTransaction(id, data)、deleteTransaction(id)、fetchPosting(id)
- [x] 1.3 在 api/client.ts 中补充报表函数：fetchBalanceSheet()
- [x] 1.4 在 api/client.ts 中补充预算 CRUD 函数：fetchBudgets()、fetchBudgetDetail(id)、fetchBudgetStatus(id, date?)、createBudget(data)、updateBudget(id, data)、deleteBudget(id)
- [x] 1.5 在 api/client.ts 中补充辅助数据函数：fetchCommodities()、fetchChannels()、fetchTags()、createAccount(data)

## 2. Pinia Stores

- [x] 2.1 改造 transaction store：支持分页加载（按月）、缓存已加载月份、CRUD actions（create/update/delete）、清除缓存
- [x] 2.2 新增 budget store：预算列表、详情、执行状态的 state 和 actions
- [x] 2.3 补充 report store：添加 fetchBalanceSheet action 和 balanceSheet state
- [x] 2.4 新增 commodity store / channel store / tag store（或在现有 store 中补充），为交易表单提供可选数据

## 3. 共享交易组件

- [x] 3.1 从 TransactionView 中提取 TransactionCard.vue 组件：包含交易摘要、金额计算、标签、展开分录、双击/左滑编辑、右滑删除交互
- [x] 3.2 提取 TransactionList.vue 组件：接收交易数组 prop，按日分组渲染 TransactionCard，支持空状态展示
- [x] 3.3 确保 TransactionCard 的金额计算逻辑（computeAmount、isTransfer、isPureImport 等）在组件内正确封装

## 4. 账户选择覆盖层

- [x] 4.1 创建 AccountPickerOverlay.vue：覆盖整个 pane，复用 AccountGrid 组件展示账户网格，点击选中高亮但不弹编辑抽屉，底部确认按钮关闭覆盖层
- [x] 4.2 创建 AccountPicker 触发器组件：按钮显示已选账户名，点击打开 AccountPickerOverlay

## 5. 交易表单覆盖层

- [x] 5.1 创建 TransactionFormOverlay.vue 骨架：覆盖 pane 的表单面板，支持新建/编辑两种模式，包含日期时间、备注、成员选择字段
- [x] 5.2 实现分录列表区域：PostingRow 组件（账户选择触发器 + 币种下拉 + 金额输入 + 报销标记 + 删除按钮），添加分录按钮
- [x] 5.3 实现自动配平逻辑：addPosting 时计算金额 = -(已有分录金额之和)
- [x] 5.4 实现借贷平衡校验：分录金额之和不为 0 时禁用确认按钮
- [x] 5.5 实现标签输入组件 TagInput：搜索已有标签 + 输入新标签名称
- [x] 5.6 实现渠道链路输入组件 ChannelPathInput：分级槽位，前几级单选，最后一级多选
- [x] 5.7 实现表单提交逻辑：新建调用 POST /api/transactions，编辑调用 PUT /api/transactions/:id，成功后关闭覆盖层并刷新列表
- [x] 5.8 实现编辑模式数据加载：从 GET /api/transactions/:id 加载数据并预填充表单

## 6. 交易页改造

- [x] 6.1 改造 TransactionView.vue：移除内联交易渲染，使用 TransactionList 共享组件
- [x] 6.2 实现连续滚动：滚动到底部自动加载上一月，滚动到顶部自动加载下一月
- [x] 6.3 添加新建交易按钮，打开 TransactionFormOverlay
- [x] 6.4 接入交易卡片的双击/左滑编辑、右滑删除事件，连接 TransactionFormOverlay 和 deleteTransaction

## 7. 日历页

- [x] 7.1 创建 CalendarGrid.vue 组件：月历网格，7 列（周一至周日），左右翻月，有交易日期视觉标记
- [x] 7.2 改造 CalendarView.vue：上方 CalendarGrid + 下方 TransactionList，点击日期筛选交易
- [x] 7.3 日历页接入交易数据：按月加载交易，根据选中日期过滤展示
- [x] 7.4 日历页添加新建交易按钮和编辑/删除交互，复用 TransactionFormOverlay

## 8. 资产页

- [x] 8.1 改造 AssetsView.vue：移除硬编码数据，调用 fetchBalanceSheet() 获取真实数据
- [x] 8.2 实现总资产/总负债/净资产计算逻辑
- [x] 8.3 渲染账户余额明细列表（账户名 + 各币种余额）

## 9. 账户新建

- [x] 9.1 在 AccountsView 添加新建账户按钮
- [x] 9.2 实现账户创建抽屉（或复用 AccountDrawer 的 create 模式）：名称、账户类型、父账户字段
- [x] 9.3 接入 POST /api/accounts，创建成功后刷新账户列表

## 10. 预算页

- [x] 10.1 改造 BudgetView.vue：移除硬编码数据，调用 fetchBudgets() 获取预算列表
- [x] 10.2 实现预算执行情况展示：选择预算表后调用 fetchBudgetStatus()，展示各账户限额/实际/剩余
- [x] 10.3 实现预算创建/编辑抽屉：名称、周期类型、币种、限额列表
- [x] 10.4 实现预算删除功能：确认对话框 + DELETE /api/budgets/:id
