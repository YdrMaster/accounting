# 审计时间字段统一 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将所有表统一配备 `created_at` + `updated_at` 审计字段（数据库驱动、日级精度），`accounts.opened_at` 退化为审计字段并删除，`transactions.date` 改名为 `date_time`（秒级、用户指定）。

**架构：** 所有 DDL 变更集中在 `schema.rs`；Domain 模型删除 `opened_at` 并将 `Transaction.date` 改为 `NaiveDateTime`；Repo 层所有 INSERT/UPDATE 不再传入时间；Service/CLI 相应调整接口。

**技术栈：** Rust, SQLite (rusqlite), chrono

---

## 文件变更清单

| 文件 | 职责 | 变更类型 |
|---|---|---|
| `accounting/src/account.rs` | Account Domain 模型 | 删除 `opened_at` 字段 |
| `accounting/src/transaction.rs` | Transaction Domain 模型 | `date` → `date_time: NaiveDateTime` |
| `accounting-sql/src/schema.rs` | 数据库 Schema + Seed + 触发器 | **大改** |
| `accounting-sql/src/repo/account.rs` | Account 仓库 | 去 `opened_at`，`close` 去参数 |
| `accounting-sql/src/repo/transaction.rs` | Transaction 仓库 | `date` → `date_time`，过滤适配 |
| `accounting-sql/src/repo/member.rs` | Member 仓库 | INSERT 去时间字段 |
| `accounting-sql/src/repo/commodity.rs` | Commodity 仓库 | INSERT 去时间字段 |
| `accounting-sql/src/repo/tag.rs` | Tag 仓库 | INSERT 去时间字段 |
| `accounting-sql/src/repo/channel.rs` | Channel 仓库 | INSERT 去时间字段 |
| `accounting-sql/src/repo/posting.rs` | Posting 仓库 | INSERT 去时间字段 |
| `accounting-sql/src/repo/attachment.rs` | Attachment 仓库 | INSERT 去时间字段 |
| `accounting-service/src/account_service.rs` | Account 服务 | `close` 去参数，测试调整 |
| `accounting-service/src/transaction_service.rs` | Transaction 服务 | 测试调整 |
| `accounting-cli/src/cmd/account.rs` | Account CLI | `close` 去日期，`add` 去 `opened_at` |
| `accounting-cli/src/cmd/tx.rs` | Transaction CLI | `--date` 解析改 `date_time` |
| `accounting-cli/src/cmd/mod.rs` | CLI Row 定义 | `AccountRow` 去 `opened_at`，`TransactionRow` 改 `date_time` |
| `accounting-sql/src/impls/sqlite.rs` | SQLite 实现测试 | 删除 `opened_at` |
| `accounting-service/src/report_service.rs` | Report 服务测试 | 删除 `opened_at` |

---

## 阶段 1：Domain 模型

### 任务 1：Account 删除 `opened_at`

**文件：** `accounting/src/account.rs`

- [ ] **步骤 1：删除 `opened_at` 字段**

```rust
pub struct Account {
    pub id: AccountId,
    pub full_name: String,
    pub account_type: AccountType,
    pub parent_id: Option<AccountId>,
    pub closed_at: Option<NaiveDate>,
    pub is_system: bool,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
}
```

- [ ] **步骤 2：同步测试代码**

测试中的 `Account` 构造删除 `opened_at` 行。

- [ ] **步骤 3：验证**

```bash
cargo test -p accounting
```

---

### 任务 2：Transaction `date` → `date_time`

**文件：** `accounting/src/transaction.rs`

- [ ] **步骤 1：字段改名改类型**

```rust
use chrono::NaiveDateTime;

pub struct Transaction {
    pub id: TransactionId,
    pub date_time: NaiveDateTime,
    pub description: String,
    pub member_id: Option<MemberId>,
    pub is_template: bool,
}
```

- [ ] **步骤 2：验证**

```bash
cargo test -p accounting
```

---

## 阶段 2：Schema

### 任务 3：Schema 大改

**文件：** `accounting-sql/src/schema.rs`

- [ ] **步骤 1：修改 `accounts` 表**

删除 `opened_at` 列，增加 `created_at` 和 `updated_at`：

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

- [ ] **步骤 2：修改 `transactions` 表**

`date` → `date_time`：

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

- [ ] **步骤 3：给其他 9 个表加 `created_at` 和 `updated_at`**

