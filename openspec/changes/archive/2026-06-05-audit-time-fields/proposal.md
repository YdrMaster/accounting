# Proposal: audit-time-fields

> 回溯归档：本变更所描述的设计早已在代码中实现（commits ~2026-06-05），此处补录决策来源。活规格未单独建审计字段能力（属内部 schema 演进，不承载面向用户的行为规格），故本变更以归档 `design.md` 为主，`--skip-specs` 归档。

## Why

数据库时间字段语义混杂，缺乏一致的审计能力：

- `accounts.opened_at` 被当作"开户日期"（业务语义），却硬编码为 `2000-01-01`，既非真实开户日也非审计字段。
- `transactions.created_at` 已是审计字段，但格式与其他表不统一。
- `commodities`、`members`、`channels`、`tags`、`postings`、`attachments` 等大批表完全没有时间字段，无法追踪"何时写入 / 何时修改"。

后果：排障困难（无法判断数据新旧）、审计能力缺失、各表时间字段含义不一导致认知负担。

## What Changes

- **统一为纯审计字段**：所有表追加 `created_at` / `updated_at`，由 SQLite `DEFAULT` 或 `AFTER UPDATE` 触发器自动生成，应用层禁止传入、禁止覆盖。
- **删除 `accounts.opened_at`**：由 `created_at` 替代（"开户日"本就是创建日的审计语义，硬编码值无保留价值）。
- **`transactions.date` 重命名为 `date_time`**：从 `NaiveDate` 升级为 `NaiveDateTime`，精确到秒，保留**业务语义**（用户指定的交易时间）——这是唯一保留业务语义的时间字段。
- **精度分层**：审计字段（`created_at`/`updated_at`/`closed_at`）精确到日（`date('now')`），业务时间（`transactions.date_time`）精确到秒。精度后来在「审计字段改进」中升级到秒级（见 `simplify-data-model` 的阶段 2）。
- **分层隔离**：`created_at`/`updated_at` 不进入 Domain 模型（纯审计，与业务无关），不暴露给 CLI/API 输出。
- **移除 Seed 中所有时间字段**：由 `DEFAULT` 自动填充。

## Capabilities

### New Capabilities

（无——审计字段为内部 schema 演进，不构成面向用户的能力规格。）

### Modified Capabilities

（无。）

## Impact

- `accounting-sql/src/schema.rs`：11 个表各加 `created_at`/`updated_at` 两列 + `update_<table>_updated_at` 触发器；`accounts` 删 `opened_at`；`transactions` 列 `date` → `date_time`；seed data 去所有时间字段。
- `accounting/src/account.rs`：删 `opened_at`。
- `accounting/src/transaction.rs`：`date: NaiveDate` → `date_time: NaiveDateTime`。
- `accounting-sql/src/repo/account.rs`：INSERT/SELECT 去 `opened_at`，`close` 去日期参数，`map_account` 简化。
- `accounting-sql/src/repo/transaction.rs`：`date` → `date_time`，过滤条件用 `DATE(date_time)`。
- `accounting-service/src/account_service.rs`：`AccountService::close` 签名变更。
- `accounting-cli/src/cmd/tx.rs`：`--date` 支持 `YYYY-MM-DD` 与 `YYYY-MM-DD HH:MM:SS` 两种格式（前者补全为 00:00:00）。
- 兼容性：旧库需重建（已确认无历史数据）。
