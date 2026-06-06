# 审计时间字段统一设计方案

## 背景与目标

当前数据库中时间字段语义混杂：

- `accounts.opened_at` 被当作"开户日期"（业务语义），但硬编码为 `2000-01-01`
- `transactions.created_at` 已是审计字段，但格式不统一
- 大量表没有时间字段，缺乏基础审计能力

本方案的目标：**将所有时间字段统一为纯数据库审计字段，仅 `transactions.date_time` 保留业务语义（用户指定的交易时间）。**

## 设计原则

1. **数据库驱动**：所有审计时间由 SQLite `DEFAULT` 或触发器自动生成，应用层禁止传入、禁止覆盖。
2. **分层隔离**：`created_at` / `updated_at` 不暴露给 Domain 模型（纯审计，与业务无关）。
3. **精度统一**：
   - 审计字段（`created_at`, `updated_at`, `closed_at`）：精确到**日**（`date('now')`）
   - 业务时间（`transactions.date_time`）：精确到**秒**（`YYYY-MM-DD HH:MM:SS`）
4. **零业务侵入**：除 `transactions.date_time` 外，时间字段不进入 CLI 输出、不参与业务逻辑。

## Schema 变更

### 通用规则（所有 11 个表）

每个表追加以下两列：

```sql
created_at  TEXT NOT NULL DEFAULT (date('now')),
updated_at  TEXT NOT NULL DEFAULT (date('now'))
```

### 各表具体变更

| 表名 | 变更 |
|---|---|
| `commodities` | 新增 `created_at`, `updated_at` |
| `accounts` | 删除 `opened_at`（由 `created_at` 替代）；保留 `closed_at`；新增 `updated_at` |
| `account_ancestors` | 新增 `created_at`, `updated_at` |
| `account_owners` | 新增 `created_at`, `updated_at` |
| `members` | 新增 `created_at`, `updated_at` |
| `channels` | 新增 `created_at`, `updated_at` |
| `tags` | 新增 `created_at`, `updated_at` |
| `transactions` | `date` 重命名为 `date_time`（`TEXT NOT NULL`）；保留 `created_at`；新增 `updated_at` |
| `postings` | 新增 `created_at`, `updated_at` |
| `attachments` | 新增 `created_at`, `updated_at` |
| `transaction_tags` | 新增 `created_at`, `updated_at` |

### `accounts` 表最终结构

```sql
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL UNIQUE,
    account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 5),
    parent_id INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (date('now')),
    closed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (date('now')),
    is_system INTEGER NOT NULL DEFAULT 0,
    billing_day INTEGER CHECK(billing_day BETWEEN 1 AND 31),
    repayment_day INTEGER CHECK(repayment_day BETWEEN 1 AND 31)
);
```

### `transactions` 表最终结构

```sql
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date_time TEXT NOT NULL,
    description TEXT NOT NULL,
    member_id INTEGER REFERENCES members(id),
    is_template INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);
```

### 触发器（`updated_at` 自动更新）

每个表配备一个 `AFTER UPDATE` 触发器：

```sql
CREATE TRIGGER IF NOT EXISTS update_<table>_updated_at
AFTER UPDATE ON <table>
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE <table>
    SET updated_at = date('now')
    WHERE <pk> = NEW.<pk>;
END;
```

**复合主键表**（`account_ancestors`, `account_owners`, `transaction_tags`）使用多列 `WHERE`：

```sql
CREATE TRIGGER IF NOT EXISTS update_account_ancestors_updated_at
AFTER UPDATE ON account_ancestors
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE account_ancestors
    SET updated_at = date('now')
    WHERE account_id = NEW.account_id AND ancestor_id = NEW.ancestor_id;
END;
```

> `WHEN OLD.updated_at = NEW.updated_at` 条件防止递归：触发器内部再次触发时，`NEW.updated_at` 已被修改，与 `OLD.updated_at` 不同，条件为假，停止递归。

### Seed Data

所有 `INSERT` 语句**完全移除**时间字段（包括系统账户原硬编码的 `opened_at`），由 `DEFAULT` 自动填充。

## Domain 模型变更

### `Account` 结构体

```rust
pub struct Account {
    pub id: AccountId,
    pub full_name: String,
    pub account_type: AccountType,
    pub parent_id: Option<AccountId>,
    pub closed_at: Option<NaiveDate>,   // 保留：判断账户是否关闭
    pub is_system: bool,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
    // 删除 opened_at
    // 不添加 created_at / updated_at
}
```

### `Transaction` 结构体

```rust
pub struct Transaction {
    pub id: TransactionId,
    pub date_time: NaiveDateTime,       // 原 date，精确到秒
    pub description: String,
    pub member_id: Option<MemberId>,
    pub is_template: bool,
}
```

### `TransactionFilter` 结构体

保持 `start_date: Option<NaiveDate>` 和 `end_date: Option<NaiveDate>`，按日期范围过滤。Repo 层 SQL 用 `DATE(date_time)` 函数比较，无需改动 Domain 模型。

### 其他所有结构体

