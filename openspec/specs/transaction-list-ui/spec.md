# transaction-list-ui

## Purpose

交易列表前端界面——按月加载交易数据，按日分组展示，包含月收支汇总、交易卡片（收支账户、成员、备注、金额、资产账户）、标签展示和金额计算规则。

## Requirements

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

### Requirement: 交易按日分组展示
交易列表 SHALL 按日期分组展示，每组显示日期、星期和当日收支汇总。

#### Scenario: 有交易的日期
- **WHEN** 某日有交易记录
- **THEN** 显示日期分组头，格式为 "MM.DD 星期X"，右侧显示 "收:¥X 支:¥Y"

#### Scenario: 无交易的日期
- **WHEN** 某日没有交易记录
- **THEN** 不显示该日期的分组头

### Requirement: 交易卡片展示
每笔交易 SHALL 以卡片形式展示，包含收支账户、成员、备注、金额、资产账户。

#### Scenario: 普通交易（有收支账户）
- **WHEN** 交易包含收支类分录
- **THEN** 卡片第一行显示收支账户名（左）和金额（右），第二行显示成员和备注（左）和资产账户（右）

#### Scenario: 转账交易（无收支账户）
- **WHEN** 交易只有资产类分录
- **THEN** 卡片左上角显示"转账"标签，金额为正值之和

#### Scenario: 退款交易
- **WHEN** 交易类型为 refund
- **THEN** 标题文字显示为灰色（muted），金额显示为绿色

#### Scenario: 交易有标签
- **WHEN** 交易有关联标签
- **THEN** 标签显示为红色小圆角矩形，位于备注下方

### Requirement: 月收支汇总展示
交易列表顶部 SHALL 显示月支出（大字号）、月收入和本月结余（小字号）。

#### Scenario: 有收支的月份
- **WHEN** 当月有交易记录
- **THEN** 显示月支出金额（大字号）、月收入和本月结余

#### Scenario: 无收支的月份
- **WHEN** 当月没有交易记录
- **THEN** 显示月支出 ¥0.00、月收入 ¥0.00、本月结余 ¥0.00

### Requirement: 金额计算规则
交易卡片金额 SHALL 按以下规则计算：
- 普通交易：资产账户分录金额之和
- 转账交易：资产账户分录正值之和

#### Scenario: 支出交易
- **WHEN** 交易包含支出分录和资产减少分录
- **THEN** 金额 = 资产分录金额之和（负数），显示为红色

#### Scenario: 收入交易
- **WHEN** 交易包含收入分录和资产增加分录
- **THEN** 金额 = 资产分录金额之和（正数），显示为绿色

#### Scenario: 转账交易
- **WHEN** 交易只有资产类分录，金额之和为 0
- **THEN** 金额 = 资产分录正值之和，显示为白色（无正负号）

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
