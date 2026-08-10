# Design: simplify-data-model

## Context

回溯归档 commits `e0a18bd8`/`73b28742`/`7ab89940`/`bdbba816`/`108e6934` ~2026-06-23~24，事后 Review 于 2026-06-24（确认各阶段 95~100% 完成）。起因：commit `81b307bf` 审查代码并在 `plan/` 下新增 6 份重构/分析计划；`docs/superpowers/specs/2026-06-23-refactor-plans-design.md` 从中筛选出 4 份"数据模型精简重构"分阶段落地。关联：`remove-liability` 在本重构之前移除了 Liability（5→4 类），`audit-time-fields` 建立了审计字段基线（本阶段 2 升其精度）。

> 注：活规格 `account-type-resolution` 描述类型推导**现状**（批量根名查询 + 系统根改名保护），但未记录这场"去冗余 → 推导"重构本身。本归档补录该决策链。特此说明：阶段 4 当时把类型改为「根节点名推导」，`account-type-resolution`（后续 2026-08-06 归档）在其上补了「批量查询 N+1 消除 + 系统根改名保护」——两者是同一机制的不同演进阶段。

## Goals / Non-Goals

**Goals:**

- 消除数据模型冗余字段（posting 维度归属、template 开关、`is_permanent`）。
- 审计精度升级到秒级，补 `settings` 审计字段。
- 账户层级由 `name` + `parent_id` + 闭包表表达，重命名/移动子树无需级联。
- 类型由根节点推导，靠树结构天然保证"同子树同类型"，去除存储型 `account_type` 列及运行时一致性校验。

**Non-Goals:**

- 不实现 `multi-tenant-analysis` 的多租户 schema-per-tenant（属产品化，另案）。
- 不实现 `sql-dual-backend` 的 PostgreSQL / Repo async 化（属基础设施迁移，另案）。
- 不动 `transactions.date_time`（用户指定交易时间，与审计无关）。

## Decisions

### D1: 分四阶段、每阶段独立提交可回滚

四阶段线性依赖（前置迁移 → 阶段1 冗余 → 阶段2 审计 → 阶段3 `full_name`→`name` → 阶段4 去 `account_type`）。每阶段以 `cargo fmt`/`clippy`/`test` 全过为出口条件，单独提交。

**理由**：一次性大改风险高（schema/种子/前端连锁），且 Review 难以定位回归。分阶段让每步是"稳定点"，可独立回滚。关键依赖：阶段 4 从根 `name` 推导类型，必须等阶段 3 把根存为 `name`。

### D2: 冗余字段直接删而非保留兼容期

删 `posting.member_id`、`posting.channel_id`、`transaction.is_template`、`AccountType::is_permanent()`，无过渡保留。

**理由**：① `posting.member_id`/`channel_id` 与 `transaction` 上的同名字段语义重复，统计查询早已改用 `t.member_id`/`t.channel_id`，posting 侧是死值；② `is_template` 数据库有列、CLI 有参，但无任何业务路径读写，是"半成品"残留；③ `is_permanent()` 在 `remove-liability` 后只剩 Asset 一种语义（与 `remove-liability` D2 一致）。无历史数据，直接删比兼容期成本低。SQLite 3.35+ 支持 `DROP COLUMN`，单条 DDL 完成。

### D3: 审计字段日级→秒级，前缀 `datetime('now')`

审计字段 `created_at`/`updated_at`/`closed_at` 的 DEFAULT 与触发器从 `date('now')` 改 `datetime('now')`，11 表 + 新增 `settings` 表共 12 表触发器同步。`transactions.date_time` 不动（业务时间，非审计）。

**理由**：`audit-time-fields` 建立审计基线时审计字段按日级够用，但报表调试与对账需要更细粒度（同日多笔修改的先后）。秒级增量存储成本可忽略，统一提升。`settings` 表当时漏给审计字段，本阶段补齐。

**备选**（否决）：只升级 DEFAULT 不动触发器。否决：DEFAULT 只管新增行，已有行 UPDATE 不走 DEFAULT，触发器必须同步升级否则 `updated_at` 仍日级。

### D4: `full_name` → `name` + `parent_id`，释放闭包表能力

`Account` 从 `full_name: String`（`"Assets:Bank:Checking"`）改为 `name: String`（`"Checking"`），唯一约束从 `full_name UNIQUE` 改 `UNIQUE(parent_id, name)`。按名查找 `get_by_name("Assets:Bank:Checking")` 改逐级：先找 `name="Assets" AND parent_id IS NULL`，再在其下逐级找子节点；或递归 CTE 一条 SQL。重命名中间节点只改本节点 `name`；移动子树只改目标 `parent_id` + 重维护闭包表——**均不动后代**，后代的 `name` 与路径由闭包表在查询时拼装（`display_path()`）。

