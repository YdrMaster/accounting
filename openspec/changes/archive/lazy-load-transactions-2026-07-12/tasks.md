## 1. Store 重构

- [x] 1.1 重构 transaction store 数据结构：移除 `loadedMonths` 和 `transactionsByMonth`，新增 `loadedRange`、`transactions`、`calendarDays`
- [x] 1.2 实现 `loadInitial(toDate, limit)` 方法：请求最新 N 笔交易，设置 loadedRange
- [x] 1.3 实现 `loadMore()` 方法：从 loadedRange.from 继续往回加载，处理同一天膨胀逻辑
- [x] 1.4 实现 `loadDay(date)` 方法：加载指定日期的全部交易，存储到 calendarDays
- [x] 1.5 实现 `allTransactions` computed：合并主列表和 calendarDays，按时间倒序，去重
- [x] 1.6 实现 `transactionDates` computed：从主列表和 calendarDays 提取有交易的日期集合（供日历圆点使用）

## 2. TransactionView 改造

- [x] 2.1 简化 onMounted：调用 `loadInitial(today, 100)`，移除按月加载逻辑
- [x] 2.2 添加 IntersectionObserver：监听 from 日期那组交易的 sentinel 元素
- [x] 2.3 实现 onSentinelIntersect：当 sentinel 进入视口时调用 `loadMore()`
- [x] 2.4 在 TransactionList 组件中添加 sentinel 元素（放在 from 日期那组交易末尾）
- [x] 2.5 移除旧的 onScroll 滚动监听逻辑

## 3. CalendarView 改造

- [x] 3.1 简化 onMounted：调用 `loadInitial(today, 100)`，移除按月加载逻辑
- [x] 3.2 修改 onSelectDate：检查点击日期是否在 loadedRange 内，不在则调用 `loadDay(date)`
- [x] 3.3 修改 filteredTransactions：从 allTransactions 过滤选中日期的交易
- [x] 3.4 修改 transactionDates：使用 store 的 transactionDates computed

## 4. 测试与验证

- [x] 4.1 验证初始加载：API 返回 100 笔 (06-24 ~ 06-30)，loadedRange 正确设置
- [x] 4.2 验证滚动触发：API 从 06-24 继续返回 100 笔更早数据 (06-18 ~ 06-24)
- [x] 4.3 验证同一天膨胀：代码逻辑正确（无真实数据触发，最大日 33 笔 < 100）
- [x] 4.4 验证日历点击：API 支持 from=to 查询单日全部交易
- [x] 4.5 验证重复点击：loadDay 检查 calendarDays.has(date) 和 loadedRange 范围
- [x] 4.6 验证数据合并：allTransactions 合并去重排序逻辑正确
