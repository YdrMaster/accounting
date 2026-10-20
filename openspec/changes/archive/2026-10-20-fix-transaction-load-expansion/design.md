# Design: fix-transaction-load-expansion

## Context

交易列表当前用 `to` + `limit` 做计数分页，再用 `expandSameDay` 把 `limit` 翻倍直到跨天，以补齐被切开的“天”。这套补齐在单日数据下不终止（见 proposal.md「Why」）。列表按天分组、每天头上有收/支小计，所以每一页必须落在完整的“天”边界上；这个约束改由后端一次满足。

## Goals / Non-Goals

**Goals:**

- 每一页都天然对齐到整天，且在任何数据形态下有界。
- 前端加载算法最简：一个游标（上一页最旧天的前一天），三个入口用同一种请求形态。
- 交易读接口失败返回非 2xx，错误不再被误当 JSON 解析。

**Non-Goals:**

- 不改筛选参数与语义。
- 不引入游标键（cursor / before）。
- 不改列表 UI 与合并去重逻辑。
- 不为“单日超大”设计天内二次分页（见 Risks）。

## Decisions

### D1: 整日分页由后端 `full_days` 参数实现

**决策**：`GET /api/transactions` 增加布尔参数 `full_days`，缺省 `false`（保持现状：最多 `limit` 笔）。为 `true` 时：

1. 先取出满足筛选、`date <= to` 的最新 `limit` 笔，定位其中最早一笔所在的那一天 D；
2. 返回满足筛选、且 `date` 落在 `[start_of_day(D), end_of_day(to)]` 内的全部交易，排序仍为 `date_time DESC, id DESC`。

返回条数不少于 `limit`（匹配总数不足 `limit` 时返回全部），页尾对齐到整天。下界用时间戳 `start_of_day(D)` 而非对列套 `date()`，以命中 `date_time` 索引；筛选条件在第 1、2 步必须完全一致（共用一个 WHERE 构造）。

**理由**：把“整天”的语义放进 SQL 一次算清，前端不再需要翻倍补天，同时保留计数分页“一次跳过空档”的好处。

**被否决备选：**

- 前端两次请求补边界（先取 K 笔，再 `from=D&to=D` 补整天）：多一次往返，逻辑分散在两处。
- 保留 `expandSameDay`，只补终止条件：边界仍是“猜”出来的，且重复下载。

### D2: 前端改成整天游标翻页

**决策**：`loadedRange.from` / `to` 两端都表示完整的天。

- `loadInitial(toDate?)`：`to = toDate ?? activeFilter.to ?? 今天`，请求 `to=<to>&limit=100&full_days=true`＋筛选；空则清空，否则 `transactions = data`，`loadedRange = { from: 最旧一笔的天, to: 最新一笔的天 }`。
- `loadMore()`：`to = loadedRange.from 的前一天`，请求同样带 `full_days=true`＋筛选；返回空即已到最早，停止；否则追加并更新 `loadedRange.from`。
- `loadDay(date)`：请求 `from=<date>&to=<date>&full_days=true`＋筛选，取回整天存入 `calendarDays`。
- 删除 `expandSameDay` 与 `EXPAND_CAP_WHEN_FILTERED`。

**理由**：没有半天、没有翻倍、不会溢出；三个入口收敛成同一个请求形态。

**被否决备选：** 保留 `limit` 翻倍 + 前端丢弃被切开的半天：仍会重复下载，且用户会看到某一天短暂不完整。

### D3: 交易读接口失败返回非 2xx

**决策**：`list_transactions` / `get_transaction` / `get_posting` 的错误分支返回 `(StatusCode, String)`（参数非法 400、数据库/内部 5xx），与 `create_transaction` 一致。前端 `apiFetch` 现有的 `!res.ok` 分支即可展示服务端文案。

**理由**：这是“错误被显示成 JSON 解析异常”的另一半根因。

**被否决备选：** 仅在前端容错非 JSON 的 2xx 响应：治标，服务端契约仍错。

## Risks / Trade-offs

- [单日交易极多时整页变大] → `full_days=true` 会把该天全部返回，页可能远大于 `limit`。家庭应用可接受；若实测过大，再评估天内二次分页。
- [第 1、2 步 WHERE 不一致] → 两处共用一个筛选构造，避免漏条件导致 D 与返回集不符。
- [与在途变更 `user-facing-text-i18n` 同文件] → 实现前排期，避免互踩函数签名与错误文案。
- [`to` 的上界语义] → `to` 按日期处理（`<= end_of_day(to)`），保证页首也是整天边界。

## Migration Plan

1. 后端加 `full_days` 参数与整日查询（含测试）。
2. 前端 store 改整天游标翻页，删除 `expandSameDay`（含测试）。
3. 交易读接口错误状态码。
4. `openspec validate --strict` + `cargo test/clippy/fmt` + 前端 `vitest/build/lint`。

**Rollback**：1 与 2/3 相互独立，可分别回退。

## Open Questions

- 是否让响应再带一个 `has_more`，省掉“翻到底时多一次空请求”？当前设计用“返回空即到底”，不引入响应结构变更。
