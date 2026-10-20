# Proposal: fix-transaction-load-expansion

## Why

交易列表现在用「取 K 笔，结果全落在同一天时就把 `limit` 翻倍直到跨天」来保证按天分组的边界正确。这套做法把“按条数分页”和“按天分组”两种粒度硬凑在一起：一旦结果永远只有一天（整个账本的交易都在同一天，或已经翻到最早一天），`limit` 就会无限翻倍，最终超出 `i64` 被后端拒绝。而这条错误又以 HTTP 200 + 纯文本返回，前端对 2xx 一律当作 JSON 解析，于是抛出 `Unexpected token '无', "无效的 limit "... is not valid JSON`，显示在月度统计与交易列表之间，但交易其实已经保存成功。

正确的做法不是在行端猜边界、补边界，而是让后端一次把“整天”给对：加一个参数，让每一页天然由完整的若干天组成；前端随之删掉翻倍逻辑。

## What Changes

- 后端 `GET /api/transactions` 增加布尔查询参数 `full_days`。为 `true` 时按“整天”分页：先取最新 `limit` 笔，定位其中最早一笔所在的那一天 D，返回 `[D 当天, to]` 范围内的全部匹配交易，因此返回条数不少于 `limit`，且页尾一定落在某一天的结尾。
- 前端据此重写加载：初始加载 `to = 筛选的 to ?? 今天`，滚动翻页 `to = 当前最旧一天的前一天`，日历加载 `from=D&to=D`，三者都走整日模式；删除 `expandSameDay`、翻倍与翻倍上限等全部逻辑。
- 不筛选、日历、筛选统一为同一套（不筛选 = `to` 默认今天，日历 = 一天范围）。
- 交易读接口失败返回非 2xx（参数非法 → 400，数据库/内部故障 → 5xx），让前端走错误分支展示服务端本地化文案，而不是把错误明文当 JSON 解析。
- 同步 `lazy-load-transactions` 与 `transaction-api` 规格。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `lazy-load-transactions`：加载改为整日模式的按天翻页；删除「同一天数据膨胀」。
- `transaction-api`：新增 `full_days` 参数（整日分页）；交易读接口失败返回非 2xx。

## Impact

- 后端：`accounting-api/src/handlers/transaction.rs`（`list_transactions` 解析并使用 `full_days`）、`accounting-service` / `accounting-sql` 的交易列表查询（支持“取整天”）。
- 前端：`accounting-web/src/stores/transaction.ts`（`loadInitial` / `loadMore` / `loadDay` 统一为整日翻页，删除 `expandSameDay`）、`accounting-web/src/api/client.ts`（`fetchTransactions` 传 `full_days`）。
- 规格：`openspec/specs/lazy-load-transactions`、`openspec/specs/transaction-api`。
- 协调：`accounting-api/src/handlers/transaction.rs` 同时被在途变更 `user-facing-text-i18n` 修改（错误文案 / `Lang`），实现时需排期。
- 测试：后端整日分页查询测试、前端 store 翻页测试。