`Member`, `Commodity`, `Tag`, `Channel`, `Posting`, `Attachment` 均**不添加**审计字段。

## Repo 变更

### 通用规则

- **所有 INSERT**：完全移除 `created_at` / `updated_at` 列，由 DEFAULT 填充。
- **所有 UPDATE**：无需显式设置 `updated_at`（触发器处理）。
- **所有 SELECT**：不查询 `created_at` / `updated_at`。

### `AccountRepo` 特殊变更

| 方法 | 变更 |
|---|---|
| `create` / `create_with_closure` | INSERT 去掉 `opened_at` 列 |
| `close` | 不再接收 `closed_at: NaiveDate` 参数；`UPDATE accounts SET closed_at = date('now') WHERE id = ?` |
| `reopen` | `UPDATE accounts SET closed_at = NULL WHERE id = ?` |
| `get` / `list` / `list_children` / `get_by_name` | SELECT 去掉 `opened_at` 列 |
| `map_account` | 删除 `opened_at` 解析逻辑；`closed_at` 保持原逻辑 |

### `TransactionRepo` 特殊变更

| 方法 | 变更 |
|---|---|
| `insert` | `date` 列改为 `date_time`；`tx.date.to_string()` → `tx.date_time.to_string()` |
| `update` | `SET date = ?` → `SET date_time = ?` |
| `get` / `list` / `count` | SELECT `date` → `date_time`；过滤条件 `transactions.date` → `DATE(transactions.date_time)` |
| `map_transaction` | `date` 解析改为 `date_time` 解析（`%Y-%m-%d %H:%M:%S`） |

## Service 变更

- `AccountService::close(id)` — 删除 `closed_at: NaiveDate` 参数
- `TransactionService` — 无接口变更（`Transaction` 内部字段改名，Service 透传）
- 所有测试中的 `Account` 构造：删除 `opened_at` 字段
- 所有测试中的 `Transaction` 构造：`date` → `date_time`

## CLI 变更

### `account` 命令

- `account add` — 构造 `Account` 时不再设置 `opened_at`
- `account close <id>` — 不再接受/传递日期参数
- `AccountRow` — 删除 `opened_at` 显示列

### `tx` 命令

- `--date` 参数支持两种格式：
  - `YYYY-MM-DD` → 自动补全为 `YYYY-MM-DD 00:00:00`
  - `YYYY-MM-DD HH:MM:SS` → 直接使用
- `TransactionRow` — `date` 列改为 `date_time`，显示完整时间戳

### 解析函数

```rust
fn parse_date_time(s: &str) -> Result<NaiveDateTime, AccountingError> {
    // 先尝试完整格式
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    // 再尝试日期格式，补 00:00:00
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .map_err(|_| AccountingError::InvalidDate(format!("时间格式应为 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS: {}", s)))
}
```

## 测试策略

1. **Schema 测试**：验证所有表包含 `created_at` 和 `updated_at` 列
2. **Seed Data 测试**：验证系统账户 `closed_at` 为 NULL，`created_at` 有值（非 NULL）
3. **触发器测试**：UPDATE 某条记录后验证 `updated_at` 发生变化
4. **Transaction 日期时间测试**：验证 `YYYY-MM-DD` 和 `YYYY-MM-DD HH:MM:SS` 两种输入格式
5. **过滤测试**：`TransactionFilter` 按日期范围过滤 `date_time` 仍然正确

## 影响范围总结

| 层级 | 文件 | 改动量 |
|---|---|---|
| Schema | `accounting-sql/src/schema.rs` | **大** — 所有表加两列 + 触发器，`accounts` 删 `opened_at`，`transactions` 重命名 `date` → `date_time`，seed data 去时间 |
| Domain | `accounting/src/account.rs` | **中** — 删除 `opened_at` |
| Domain | `accounting/src/transaction.rs` | **中** — `date` → `date_time: NaiveDateTime` |
| Repo | `accounting-sql/src/repo/account.rs` | **大** — INSERT/SELECT 去 `opened_at`，`close` 去参数，`map_account` 简化 |
| Repo | `accounting-sql/src/repo/transaction.rs` | **大** — `date` → `date_time`，过滤条件适配 |
| Repo | `accounting-sql/src/repo/*`（其他） | **中** — 各 repo INSERT/SELECT 去时间字段 |
| Service | `accounting-service/src/account_service.rs` | **中** — `close` 签名变更，测试调整 |
| Service | `accounting-service/src/transaction_service.rs` | **小** — 测试中的 `Transaction` 构造改字段名 |
| CLI | `accounting-cli/src/cmd/account.rs` | **中** — `close` 去日期，`add` 去 `opened_at` |
| CLI | `accounting-cli/src/cmd/tx.rs` | **中** — `--date` 解析改为 `date_time`，支持两种格式 |
| CLI | `accounting-cli/src/cmd/mod.rs` | **小** — `AccountRow` 去 `opened_at`，`TransactionRow` `date` → `date_time` |
| 其他 | `accounting-sql/src/impls/sqlite.rs` 等 | **中** — 所有硬编码 `opened_at` / `date` 删除或改名 |
