# Design: unify-income-expense-stats

## Context

动机见 proposal.md。技术上需要统一的实现点有四处：

| 位置 | 当前口径 | 代码 |
| --- | --- | --- |
| 日历格子（后端） | A 资产根·逐分录 | `accounting-sql/src/repo/posting.rs::posting_daily_summary` |
| 交易列表日头 / 卡片（前端） | B 资产根·逐交易净额 | `accounting-web/src/utils/amount.ts::sumAssetAmount` |
| 交易页月 hero（前端） | C 收支分类账户 | `accounting-web/src/views/TransactionView.vue` |
| 现金流量 / 预算报表（后端） | C 收支分类账户 | `report/cash_flow.rs`、`report/budget.rs` |

约束：账户类型由根账户名推导；后端金额以整数最小单位存储、按 commodity 精度换算（`from_db_amount`）；前端以 `Decimal`（字符串）计算。当前实际库仅 CNY 一个 commodity。

## Goals / Non-Goals

**Goals:**

- 让所有「收支汇总统计」共享同一定义（B 资产根·逐交易净额），消除日历/日头/月 hero 之间的数值分歧。
- 让前端存在唯一一份口径实现（`sumAssetAmount`），后端存在唯一一份（改写后的 `posting_daily_summary`）。
- 删除已无调用方的 `posting_summary`/`PostingSummary` 与其陈旧 spec 要求。

**Non-Goals:**

- 不改现金流量报表与预算执行报表的逐账户口径（见 D5）。
- 不新增周期汇总 API 端点，不改月 hero 的数据来源与加载窗口（见 D3、Open Questions）。
- 不改数据库 schema、不做数据迁移。

## Decisions

### D1: 统一口径取 B「资产根·逐交易净额」

对每笔交易取其全部资产根分录金额之和 `S`：`S > 0` 计入收入 `S`，`S < 0` 计入支出 `|S|`，`S = 0` 不计入；非资产根分录不参与。

- 理由：B 是交易列表既有且已被 spec（`transaction-list-ui` 金额计算规则）固定的口径，也是用户在本次问题中确认的基准；它天然排除纯转账，又满足 `transaction-summary-api` 的「仅 Assets 根」终局决策（期初余额按资产侧计入收入，Equity 侧不计）。
- 否决 A（资产根·逐分录）：把纯转账计成收入 + 支出各一笔，正是本次不一致的根源。
- 否决 C（收支分类账户）：与 `transaction-summary-api`「仅 Assets 根、资产增加即收入」的终局决策冲突（期初余额不入账），且退款会以负支出呈现。

### D2: 后端 daily-summary 改为「先按交易净额、再按天拆分」

`posting_daily_summary` 由单层「逐分录 `SUM(CASE ...)`」改为两级聚合：

```sql
SELECT day, commodity_id,
       SUM(CASE WHEN tx_net > 0 THEN tx_net  ELSE 0 END) AS income,
       SUM(CASE WHEN tx_net < 0 THEN -tx_net ELSE 0 END) AS expense
FROM (
  SELECT date(t.date_time) AS day, p.commodity_id, t.id AS tx_id,
         SUM(p.amount) AS tx_net
  FROM postings p
  JOIN transactions t ON p.transaction_id = t.id
  JOIN accounts a ON p.account_id = a.id
  JOIN account_ancestors aa ON a.id = aa.account_id
  JOIN account_names an ON aa.ancestor_id = an.account_id
       AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
  WHERE an.is_system = 1 AND an.name = 'Assets'
    AND t.date_time >= ?1 AND t.date_time <= ?2
  GROUP BY t.id, p.commodity_id
)
GROUP BY day, commodity_id
ORDER BY day
```

内层按 `(transaction_id, commodity_id)` 求资产净额，外层按 `(day, commodity_id)` 符号拆分后求和，最后仍按 commodity 精度换算并在同日合并（沿用既有 `load_precisions` + `from_db_amount` 逻辑）。`S = 0` 的交易自然贡献 0，从而「纯转账日不出现金额」。

- 理由：与前端 `sumAssetAmount` 的语义一一对应（前端按交易聚合、后端按交易聚合），且保留多币种精度处理。
- 备选：把净额算法放前端、用一个新端点返回整月交易明细——否决，会把整月明细拉到前端，且违背「日历格子一次聚合查询」的既有设计（`calendar-daily-stats`）。