在 `commodities`, `account_ancestors`, `account_owners`, `members`, `channels`, `tags`, `postings`, `attachments`, `transaction_tags` 每个表的定义末尾追加：

```sql
created_at TEXT NOT NULL DEFAULT (date('now')),
updated_at TEXT NOT NULL DEFAULT (date('now'))
```

- [ ] **步骤 4：Seed Data 去时间**

accounts 的 INSERT 去掉 `opened_at` 列：

```sql
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('Equity:OpeningBalances', 3, NULL, 1),
('Income:Uncategorized', 4, NULL, 1),
('Expenses:Uncategorized', 5, NULL, 1),
('Expenses:Fees', 5, NULL, 1),
('Expenses:Discounts', 5, NULL, 1),
('Expenses:InstallmentFees', 5, NULL, 1),
('Assets:Cashback', 1, NULL, 1);
```

- [ ] **步骤 5：添加 11 个触发器**

在 `SCHEMA_SQL` 末尾追加：

```sql
CREATE TRIGGER IF NOT EXISTS update_commodities_updated_at
AFTER UPDATE ON commodities
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE commodities SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_accounts_updated_at
AFTER UPDATE ON accounts
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE accounts SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_account_ancestors_updated_at
AFTER UPDATE ON account_ancestors
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE account_ancestors SET updated_at = date('now')
    WHERE account_id = NEW.account_id AND ancestor_id = NEW.ancestor_id;
END;

CREATE TRIGGER IF NOT EXISTS update_account_owners_updated_at
AFTER UPDATE ON account_owners
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE account_owners SET updated_at = date('now')
    WHERE account_id = NEW.account_id AND member_id = NEW.member_id;
END;

CREATE TRIGGER IF NOT EXISTS update_members_updated_at
AFTER UPDATE ON members
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE members SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_channels_updated_at
AFTER UPDATE ON channels
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE channels SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_tags_updated_at
AFTER UPDATE ON tags
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE tags SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_transactions_updated_at
AFTER UPDATE ON transactions
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE transactions SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_postings_updated_at
AFTER UPDATE ON postings
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE postings SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_attachments_updated_at
AFTER UPDATE ON attachments
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE attachments SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_transaction_tags_updated_at
AFTER UPDATE ON transaction_tags
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE transaction_tags SET updated_at = date('now')
    WHERE transaction_id = NEW.transaction_id AND tag_id = NEW.tag_id;
END;
```

- [ ] **步骤 6：验证**

```bash
cargo test -p accounting-sql
```

---

## 阶段 3：Account Repo

### 任务 4：`accounting-sql/src/repo/account.rs`

- [ ] **步骤 1：INSERT 去 `opened_at`**

```rust
conn.execute(
    "INSERT INTO accounts
     (full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    params![
        account.full_name,
        account.account_type as i32,
        account.parent_id.map(|id| id.0),
        account.closed_at.map(|d| d.to_string()),
        account.is_system as i32,
        account.billing_day,
        account.repayment_day,
    ],
)?;
```

- [ ] **步骤 2：SELECT 去 `opened_at`**

所有 SELECT 语句的列列表去掉 `opened_at`：

```sql
SELECT id, full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day
```

- [ ] **步骤 3：`close` 去参数**

```rust
fn close(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError> {
    conn.execute(
        "UPDATE accounts SET closed_at = date('now') WHERE id = ?1",
        params![id.0],
    )?;
    Ok(())
}
```

- [ ] **步骤 4：`map_account` 删除 `opened_at` 解析**

删除 `opened_str` 和 `opened_at` 的解析代码。列索引相应调整（`closed_at` 从索引 5 变为 4，`is_system` 从 6 变为 5，以此类推）。

- [ ] **步骤 5：同步测试**

测试中所有 `Account` 构造删除 `opened_at`。`test_close_and_reopen` 中 `repo.close(&conn, found.id)` 不再传日期。

- [ ] **步骤 6：验证**

```bash
cargo test -p accounting-sql
```

---

## 阶段 4：Transaction Repo

### 任务 5：`accounting-sql/src/repo/transaction.rs`

- [ ] **步骤 1：INSERT `date` → `date_time`**

```rust
conn.execute(
    "INSERT INTO transactions (date_time, description, member_id, is_template)
     VALUES (?1, ?2, ?3, ?4)",
    params![
        tx.date_time.to_string(),
        tx.description,
        tx.member_id.map(|id| id.0),
        tx.is_template as i32,
    ],
)?;
```

