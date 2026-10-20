# Tasks: fix-transaction-load-expansion

## 1. 后端：整日分页（full_days）

- [x] 1.1 `accounting-sql` / `accounting-service` 的交易列表查询支持整日模式：定位最新 `limit` 笔匹配交易中最早一笔所在的天 D，返回 `[start_of_day(D), end_of_day(to)]` 内全部匹配交易，第 1、2 步复用同一筛选 WHERE。验证：新增查询测试断言「页尾对齐整天」「单日超过 limit 时整天全量返回」通过。
- [x] 1.2 `accounting-api/src/handlers/transaction.rs` 的 `list_transactions` 解析布尔参数 `full_days` 并透传；缺省 `false` 时行为不变。验证：`cargo test -p accounting-api` 断言带与不带 `full_days` 的两种响应。

## 2. 前端：整天游标翻页

- [x] 2.1 `accounting-web/src/api/client.ts` 的 `fetchTransactions` 支持透传 `full_days`。验证：`client.spec.ts` 断言请求 URL 含 `full_days=true`。
- [x] 2.2 `accounting-web/src/stores/transaction.ts`：`loadInitial` / `loadMore` / `loadDay` 改为整日模式请求；`loadMore` 游标改为「最旧一天的前一天」；删除 `expandSameDay` 与 `EXPAND_CAP_WHEN_FILTERED`。验证：store 单测覆盖初始加载、翻页、到底（返回空）即停、日历整天。

## 3. 后端：交易读接口失败返回非 2xx

- [x] 3.1 与在途变更 `user-facing-text-i18n` 排期后，将 `list_transactions` / `get_transaction` / `get_posting` 的错误返回改为 `(StatusCode, String)`（参数非法 400、数据库/内部 5xx）。验证：测试断言 `GET /api/transactions?limit=abc` 返回 400 而非 200。
- [x] 3.2 更新受返回类型变化影响的 handler 测试调用点。验证：`cargo test -p accounting-api` 全绿。

## 4. 集成验证

- [x] 4.1 手测：分别构造「全部交易都在同一天」「滚动翻页到最早一天」「单日超过 100 笔」三种数据，确认编辑并保存交易后不再出现 `Unexpected token … is not valid JSON`，且每天小计与列表完整。
- [x] 4.2 全量门禁：`cargo fmt --check`、`cargo clippy --workspace -- -D warnings`、`cargo test --workspace` 与前端 `npm run build` / `lint` / `vitest` 全部通过。
- [x] 4.3 `openspec validate fix-transaction-load-expansion --strict` 通过。

## Workflow follow-up

- 与在途变更 `user-facing-text-i18n` 协调 `transaction.rs` 同文件合并。
- 按项目评审要求归档本变更（默认 sync 活规格）。