### D3: 前端月 hero 复用 `sumAssetAmount`

`TransactionView.vue` 的 `monthlyExpense`/`monthlyIncome`/`monthlyBalance` 改为遍历已加载交易并对每笔调用 `sumAssetAmount`：收入 = `Σmax(S,0)`、支出 = `Σmax(-S,0)`、结余 = `ΣS`。删除其中按 `p.account_type === 'expense'/'income'` 的求和分支。

- 理由：口径实现单一来源，月 hero 与日头、卡片必然一致；改动面小。
- 备份/否决：新增（或复活）`GET /api/reports/summary` 周期汇总端点让 hero 走后端——本期不做：一是 spec 已决定移除此端点，二是它引入新契约与数据流，与「统一口径」目标无关；hero 的加载窗口问题是既有问题（见 Open Questions）。

### D4: 删除死代码 `posting_summary` / `PostingSummary`

全仓检索确认：`PostingSummary` 与 `posting_summary` 仅被 `accounting-sql/src/repo/facade.rs` 的 facade 声明和 `posting.rs` 的一个单测引用，无 handler、service、CLI 或前端调用；其唯一出口 `GET /api/reports/summary` 已在 `restructure-report-module` 中移除。

处理：删除 `posting_summary` 函数、`PostingSummary` 结构、facade 声明，以及单测 `test_summary_and_assets_not_doubled` 中针对它的断言（保留同测试内 `posting_sum_all_assets` 不得因 JOIN 翻倍的断言，该查询仍被资产负债表使用）。

- 否决「保留并改写为逐交易净额」：没有任何调用方，改写不产生行为收益，违背本次清理目标。

### D5: 现金流量 / 预算报表保持分类账户口径（明确不做）

`cash_flow.rs`、`budget.rs` 及 `category-breakdown-report` 的产出是**按 Income/Expenses 账户拆分的层级明细**，预算还需对具体账户设限额。资产口径按定义丢失「账户归属」，无法生成这类明细。故本变更不修改它们，仅保证「转账不计入」这一点在两类报表中本就成立（转账没有 Income/Expenses 分录，除非含手续费/折扣这种本应计入的分录）。

### D6: spec 处理

- `transaction-summary-api`：MODIFIED「汇总计算规则」「按天收支汇总端点」为逐交易净额；REMOVED 陈旧的「收支汇总 Service 接口」「收支汇总 API 端点」（`/api/reports/summary` 与 `ReportService` 均不存在）。
- `transaction-list-ui`：MODIFIED「金额计算规则」，把月收支汇总纳入并钉死为同一口径，防止回归。
- 直接更新 live `transaction-summary-api` 的 `## Purpose`，去掉已移除端点的描述（按 openspec 工具指引，Purpose 变更改主 spec 而非 delta）。

## Risks / Trade-offs

- [日历/月 hero 在纯转账日不再显示金额，用户可能误以为丢失] → 该行为与交易列表日头完全一致，且 `transaction-list-ui` 已定义；发布说明点明「转账不计入收支」。
- [多币种下前端 `sumAssetAmount` 跨币种直接相加，与后端按币种分别净额可能不同] → 当前仅 CNY，二者等价；记录为 Open Question，不在本期处理。
- [月 hero 仅统计已加载交易，月内交易数超过加载窗口时数值偏小] → 既有问题（本次仅统一口径、不改变数据窗口）；记录为 Open Question。
- [删除 `posting_summary` 影响其他 crate 编译] → 全仓已确认无跨 crate 引用；删除后跑 workspace 全量编译与测试。

## Migration Plan

- 无数据库迁移、无数据回填。
- 发布即生效：日历格子与交易列表日头从此一致。
- 回滚：纯代码 + spec 变更，按常规 revert 即可（不涉及数据）。

## Open Questions

- 月 hero 是否改为调用服务端按天汇总覆盖整月（解决加载窗口导致的数值偏小）？本期不做，可作为后续独立变更。
- 多币种交易的收支归属（按币种各自净额，还是跨币种合并净额）是否需要明确？当前仅 CNY，暂不决策。
