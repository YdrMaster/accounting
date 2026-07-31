# Design: saving-plan

## Context

预算（Budget）已全链路落地：核心域 `accounting/src/budget.rs`、SQL `accounting-sql/src/{schema,repo/budget,database}.rs`、service `accounting-service/src/report/budget.rs`、API `accounting-api/src/handlers/budget.rs`、CLI `accounting-cli/src/cmd/budget.rs`。预算是「支出流量的上限」：每账户一条限额，按 `FinancePeriod` 自然周期（daily/weekly-sun/weekly-mon/monthly/yearly）滚动计量，`actual ≤ limit` 达标。

攒钱计划（SavingPlan）是镜像需求：「资产存量的下限」。两个典型场景：

- 一次性：10 月旅行，要求 9 月底余额宝+微信零钱**合计** ≥ 5000 元（旅行基金）。
- 循环：每月 30 日前招行账户须留有 6000 元交房租。

约束与现状：

- schema 无迁移框架，全部 DDL 为 `SCHEMA_STATEMENTS` 中的 `CREATE TABLE IF NOT EXISTS` 幂等语句，打开数据库时执行。
- 既有 `budgets.period` 为 `INTEGER NOT NULL`，既有库（my.db）改可空需重建表。
- 账户类型（Asset/Equity/Income/Expense）不在账户行上，由根账户名推导；层级聚合走 `account_ancestors` 闭包表。
- 预算的实际值聚合 `posting_sum_by_period` 是带时间窗的流量聚合，且**不含后代**；含后代聚合另有 `sum_by_account_with_descendants` 可参照。

## Goals / Non-Goals

**Goals:**

- 攒钱计划全链路：core 模型 → SQL → service → REST API → CLI（不含前端 UI）。
- 账户集合共享一个目标金额（pooled target），余额口径含后代子账户，实时可查缺口。
- 预算与攒钱计划统一新时间语义：`period` 可空（一次性/无节奏）+ `deadline` 失效列。
- 校验收紧：预算仅 Expenses 子树，攒钱计划仅 Assets 子树（写入时校验）。

**Non-Goals:**

- 前端 UI（budget-view / saving-plan 页面均不在本次范围）。
- `FinancePeriod` 周期计算器接口不变（曾讨论返回 `Option` 的时间轴截断方案，已被更朴素的 deadline 列取代）。
- 历史数据迁移清理（既有库中若有挂在非支出账户上的预算，保持原样，仅在下次更新时被校验拦截）。
- 攒钱计划的「期间净流入」统计（只认余额，不认流量）。
- 预算的含后代聚合改造（维持现状，仅攒钱计划含后代）。

## Decisions

### D1: 共享目标（pooled target）而非每账户目标

攒钱计划一张表存一个 `target_amount`，账户集合存关联表 `saving_plan_accounts`。旅行基金语义是「余额宝+微信零钱合计 5000」，用户不关心钱在哪个账户；每账户拆目标（镜像 `budget_limits`）会强迫用户手动分配金额，且判定逻辑变复杂。

备选：镜像 `budget_limits` 每账户一条目标 —— 拒绝，语义不符。

### D2: 余额（存量）口径，实时判定

判定式：`gap = target − Σ balance(账户集合, 截至查询日)`，`balance ≥ target` 即达标。余额含后代（走闭包表聚合）。状态随时可查，不等期末——周期只提供「节奏」（当前周期区间、下一个检查点），不锁计算窗口。房租场景中付完房租余额跌落，缺口立即出现提醒补齐，正是期望行为。

备选：期间净流入（流量口径）—— 拒绝，「账上有没有这笔钱」才是用户的直觉语义；且流量口径无法处理「历史已有存款」。

### D3: 时间语义 = nullable period + deadline 列，service 层判定失效

- `period INTEGER NULL`：NULL = 一次性/无节奏；非 NULL = 按 `FinancePeriod` 循环。
- `deadline TEXT NULL`（`'YYYY-MM-DD'`）：NULL = 永久有效；`查询日 > deadline` 时计划失效。
- 失效判定在 service 层做（`deadline.map_or(false, |d| date > d)`），`FinancePeriod::period_range` 签名不变。
- 四种组合：`(NULL, NULL)`=永久下限（应急金）、`(NULL, ddl)`=一次性（旅行基金）、`(period, NULL)`=永久循环（房租/餐饮预算）、`(period, ddl)`=限期循环（季度差旅）。

备选：（a）周期计算器返回 `Option`、时间轴在 deadline 截断 —— 用户评估后放弃，改动面大收益小；（b）`start_date + end_date` 生效区间 —— 拒绝，`start_date` 场景稀薄，引入第三个时间概念。

### D4: 一次性预算的计量窗口 = 全部历史累计

预算 `period` 为 NULL 时，实际值 = 从最早记录到 min(查询日, deadline) 的 posting 合计（不限下界）。不新增列。已知副作用：计划创建前的历史交易会被计入。

注：`budgets` 表现有 `created_at` 列，本可作为窗口下界，但用户明确选择「全部历史累计」的朴素语义；若日后产生困扰可平滑收紧为 `max(created_at, ...)`，无需 schema 变更。

### D5: 失效后状态返回 200 + `expired: true`

status 接口对已过期的计划/预算返回 200，响应体含 `expired: true`（及最后有效信息），CLI 显示「已失效」。优于 404：客户端可区分「不存在」与「已结束」，且对既有前端向后兼容（新增可选字段）。

### D6: 账户类型限制在 service 层校验

预算账户须位于 Expenses 根子树内，攒钱计划账户须位于 Assets 根子树内。校验时机为 create/update，复用闭包表判断祖先。不迁移历史数据。

### D7: 余额聚合新增 SQL 函数

新增 `account_balance_by_ids(db, account_ids, commodity_id, as_of_date) -> Decimal`：按闭包表展开后代，`SUM(postings)` 过滤 `t.date_time <= end_of_day(as_of)` 与 commodity。参照 `sum_by_account_with_descendants` 加时间上限。攒钱计划状态只调这一个聚合；`exclude-from-budget` 标签排除规则**不适用**于攒钱计划（余额是事实，不是可豁免的消费）。

## Risks / Trade-offs

- [既有库 `budgets.period` 为 NOT NULL，SQLite 无法改列约束] → 已确认无存量数据，不做迁移；旧库文件删除重建（见 Migration Plan）。
- [一次性预算混入创建前历史交易] → D4 已述，语义文档化；用户可用专用账户规避。
- [校验收紧是 BREAKING，既有 API 客户端/前端可能给预算挂非支出账户] → 前端当前版本只挂支出账户（预算页语义即消费预算）；写入时报 400 语义明确。
- [pooled 余额把负债账户（Assets 下负余额的信用卡）拉低合计] → 语义正确（负债确实消耗可动用资金），文档说明即可。
- [预算与攒钱计划时间语义统一但结构不对称（per-account vs pooled）] → 有意为之：流量天然按账户归属，存量目标天然跨账户归集。

## Migration Plan

无迁移：确认项目无存量数据，新 schema（budgets 可空 period + deadline、saving_plan 三表）随 `CREATE TABLE IF NOT EXISTS` 直接生效。旧库文件（如 my.db）删除重建即可，不提供升级路径。

回滚：删除追加的 DDL 与新代码即可，无数据负担。

## Open Questions

- 无。（CLI `--period` 在攒钱计划 create 上做成可选、缺省 NULL；预算 create 的 `--period` 也放宽为可选——已含在 proposal 的 budget-cli delta 中。）