- [ ] **步骤 2：UPDATE `date` → `date_time`**

```sql
UPDATE transactions
SET date_time = ?1, description = ?2, member_id = ?3, is_template = ?4
WHERE id = ?5
```

- [ ] **步骤 3：SELECT `date` → `date_time`**

所有 SELECT 中的 `transactions.date` 改为 `transactions.date_time`。

- [ ] **步骤 4：过滤条件适配**

`list` 和 `count` 中的日期过滤条件改为用 `DATE(transactions.date_time)`：

```sql
DATE(transactions.date_time) >= ?
DATE(transactions.date_time) <= ?
```

ORDER BY 也要改：

```sql
ORDER BY transactions.date_time DESC, transactions.id DESC
```

- [ ] **步骤 5：`map_transaction` 解析 `date_time`**

```rust
fn map_transaction(row: &rusqlite::Row) -> Result<Transaction, rusqlite::Error> {
    let date_time_str: String = row.get(1)?;
    let date_time = NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            1, rusqlite::types::Type::Text, Box::new(e)
        ))?;

    Ok(Transaction {
        id: TransactionId(row.get(0)?),
        date_time,
        description: row.get(2)?,
        member_id: row.get::<_, Option<i64>>(3)?.map(accounting::id::MemberId),
        is_template: row.get::<_, i32>(4)? != 0,
    })
}
```

- [ ] **步骤 6：同步测试**

`sample_tx` 和测试中的 `Transaction` 构造：

```rust
fn sample_tx() -> Transaction {
    Transaction {
        id: TransactionId(0),
        date_time: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
            .and_hms_opt(10, 30, 0).unwrap(),
        description: "Grocery shopping".to_string(),
        member_id: None,
        is_template: false,
    }
}
```

日期过滤测试同步调整。

- [ ] **步骤 7：验证**

```bash
cargo test -p accounting-sql
```

---

## 阶段 5：其他 Repo

### 任务 6：其他 Repo 去时间字段

**文件：**
- `accounting-sql/src/repo/member.rs`
- `accounting-sql/src/repo/commodity.rs`
- `accounting-sql/src/repo/tag.rs`
- `accounting-sql/src/repo/channel.rs`
- `accounting-sql/src/repo/posting.rs`
- `accounting-sql/src/repo/attachment.rs`

- [ ] **步骤 1：逐个修改 INSERT 语句**

每个 repo 的 INSERT 去掉 `created_at` / `updated_at` 列（当前 Schema 中这些表还没有这两个列，但应用层的 INSERT 也不能传）。

实际上，这些 repo 的 INSERT 当前**没有**传时间字段（因为 schema 中原本就没有）。但新 schema 加了 `DEFAULT`，所以不需要改动 INSERT。只需确认它们没有显式插入时间字段即可。

**验证：** 这些 repo 的 INSERT 当前只插入业务字段，无需修改。

---

## 检查点 1

```bash
cargo test -p accounting-sql
```

---

## 阶段 6：Service 层

### 任务 7：`accounting-service/src/account_service.rs`

- [ ] **步骤 1：`close` 删除 `closed_at` 参数**

```rust
pub async fn close(&self, id: AccountId) -> Result<(), AccountingError> {
```

内部调用：`tx.account_repo().close(&tx.conn(), id)`

- [ ] **步骤 2：同步测试**

```rust
service.close(id).await.unwrap();
```

`sample_account` 删除 `opened_at`。

- [ ] **步骤 3：验证**

```bash
cargo test -p accounting-service
```

---

### 任务 8：`accounting-service/src/transaction_service.rs`

- [ ] **步骤 1：同步测试**

`sample_account` 删除 `opened_at`。
所有 `Transaction` 构造：`date` → `date_time`。

- [ ] **步骤 2：验证**

```bash
cargo test -p accounting-service
```

---

## 检查点 2

```bash
cargo test -p accounting-service
```

---

## 阶段 7：CLI

### 任务 9：`accounting-cli/src/cmd/account.rs`

- [ ] **步骤 1：`account add` 去 `opened_at`**

```rust
let account = Account {
    id: AccountId(0),
    full_name: args.full_name,
    account_type: args.r#type.into(),
    parent_id: args.parent.map(AccountId),
    closed_at: None,
    is_system: false,
    billing_day: args.billing_day,
    repayment_day: args.repayment_day,
};
```

