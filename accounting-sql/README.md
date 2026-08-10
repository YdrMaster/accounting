# accounting-sql

数据库层：sqlx SQLite 实现。位于分层架构第二层，依赖 [`accounting`](../accounting)，为 `accounting-service` 提供 Repository trait 与 SQLite 落地。

## 职责

- Repository 模式：`AccountRepo`、`TransactionRepo`、`PostingRepo`、`BudgetRepo`、`SavingPlanRepo`、`MemberRepo`、`ChannelRepo`、`ChannelPathRepo`、`TagRepo`、`CommodityRepo`、`AttachmentRepo`、`AccountMappingRepo` 等 trait。
- 严格关系化 schema（11 张业务表 + 闭包表 + 配置/预算/攒钱计划等扩展表）。
- **闭包表** `account_ancestors` 维护账户层次，支持 `O(1)` 后代聚合查询。
- `ConnectionPool` 与 `Database` / `Transaction` trait 封装事务边界。
- 审计字段 `created_at`/`updated_at` 由 SQLite `DEFAULT(datetime('now'))` 与 `AFTER UPDATE` 触发器自动维护，应用层零侵入。
- 帐户类型不再存储列——由根账户名推导（见活规格 `account-type-resolution`）。

## 设计文档

完整的表结构、索引、约束、种子数据、Repository 设计见 [`../spec/sql.md`](../spec/sql.md)。类型推导机制的活规格见 [`../openspec/specs/account-type-resolution/spec.md`](../openspec/specs/account-type-resolution/spec.md)。审计字段决策来源见归档 [`audit-time-fields`](../openspec/changes/archive/2026-06-05-audit-time-fields/design.md) 与 [`simplify-data-model`](../openspec/changes/archive/2026-06-23-simplify-data-model/design.md)（阶段 2 升级到秒级）。

## 分层上下文

见根 [`README.md`](../README.md) 的"分层架构"与"各 crate 文档"。