**理由**：`full_name` 把路径硬编码进每行，重命名/移动任何中间节点须级联重写其后代全名，是 O(子树大小) 写。改为纯 `name` 后，路径是**推导结果**而非存储事实，写降为 O(1)，闭包表本就维护层级，能力被正确利用。前端不再 `split(':')`，改用 `parent_id` 构树。

### D5: 去 `account_type` 列，类型由根节点 `name` 推导

`Account` 删 `account_type` 字段；schema 删列、索引 `idx_accounts_type`、CHECK。`AccountType` 枚举保留为领域概念，只实现 `FromStr`（根节点 `name` → `AccountType`）。需要类型的场景（关户验证、报表分类、SQL 聚合、列表过滤）运行时：JOIN 闭包表取 `depth=MAX(depth)` 的根节点 → `AccountType::from_str(root_name)`。SQL 聚合 `GROUP BY a.account_type` 改 `JOIN account_ancestors aa ON ... aa.depth=(SELECT MAX(depth)...) JOIN accounts ra ON aa.ancestor_id=ra.id GROUP BY ra.id`。

**不变量**：去掉存储列后，"同棵子树类型一致"由**树结构天然保证**——类型只取决于根节点，子节点不可能类型不一致。`create_cascading` 的运行时类型一致性校验因此删除。

**备选**（否决）：保留 `account_type` 列但"由根节点派生写入"。否决：列是冗余事实源，派生关系须靠触发器/应用维护一致性，违反单一来源；不如运行时推导，零冗余。后续 `account-type-resolution` 在此基础上把"逐账户查根名"批量化、并保护系统根账户名不被改（稳定推导锚点）。

### D6: 前端彻底去 `split(':')`

阶段 3/4 同步重构前端：`accounting-web` 用 `parent_id` 构建账户树、过滤子节点，移除所有 `split(':')`/`pop` 路径操作；`account_type` 作为后端推导的只读字段在 DTO 返回。

**理由**：后端 `full_name` 一旦消失，前端若仍 `split` 会立即崩；趁重构一次性切换，避免半旧半新。

### D7: RULE_06 `and_hms_opt().unwrap()` 集中治理

Review 发现 9 处生产代码 `and_hms_opt(0,0,0).unwrap()` / `and_hms_opt(23,59,59).unwrap()`（虽语义上必返回 Some，但违反禁 unwrap 规范）。新建 `accounting/src/datetime_utils.rs` 集中 `start_of_day()`/`end_of_day()`（带 SAFETY 注释用 `expect`），9 处替换。CLI/pool/测试 `expect` 豁免（CLI 允许 panic，pool 锁中毒保留 unwrap，测试代码豁免）。

**理由**：散落的 `unwrap` 难维护；集中辅助函数让"0:0:0 合法"这一不变量以 SAFETY 注释固化在单一处，后续修改有据。

## Risks / Trade-offs

- **cherry-pick `fcb8fea6` 与 `4b9fa057`(remove-liability) 冲突**：两者都删 Liability 相关代码，`account.rs`/`schema.rs`/`dto.rs` 删除冲突须以"两者删除意图并集"为准。已解决（见原设计文档"解决策略"）。
- **`DROP COLUMN` 兼容性**：需 rusqlite bundled SQLite ≥ 3.35.0；若不足改"建新表→复制→删旧→重命名"。已确认版本满足。
- **`sum_by_channel()` 隐患**：删 `posting.channel_id` 后必须改 `t.channel_id` 加入 `transactions` JOIN，否则统计丢数据。已处理。
- **类型推导 N+1**：阶段 4 当时的逐账户根名查询在账户多时有 N+1；后续 `account-type-resolution` 批量化修复。本变更接受该技术债暂存。
- **根账户改名导致推导失效**：阶段 4 依赖根 `name` 稳定；若用户改根名则类型推导失效。`account-type-resolution` 后续加"禁止改系统根名"保护。

## Migration Plan

无生产数据迁移（测试库直接重建）。每阶段：改 schema/种子 → 改 Domain → 改 repo → 改 service → 改 API/CLI → 改前端 → 全套 `cargo` 校验 —— 全绿方提交。前端 `npm run build` 校验去 `split(':')` 无遗漏。

## Open Questions

- 无（回溯归档，决策已定型）。
