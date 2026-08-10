# Proposal: simplify-data-model

> 回溯归档：本变更所述设计已实现（commits `e0a18bd8`、`73b28742`、`7ab89940`、`bdbba816`、`108e6934` ~2026-06-23~24，Review 于 2026-06-24）。活规格 `account-type-resolution` 描述了类型推导的**现状**，但本归档补录那场"从冗余到推导"重构的**决策来源**（4 阶段 + 前置迁移）。`--skip-specs` 归档。

## Why

提交 `81b307bf`（审查代码、提出修改意见）在 `plan/` 下新增 6 份重构/分析计划。核心数据模型在多轮迭代后积累了冗余与不一致：

- `Posting` 上冗余存 `member_id`/`channel_id`，与 `Transaction` 上的同名字段重复且口径不一。
- `Transaction.is_template` 是从未走通的功能开关（数据库有列、CLI 有参，无业务路径）。
- 审计字段精度停留在日级（`date('now')`），报表调试与对账粒度不足。
- `Account` 用 `full_name`（`Assets:Bank:Checking`）作为唯一标识，重命名/移动子树需级联重写后代全名，闭包表的层级能力被浪费。
- `Account` 上存 `account_type` 列，与"类型由树根决定"的不变量重复，且同一棵子树类型一致性靠运行时校验而非结构天然保证。

`docs/superpowers/specs/2026-06-23-refactor-plans-design.md` 将 `plan/` 的 6 份计划筛选为 4 份"数据模型精简重构"（排除多租户分析与 PostgreSQL 双后端）分阶段落地。

## What Changes

四阶段顺序执行（每阶段独立提交、可回滚）：

1. **前置迁移** `fcb8fea6`：移除 `Account.position` 字段与前端拖拽排序（cherry-pick 已在 main 落地的提交）。
2. **阶段 1 移除冗余字段**：删 `posting.member_id`、`posting.channel_id`、`transaction.is_template`、`AccountType::is_permanent()`。
3. **阶段 2 审计字段改进**：审计字段精度日级→秒级（`date('now')`→`datetime('now')`，12 表 + 触发器）；`settings` 表补 `created_at`/`updated_at` + 触发器。
4. **阶段 3 账户名称重构**：`Account.full_name` → `name`（只存本节点名），层级靠 `parent_id` + 闭包表；唯一约束改 `UNIQUE(parent_id, name)`；按名查找改逐级查找或递归 CTE；重命名/移动子树无需级联。
5. **阶段 4 账户类型重构**：删 `Account` 上的 `account_type` 列、索引、CHECK；`AccountType` 保留为领域概念，只实现 `FromStr`（从根节点 `name` 推导）；需要类型的场景运行时 JOIN 闭包表取根节点推导。

依赖：阶段 3 必须在阶段 4 之前（阶段 4 从根 `name` 推导，阶段 3 才把根存为 `name`）。阶段 1/2 较独立靠前。

## Capabilities

### New Capabilities

（无——内部数据模型精简，不构成面向用户新能力。类型推导机制在活规格 `account-type-resolution` 已记录。）

### Modified Capabilities

（无——活规格已反映重构后行为，本归档补决策来源。）

## Impact

- `accounting/src/account.rs`、`transaction.rs`、`posting.rs`、`transaction_filter.rs`、`closure.rs`、`account_type.rs`、`datetime_utils.rs`（新建，集中 `start_of_day`/`end_of_day`）：去冗余字段、`full_name`→`name`、去 `account_type` 字段、`FromStr` 推导。
- `accounting-sql/src/schema.rs`：`postings`/`transactions` 删列；审计精度升级；`accounts` 改 `name`+`UNIQUE(parent_id,name)`、删 `account_type` 列/索引/CHECK；种子调整。
- `accounting-sql/src/repo/account.rs`：`get_by_name` 改递归逐级；`map_account` 去类型映射。
- `accounting-sql/src/repo/posting.rs`：统计聚合改 `JOIN account_ancestors` + `JOIN accounts ra` 取根节点 GROUP BY。
- `accounting-service/src/account_service.rs`：`create_cascading` 重写不拼路径；去类型一致性校验（结构天然保证）。
- `accounting-service/src/report/mod.rs` / `cash_flow.rs`：根名调用点适配。
- `accounting-api`/`accounting-cli`：DTO/参数/输出去冗余字段、`account_type` 改只读推导、`--type` 按根节点过滤。
- `accounting-web`：去所有 `split(':')`，用 `parent_id` 构树。
- 验证：每阶段 `cargo fmt`/`cargo clippy --workspace --all-targets`/`cargo test --workspace` 全过方提交。
