## Why

当前前端交易列表和日历页面只加载一个月的数据（默认 100 笔 limit），导致用户只能看到最近 1-2 周的交易记录。数据库中有 4-6 月共 1082 笔交易，但前端无法展示完整数据。需要实现触发式懒加载，让用户能够浏览所有历史交易。

## What Changes

- **初始加载策略**：从当日往回加载最新 100 笔交易，记录已加载的时间范围 `[from, to]`
- **滚动触发加载**：当用户滚动到已加载范围的最老日期时，自动触发加载更早的 100 笔交易
- **同一天数据膨胀**：如果 100 笔交易都在同一天，自动将 limit 翻倍（200→400→800...）直到跨天
- **日历按需加载**：点击日历某天时，如果该天不在已加载范围内，单独加载该天全部交易
- **数据结构重构**：从按月存储改为按时间范围存储，支持连续的时间范围追踪

## Capabilities

### New Capabilities

- `lazy-load-transactions`: 触发式懒加载机制，包括初始加载、滚动触发、同一天膨胀、日历按需加载

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

**前端代码**：
- `stores/transaction.ts` - 重构数据结构（`loadedRange`、`transactions`、`calendarDays`）
- `views/TransactionView.vue` - 简化初始化逻辑，用 IntersectionObserver 替代滚动监听
- `views/CalendarView.vue` - 点击日期时触发按需加载
- `components/TransactionList.vue` - 添加 sentinel 元素用于触发加载

**后端 API**：
- 无需修改 - 现有 `/api/transactions` 已支持 `from`、`to`、`limit` 参数组合

**用户体验**：
- 初始加载更快（只拉 100 笔而非整月）
- 滚动时自动加载更早数据，无需手动翻页
- 日历可自由翻月，点击任意日期即时加载