- [ ] **步骤 2：`account close` 去日期参数**

```rust
#[derive(Args)]
pub struct AccountCloseArgs {
    pub id: i64,
}
```

调用：`service.close(AccountId(args.id)).await?`

- [ ] **步骤 3：验证编译**

```bash
cargo check -p accounting-cli
```

---

### 任务 10：`accounting-cli/src/cmd/tx.rs`

- [ ] **步骤 1：`parse_date` → `parse_date_time`**

```rust
fn parse_date_time(s: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .map_err(|_| AccountingError::InvalidDate(
            format!("时间格式应为 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS: {}", s)
        ))
}
```

- [ ] **步骤 2：替换所有 `parse_date` 调用为 `parse_date_time`**

`parse_tx_args` 和 `parse_tx_args_for_update` 中使用 `parse_date_time`。
`Transaction` 构造：`date_time: parse_date_time(&args.date)?`

- [ ] **步骤 3：`build_filter` 适配**

`build_filter` 中仍然使用 `parse_date_time`，但 `TransactionFilter` 保持 `start_date: Option<NaiveDate>`。需要将 `NaiveDateTime` 转为 `NaiveDate`：

```rust
if let Some(ref from) = args.from {
    filter.start_date = Some(parse_date_time(from)?.date());
}
```

- [ ] **步骤 4：验证编译**

```bash
cargo check -p accounting-cli
```

---

### 任务 11：`accounting-cli/src/cmd/mod.rs`

- [ ] **步骤 1：`AccountRow` 去 `opened_at`**

```rust
#[derive(Tabled, Serialize)]
pub struct AccountRow {
    pub id: i64,
    pub full_name: String,
    pub account_type: String,
    pub parent_id: String,
    pub closed_at: String,
    pub is_system: bool,
}
```

`From<&Account>` 同步调整。

- [ ] **步骤 2：`TransactionRow` `date` → `date_time`**

```rust
#[derive(Tabled, Serialize)]
pub struct TransactionRow {
    pub id: i64,
    pub date_time: String,
    pub description: String,
    pub member_id: String,
    pub is_template: bool,
}
```

`From<&Transaction>`：`date_time: t.date_time.to_string()`

- [ ] **步骤 3：验证编译**

```bash
cargo check -p accounting-cli
```

---

## 阶段 8：其他实现和测试

### 任务 12：`accounting-sql/src/impls/sqlite.rs`

- [ ] **步骤 1：删除 `opened_at`**

找到文件中所有 `opened_at` 的硬编码引用并删除。

### 任务 13：`accounting-service/src/report_service.rs`

- [ ] **步骤 1：删除 `opened_at`**

同样处理测试中的硬编码 `opened_at`。

### 任务 14：触发器测试

**文件：** `accounting-sql/src/schema.rs`（tests 模块）

- [ ] **步骤 1：增加触发器验证测试**

```rust
#[test]
fn test_updated_at_trigger() {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    insert_seed_data(&conn).unwrap();

    let before: String = conn
        .query_row("SELECT updated_at FROM accounts WHERE id = 1", [], |row| row.get(0))
        .unwrap();

    conn.execute("UPDATE accounts SET full_name = full_name || 'X' WHERE id = 1", [])
        .unwrap();

    let after: String = conn
        .query_row("SELECT updated_at FROM accounts WHERE id = 1", [], |row| row.get(0))
        .unwrap();

    assert_ne!(before, after);
}
```

---

## 最终检查点

```bash
cargo test --workspace
cargo build --release --workspace
```

---

## 风险与回退

| 风险 | 缓解措施 |
|---|---|
| SQLite 版本不支持 `DEFAULT` 表达式 | `date('now')` 在 SQLite 3.1+ 即支持，项目用 bundled，安全 |
| 触发器递归 | `WHEN OLD.updated_at = NEW.updated_at` 条件已验证可防止递归 |
| 列索引偏移导致 `map_account`/`map_transaction` 解析错误 | 仔细核对 SELECT 列顺序与 `row.get(index)` 索引 |
| `NaiveDateTime` 解析格式不匹配 | 统一用 `%Y-%m-%d %H:%M:%S`，测试覆盖两种 CLI 输入格式 |
| 遗漏文件 | 最终检查点 `cargo test --workspace` 会暴露所有编译错误 |

**回退策略：** 每个阶段结束后可独立 `git commit`，若最终测试失败可逐阶段回滚。
