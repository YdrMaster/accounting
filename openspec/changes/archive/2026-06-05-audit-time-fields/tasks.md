# Tasks: audit-time-fields

> 回溯归档：以下任务均已实现（commits ~2026-06-05），checkbox 标记为 `[x]` 复盘。

## 1. Schema 层（`accounting-sql/src/schema.rs`）

- [x] 1.1 11 个表追加 `created_at TEXT NOT NULL DEFAULT (date('now'))` 与 `updated_at TEXT NOT NULL DEFAULT (date('now'))`
- [x] 1.2 每个表建 `update_<table>_updated_at` 触发器（带 `WHEN OLD.updated_at = NEW.updated_at` 防递归）；复合主键表用多列 `WHERE`
- [x] 1.3 `accounts` 删除 `opened_at` 列与相关约束
- [x] 1.4 `transactions` 列 `date` 重命名为 `date_time`（`TEXT NOT NULL`）
- [x] 1.5 Seed data 全面移除时间字段（含系统账户原 `opened_at`），由 DEFAULT 填充

## 2. Domain 层（`accounting`）

- [x] 2.1 `account.rs`：`Account` 删除 `opened_at` 字段
- [x] 2.2 `transaction.rs`：`Transaction.date: NaiveDate` → `date_time: NaiveDateTime`
- [x] 2.3 `transaction_filter.rs`：`start_date`/`end_date` 保持 `Option<NaiveDate>`，Repo 层用 `DATE(date_time)` 比较
- [x] 2.4 其余结构体（`Member`/`Commodity`/`Tag`/`Channel`/`Posting`/`Attachment`）不加审计字段

## 3. Repo 层（`accounting-sql`）

- [x] 3.1 `repo/account.rs`：INSERT/SELECT 去 `opened_at`；`close` 去日期参数改为 `SET closed_at = date('now')`；`reopen` 置 NULL；`map_account` 删 `opened_at` 解析
- [x] 3.2 `repo/transaction.rs`：`date` → `date_time`；INSERT/UPDATE/SELECT/map 全适配；过滤条件 `DATE(transactions.date_time)`
- [x] 3.3 其余 `repo/*`：INSERT/SELECT 统一移除审计字段

## 4. Service / CLI / API

- [x] 4.1 `account_service.rs`：`AccountService::close(id)` 去除 `closed_at: NaiveDate` 参数
- [x] 4.2 `accounting-cli/src/cmd/tx.rs`：`--date` 解析改为 `parse_date_time`，支持 `YYYY-MM-DD`（补 00:00:00）与完整时间戳两种格式
- [x] 4.3 `TransactionRow` 列 `date` → `date_time` 显示完整时间戳；`AccountRow` 删 `opened_at`

## 5. 测试

- [x] 5.1 Schema 测试：所有表含 `created_at`/`updated_at`
- [x] 5.2 Seed 测试：系统账户 `closed_at` 为 NULL、`created_at` 非 NULL
- [x] 5.3 触发器测试：UPDATE 后 `updated_at` 变化
- [x] 5.4 交易时间测试：两种 `--date` 输入格式
