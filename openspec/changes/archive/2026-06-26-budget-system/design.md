## Context

项目是一个 Rust 记账系统，包含 `accounting`（核心库）、`accounting-sql`（数据库层）、`accounting-service`（业务层）、`accounting-cli`（CLI）4 个 crate。数据库为 SQLite，通过 Repository trait 抽象。账户系统使用闭包表（`account_ancestors`）维护层级关系。标签系统是扁平结构（无层级），标签挂在 Transaction 级别。当前无预算相关代码，但架构文档已规划 `accounting-budget` crate（Phase 2）。

现有统计能力：`ReportService` 提供 `stats_by_tag`/`stats_by_member`/`stats_by_channel` 以及 `summary()` 方法。`PostingRepo` 有 `sum_by_tag`/`sum_by_member`/`sum_by_channel` 等 SQL 聚合查询。`TransactionFilter` 支持多维度过滤。前端 `BudgetView.vue` 有一份硬编码的预算仪表盘 mockup。

## Goals / Non-Goals

**Goals:**
- 用户可创建多个预算表，每个预算表绑定一个周期和一组账户限额
- 系统自动按周期统计账户（含后代）的支出/收入，与限额对比计算执行情况
- 查询预算执行情况时支持任意日期，内部自动计算所在周期
- 新增两个内置标签（不计收支、不计预算）控制统计排除行为
- 核心类型定义在 `accounting` 核心库，与现有模式一致

**Non-Goals:**
- API 层（`accounting-api`）和前端（`accounting-web`）暂不实现
- 预算预警/通知功能暂不实现
- 多币种汇率折算暂不实现（当前仅统计匹配 budget.commodity_id 的分录）
- 不新建 `accounting-budget` 独立 crate（核心类型放 `accounting`，逻辑放 `accounting-service`）

## Decisions

### D1: 预算表 + 限额映射为独立实体 + 关联表（方案 A）

**选择**：budgets 表 + budget_limits 表，独立实体 + 关联表模式。

**替代方案**：
- B：JSON 存储在 settings 表 — 无法做外键约束，查询效率低，与项目模式不一致
- C：单表 + limits_json 列 — JSON 列无外键约束，修改单个限额需重写整行

**理由**：与项目现有 `tags` + `transaction_tags`、`channels` + `channel_paths` 模式一致。支持外键约束保证 account_id 引用完整性，ON DELETE CASCADE 自动清理。统计查询可高效 JOIN。

### D2: BudgetPeriod 枚举合并周起始日

**选择**：`WeeklyFromSunday` 和 `WeeklyFromMonday` 作为独立枚举变体，无额外字段。

**替代方案**：`Weekly` + `week_start_day: Option<WeekStartDay>` 字段 — 增加无效状态（如 Daily + week_start_day），需要额外验证。

**理由**：消除无效状态，类型系统保证正确性。枚举值 1-5，数据库存整数。

### D3: 限额映射到任意账户

**选择**：BudgetLimit 的 account_id 可指向任意账户（含 Income/Expense/Asset/Equity 子账户）。

**理由**：支持收入预算（如"每月工资目标"）和支出预算。非叶账户统计时通过闭包表包含所有后代。

### D4: 本币折算策略

**选择**：当前仅统计 `commodity_id = budget.commodity_id` 的分录，非本币交易不计入预算统计。

**理由**：系统暂无汇率表，简单直接。后续引入 `exchange_rates` 表后可扩展。

### D5: 两个内置标签互相独立

**选择**：`不计收支` 和 `不计预算` 互相独立，标记一个不自动应用另一个。

**理由**：灵活性最大。用户可能需要"计入收支但不算预算"或"不算收支但算预算"的场景。

### D6: 核心类型放 `accounting` 核心库

**选择**：Budget、BudgetLimit、BudgetPeriod、BudgetError 等核心类型定义在 `accounting` 核心库。

**理由**：预算类型需要持久化到数据库，与 Account、Tag、Channel 等类型同级。Service 逻辑放在 `accounting-service` 与现有 ReportService 同级。

## Risks / Trade-offs

- **[非本币交易不纳入预算统计]** → 后续引入汇率表后重构 `sum_by_account_with_descendants`，当前阶段接受此限制
- **[闭包表 JOIN 可能影响大数据量性能]** → SQLite 单文件场景下账户数量有限（通常 < 1000），闭包表加上日期范围索引足够。可在 postings 上建 `(account_id, commodity_id)` 复合索引优化
- **[限额粒度为账户而非标签]** → 部分用户可能想要按标签设预算。当前按账户更符合复式记账体系，后续可通过"虚拟账户+标签映射"实现