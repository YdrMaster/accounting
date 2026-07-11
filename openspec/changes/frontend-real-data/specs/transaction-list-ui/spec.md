## MODIFIED Requirements

### Requirement: 交易列表按月加载
系统 SHALL 支持连续滚动加载交易列表，默认加载当前月份，滚动到边界时自动加载相邻月份数据。页面顺序中交易页面位于第一位。

#### Scenario: 加载当月交易
- **WHEN** 用户打开交易列表
- **THEN** 系统加载当前月份的交易数据，交易页面为默认显示页面

#### Scenario: 向下滚动加载更多
- **WHEN** 用户向下滚动到列表底部
- **THEN** 系统自动加载上一个月的交易数据，追加到列表末尾

#### Scenario: 向上滚动加载更新
- **WHEN** 用户向上滚动到列表顶部
- **THEN** 系统自动加载下一个月的交易数据，前置到列表开头

#### Scenario: 无更多数据
- **WHEN** 已加载所有历史月份或已到最新月份
- **THEN** 不再触发加载请求

## ADDED Requirements

### Requirement: 交易列表抽取为共享组件
TransactionList 和 TransactionCard SHALL 抽取为独立共享组件，供 TransactionView 和 CalendarView 共同使用。

#### Scenario: TransactionView 使用共享组件
- **WHEN** TransactionView 渲染交易列表
- **THEN** 使用 TransactionList 组件，传入连续滚动的数据源

#### Scenario: CalendarView 使用共享组件
- **WHEN** CalendarView 渲染交易列表
- **THEN** 使用同一个 TransactionList 组件，传入按日筛选的数据源

### Requirement: 交易列表新建交易入口
TransactionList 所在页面 SHALL 提供新建交易按钮，点击后打开交易表单覆盖层。

#### Scenario: 交易页新建按钮
- **WHEN** 用户在交易页
- **THEN** 页面提供新建交易按钮

#### Scenario: 日历页新建按钮
- **WHEN** 用户在日历页
- **THEN** 页面提供新建交易按钮
