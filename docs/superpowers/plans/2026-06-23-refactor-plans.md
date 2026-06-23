# 数据模型精简重构实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 实现 4 份数据模型精简重构计划（移除冗余字段、审计字段改进、账户名称重构、账户类型重构），并将 main 分支的 `fcb8fea6` 迁移到当前分支。

**架构：** 保持现有分层架构不变（accounting / accounting-sql / accounting-service / accounting-api / accounting-cli / accounting-web），通过逐步修改各层数据模型、Repo、Service、API、CLI 和 Web 代码完成重构。每个阶段独立验证，确保 `cargo fmt && cargo clippy && cargo test` 全绿后再提交。

**技术栈：** Rust（tokio、rusqlite、serde、rust_decimal）、SQLite、Vite/Vue3（前端）。

---

## 文件结构与职责

### accounting crate（核心域模型）

| 文件 | 职责 |
|------|------|
| `accounting/src/account.rs` | `Account` 结构体（将删除 `full_name`、`account_type`、`position`） |
| `accounting/src/account_type.rs` | `AccountType` 枚举（将删除 `from_prefix()` 等方法，只保留 `FromStr`） |
| `accounting/src/posting.rs` | `Posting` 结构体（将删除 `member_id`、`channel_id`） |
| `accounting/src/transaction.rs` | `Transaction` 结构体（将删除 `is_template`） |
| `accounting/src/transaction_filter.rs` | `TransactionFilter`（将删除 `is_template` 字段） |
| `accounting/src/closure.rs` | `AccountNode`（将 `full_name` 改为 `name`） |
| `accounting/src/validation.rs` | 关户验证（参数将从 `AccountType` 改为根节点信息） |

### accounting-sql crate（数据持久化）

| 文件 | 职责 |
|------|------|
| `accounting-sql/src/schema.rs` | SQLite DDL 与种子数据（所有表结构变更集中在此） |
| `accounting-sql/src/repo/account.rs` | `AccountRepo` trait 与实现（删除 `account_type` 映射，`full_name` → `name`） |
| `accounting-sql/src/repo/posting.rs` | `PostingRepo` trait 与实现（删除 `member_id`、`channel_id`，报表 SQL 调整） |
| `accounting-sql/src/repo/transaction.rs` | `TransactionRepo` trait 与实现（删除 `is_template`） |

### accounting-service crate（业务逻辑）

| 文件 | 职责 |
|------|------|
| `accounting-service/src/account_service.rs` | 账户服务（`create_cascading()` 重写，关户验证调整） |
| `accounting-service/src/transaction_service.rs` | 交易服务（构造 Posting 时不再设置已删除字段） |
| `accounting-service/src/report_service.rs` | 报表服务（按根节点推导 `AccountType` 后分类） |

### accounting-api / accounting-cli / accounting-web

| 文件 | 职责 |
|------|------|
| `accounting-api/src/dto.rs` | DTO 字段调整（删除 `full_name`、`account_type`、`is_template`、`member_id`、`channel_id`、`position` 等） |
| `accounting-api/src/handlers/account.rs` | 账户 handler（接口参数调整） |
| `accounting-api/src/handlers/transaction.rs` | 交易 handler（`is_template`、分录字段调整） |
| `accounting-cli/src/cmd/account.rs` | 账户 CLI 命令（按 `name` + `parent_id` 创建/查找） |
| `accounting-cli/src/cmd/tx.rs` | 交易 CLI 命令（分录字段、 `--template` 调整） |
| `accounting-cli/src/cmd/mod.rs` | 显示字段调整 |
| `accounting-web/src/` | 前端组件（去掉 `full_name.split(':')`，直接使用 `name`） |

---

## 前置迁移：cherry-pick `fcb8fea6`

### 任务 0：将 `fcb8fea6` 迁移到当前分支

**文件：**
- 修改：`accounting/src/account.rs`、`accounting-sql/src/schema.rs`、`accounting-sql/src/repo/account.rs`、`accounting-sql/src/impls/sqlite.rs`、`accounting-service/src/account_service.rs`、`accounting-service/src/report_service.rs`、`accounting-service/src/transaction_service.rs`、`accounting-api/src/dto.rs`、`accounting-api/src/handlers/account.rs`、`accounting-web/` 多个组件

- [ ] **步骤 1：执行 cherry-pick**

运行：

```bash
cd /home/mechdancer/repos/accounting
git cherry-pick fcb8fea64c5dc25b6228b1866beb5c1826d902e2
```

- [ ] **步骤 2：解决冲突（如有）**

预期冲突文件：`accounting/src/account.rs`、`accounting-sql/src/schema.rs`、`accounting-api/src/dto.rs`。

解决原则：
- 保留 `Liability` 的删除（来自当前分支 `4b9fa057`）
- 保留 `position` 字段和拖拽排序的删除（来自 `fcb8fea6`）
- 最终 `Account` 不应包含 `position` 字段

- [ ] **步骤 3：验证编译与测试**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

- [ ] **步骤 4：提交 cherry-pick 结果**

```bash
git add -A
git commit -m "refactor: migrate main commit fcb8fea6 - remove account position and drag reorder"
```

---

## 阶段 1：移除冗余字段

### 任务 1.1：删除 `Posting` 的 `member_id` 和 `channel_id`

**文件：**
- 修改：`accounting/src/posting.rs`
- 修改：`accounting-sql/src/schema.rs`
- 修改：`accounting-sql/src/repo/posting.rs`
- 修改：`accounting-service/src/transaction_service.rs`
- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/transaction.rs`
- 修改：`accounting-cli/src/cmd/tx.rs`

- [ ] **步骤 1：修改 `accounting/src/posting.rs`**

删除字段：

```rust
pub member_id: Option<MemberId>,
pub channel_id: Option<ChannelId>,
```

同时更新该文件中的测试/示例构造（如有）。

- [ ] **步骤 2：修改 `accounting-sql/src/schema.rs`**

在 `postings` 表定义中删除：

```sql
member_id INTEGER REFERENCES members(id),
channel_id INTEGER REFERENCES channels(id),
```

- [ ] **步骤 3：修改 `accounting-sql/src/repo/posting.rs`**

1. 在 `insert()` 的 SQL 中删除 `member_id, channel_id` 列及对应参数。
2. 在 `get()`、`list_by_transaction()`、`list_by_account()` 的 SELECT 和 `Posting` 构造中删除 `member_id`、`channel_id` 映射。
3. 更新 `sample_posting()` 测试辅助函数。
4. 更新测试中直接插入分录的 SQL（如 `INSERT INTO postings ...`），移除已删除列。

- [ ] **步骤 4：修改 `accounting-service/src/transaction_service.rs`**

删除测试中 `sample_posting()` 的 `member_id` 和 `channel_id` 字段。

- [ ] **步骤 5：修改 `accounting-api/src/dto.rs`**

`PostingDto` 和 `PostingRequest` 中本无这两个字段，检查确认无需改动。

- [ ] **步骤 6：修改 `accounting-api/src/handlers/transaction.rs`**

在构造 `Posting` 时不再设置 `member_id` 和 `channel_id`。

- [ ] **步骤 7：修改 `accounting-cli/src/cmd/tx.rs`**

移除从命令行读取 posting 的 `member_id` 和 `channel_id` 的代码。

- [ ] **步骤 8：运行验证**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

- [ ] **步骤 9：提交**

```bash
git add -A
git commit -m "refactor: remove posting.member_id and posting.channel_id"
```

---

### 任务 1.2：将 `sum_by_channel()` 改为按 transaction 级别渠道分组

**文件：**
- 修改：`accounting-sql/src/repo/posting.rs`

- [ ] **步骤 1：修改 `sum_by_channel()` 的 WHERE 条件**

将：

```rust
let mut conditions = vec!["1=1", "p.channel_id IS NOT NULL"];
```

改为：

```rust
let mut conditions = vec!["1=1", "t.channel_id IS NOT NULL"];
```

- [ ] **步骤 2：修改渠道过滤条件**

将：

```rust
if let Some(channel) = filter.channel_id {
    conditions.push("p.channel_id = ?");
    params_vec.push(Box::new(channel.0));
}
```

改为：

```rust
if let Some(channel) = filter.channel_id {
    conditions.push("t.channel_id = ?");
    params_vec.push(Box::new(channel.0));
}
```

- [ ] **步骤 3：修改 SELECT 和 GROUP BY**

将：

```sql
SELECT p.channel_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
```

改为：

```sql
SELECT t.channel_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
```

同时 `GROUP BY p.channel_id, p.commodity_id, a.account_type` 改为 `GROUP BY t.channel_id, p.commodity_id, a.account_type`。

- [ ] **步骤 4：更新测试 `test_sum_by_channel()`**

在 `sample_posting` 中设置 `channel_id` 的代码需要改为在 `Transaction` 级别设置渠道。由于 `Posting` 不再有 `channel_id`，测试中应直接通过 SQL 插入 transaction 时指定 `channel_id`，并移除 `p1.channel_id = Some(channel_id)` 等代码。

- [ ] **步骤 5：运行验证**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

- [ ] **步骤 6：提交**

```bash
git add -A
git commit -m "refactor: update sum_by_channel to group by transaction channel_id"
```

---

### 任务 1.3：删除 `Transaction` 的 `is_template`

**文件：**
- 修改：`accounting/src/transaction.rs`
- 修改：`accounting/src/transaction_filter.rs`
- 修改：`accounting-sql/src/schema.rs`
- 修改：`accounting-sql/src/repo/transaction.rs`
- 修改：`accounting-sql/src/repo/posting.rs`
- 修改：`accounting-service/src/transaction_service.rs`
- 修改：`accounting-cli/src/cmd/tx.rs`
- 修改：`accounting-cli/src/cmd/mod.rs`
- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/transaction.rs`
- 修改：`accounting-web/src/stores/transaction.ts`

- [ ] **步骤 1：修改 `accounting/src/transaction.rs`**

删除字段：

```rust
pub is_template: bool,
```

- [ ] **步骤 2：修改 `accounting/src/transaction_filter.rs`**

删除字段：

```rust
pub is_template: Option<bool>,
```

- [ ] **步骤 3：修改 `accounting-sql/src/schema.rs`**

在 `transactions` 表定义中删除：

```sql
is_template INTEGER NOT NULL DEFAULT 0,
```

- [ ] **步骤 4：修改 `accounting-sql/src/repo/transaction.rs`**

1. 在 `insert()` 的 SQL 中删除 `is_template` 列及参数。
2. 在 `get()` 的 SELECT 和 `map_transaction()` 中删除 `is_template` 映射。
3. 在 `list()` 和 `count()` 中删除 `is_template` 过滤条件。
4. 在 `update()` 的 SQL 中删除 `is_template` 列及参数。
5. 更新 `sample_tx()` 和测试中构造 `Transaction` 的代码。
6. 更新测试中直接插入 `transactions` 的 SQL，移除 `is_template` 列。

- [ ] **步骤 5：修改 `accounting-sql/src/repo/posting.rs`**

在 `sum_by_tag()`、`sum_by_member()`、`sum_by_channel()` 中删除 `is_template` 过滤条件及参数。

- [ ] **步骤 6：修改 `accounting-service/src/transaction_service.rs`**

更新测试中构造 `Transaction` 的代码，删除 `is_template` 字段。

- [ ] **步骤 7：修改 `accounting-cli/src/cmd/tx.rs` 和 `mod.rs`**

移除 `--template` 参数及显示字段。

- [ ] **步骤 8：修改 `accounting-api/src/dto.rs`**

从 `TransactionDto` 中删除 `is_template` 字段。

- [ ] **步骤 9：修改 `accounting-api/src/handlers/transaction.rs`**

移除 `is_template` 相关处理。

- [ ] **步骤 10：修改 `accounting-web/src/stores/transaction.ts`**

移除 `is_template` 字段。

- [ ] **步骤 11：运行验证**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

- [ ] **步骤 12：提交**

```bash
git add -A
git commit -m "refactor: remove transaction.is_template"
```

---

### 任务 1.4：删除 `AccountType::is_permanent()`

**文件：**
- 修改：`accounting/src/account_type.rs`

- [ ] **步骤 1：检查当前代码**

当前 `accounting/src/account_type.rs` 中已无 `is_permanent()` 方法（可能已在之前的提交中删除）。确认无此方法后，本任务无需改动。

- [ ] **步骤 2：如存在则删除**

如存在，删除方法及对应测试。

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
git add -A
git commit -m "refactor: remove AccountType::is_permanent()"
```

---

## 阶段 2：审计字段改进

### 任务 2.1：将审计字段精度从 `date('now')` 改为 `datetime('now')`

**文件：**
- 修改：`accounting-sql/src/schema.rs`

- [ ] **步骤 1：替换所有表默认值**

将 `SCHEMA_SQL` 中所有：

```sql
created_at TEXT NOT NULL DEFAULT (date('now')),
updated_at TEXT NOT NULL DEFAULT (date('now'))
```

替换为：

```sql
created_at TEXT NOT NULL DEFAULT (datetime('now')),
updated_at TEXT NOT NULL DEFAULT (datetime('now'))
```

涉及表：`commodities`、`accounts`、`account_ancestors`、`account_owners`、`members`、`channels`、`tags`、`transactions`、`postings`、`attachments`、`transaction_tags`。

- [ ] **步骤 2：替换所有触发器中的 `date('now')`**

将所有 `update_*_updated_at` 触发器中的：

```sql
UPDATE ... SET updated_at = date('now') WHERE ...
```

替换为：

```sql
UPDATE ... SET updated_at = datetime('now') WHERE ...
```

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
git add -A
git commit -m "refactor: change audit fields precision from date to datetime"
```

---

### 任务 2.2：给 `settings` 表补充审计字段和触发器

**文件：**
- 修改：`accounting-sql/src/schema.rs`

- [ ] **步骤 1：修改 `settings` 表定义**

将：

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

改为：

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- [ ] **步骤 2：添加 settings 触发器**

在 `SCHEMA_SQL` 末尾追加：

```sql
CREATE TRIGGER IF NOT EXISTS update_settings_updated_at
AFTER UPDATE ON settings
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE key = NEW.key;
END;
```

- [ ] **步骤 3：更新 `test_audit_columns_exist()`**

将 `tables` 数组追加 `"settings"`。

- [ ] **步骤 4：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
git add -A
git commit -m "refactor: add audit columns to settings table"
```

---

## 阶段 3：账户名称重构（`full_name` → `name`）

### 任务 3.1：修改核心域模型 `Account` 和 `AccountNode`

**文件：**
- 修改：`accounting/src/account.rs`
- 修改：`accounting/src/closure.rs`

- [ ] **步骤 1：修改 `accounting/src/account.rs`**

将：

```rust
pub full_name: String,
```

改为：

```rust
pub name: String,
```

并更新该文件中的测试构造：

```rust
let a = Account {
    id: AccountId(1),
    name: "Cash".to_string(),
    // ...
};
```

- [ ] **步骤 2：修改 `accounting/src/closure.rs`**

将：

```rust
pub full_name: String,
```

改为：

```rust
pub name: String,
```

并更新测试。

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting
git add -A
git commit -m "refactor: rename Account.full_name to name and AccountNode.full_name to name"
```

---

### 任务 3.2：修改 SQLite Schema 和种子数据

**文件：**
- 修改：`accounting-sql/src/schema.rs`

- [ ] **步骤 1：修改 `accounts` 表定义**

将：

```sql
full_name TEXT NOT NULL UNIQUE,
```

改为：

```sql
name TEXT NOT NULL,
```

并在表定义末尾追加：

```sql
UNIQUE(parent_id, name)
```

- [ ] **步骤 2：修改种子数据**

将 `SEED_ACCOUNTS_ROOT_EN` 和 `SEED_ACCOUNTS_ROOT_ZH` 中的 `full_name` 改为 `name`，并移除 `account_type` 列（为阶段 4 做准备，但阶段 3 可以先不改 `account_type`，等阶段 4 再处理）。

**注意：** 阶段 3 只改 `full_name` → `name`，不改 `account_type`。种子数据改为：

```sql
INSERT OR IGNORE INTO accounts (name, account_type, parent_id, is_system) VALUES
('Assets', 1, NULL, 1),
('Equity', 2, NULL, 1),
('Income', 3, NULL, 1),
('Expenses', 4, NULL, 1);
```

子账户种子数据改为使用 `name` 并通过 `parent_id` 关联：

```sql
INSERT OR IGNORE INTO accounts (name, account_type, parent_id, is_system) VALUES
('OpeningBalances', 2, (SELECT id FROM accounts WHERE name = 'Equity'), 1),
('Fees', 4, (SELECT id FROM accounts WHERE name = 'Expenses'), 1),
...
```

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-sql
git add -A
git commit -m "refactor: update schema and seed data for name + parent_id unique constraint"
```

---

### 任务 3.3：修改 `AccountRepo` 从 `full_name` 到 `name`

**文件：**
- 修改：`accounting-sql/src/repo/account.rs`

- [ ] **步骤 1：更新所有 SELECT 列**

将所有 SELECT 中的 `full_name` 改为 `name`。

- [ ] **步骤 2：更新 `create()` 的 SQL**

将 INSERT 列中的 `full_name` 改为 `name`，参数从 `account.full_name` 改为 `account.name`。

- [ ] **步骤 3：重写 `get_by_name()`**

原方法按完整路径名查找。改为按**逐级查找**实现：

```rust
fn get_by_name(
    &self,
    conn: &Connection,
    name: &str,
) -> Result<Option<Account>, crate::error::DbError> {
    let segments: Vec<&str> = name.split(':').collect();
    let mut parent_id: Option<i64> = None;
    for segment in segments {
        let mut stmt = conn.prepare(
            "SELECT id, name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day, position
             FROM accounts WHERE name = ?1 AND parent_id IS ?2"
        )?;
        let mut rows = stmt.query(params![segment, parent_id])?;
        if let Some(row) = rows.next()? {
            parent_id = Some(row.get::<_, i64>(0)?);
        } else {
            return Ok(None);
        }
    }
    parent_id.map(|id| self.get(conn, AccountId(id))).transpose()
}
```

- [ ] **步骤 4：更新 `map_account()`**

将 `full_name: row.get(1)?` 改为 `name: row.get(1)?`。

- [ ] **步骤 5：更新测试**

测试中构造 `Account` 的 `full_name` 改为 `name`，如 `name: "Bank".to_string()`。测试中调用 `get_by_name("Equity:OpeningBalances")` 仍应工作。

- [ ] **步骤 6：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-sql
git add -A
git commit -m "refactor: update AccountRepo to use name instead of full_name"
```

---

### 任务 3.4：修改 `accounting-service` 中的账户服务

**文件：**
- 修改：`accounting-service/src/account_service.rs`

- [ ] **步骤 1：更新 `create()` 中的字段名**

将 `account.full_name` 改为 `account.name`。

- [ ] **步骤 2：重写 `create_cascading()`**

原方法按 `:` 切分 `full_name` 并逐级创建，每级拼接路径。改为：

1. 第一段作为根账户 `name`，从根账户 `name` 推导 `AccountType`（仍使用 `from_prefix`，阶段 4 再改为 `FromStr`）。
2. 逐级查找/创建，每级只使用本级 `name` 和 `parent_id`。
3. 叶子账户设置 `billing_day` 和 `repayment_day`。

伪代码：

```rust
pub async fn create_cascading(
    &self,
    path: &str,
    billing_day: Option<u8>,
    repayment_day: Option<u8>,
    owner_ids: &[MemberId],
) -> Result<AccountId, AccountingError> {
    let segments: Vec<&str> = path.split(':').collect();
    if segments.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            t!("account_name_empty").to_string(),
        ));
    }

    let account_type = AccountType::from_prefix(segments[0]).ok_or_else(|| {
        AccountingError::InvalidTransaction(
            t!("unrecognized_account_prefix", prefix = segments[0]).to_string(),
        )
    })?;

    let tx = self.db.transaction().await.map_err(|e| ...)?;
    let mut parent_id: Option<AccountId> = None;
    let mut last_id: Option<AccountId> = None;

    for (i, segment) in segments.iter().enumerate() {
        // 按 name + parent_id 查找
        let existing = tx.account_repo().get_by_name(&tx.conn(), segment).map_err(...)?;
        // 注意：这里需要新的 Repo 方法 get_by_name_and_parent，或者先不处理
        ...
    }
}
```

**注意：** 由于 `get_by_name()` 现在按路径查找，在级联创建中需要按 `name + parent_id` 查找。可以在 `AccountRepo` 中新增方法 `get_child_by_name(parent_id, name)`，或临时使用 `list_children()` 过滤。

推荐新增 `AccountRepo::get_by_parent_and_name(parent_id: Option<AccountId>, name: &str)` 方法，供 `create_cascading()` 使用。

- [ ] **步骤 3：更新 `AccountService` 测试**

将 `sample_account("Assets:Cash", ...)` 改为 `sample_account("Cash", ...)`，并设置 `parent_id`。

- [ ] **步骤 4：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-service
git add -A
git commit -m "refactor: update AccountService for name + parent_id model"
```

---

### 任务 3.5：修改 API DTO 和 Handlers

**文件：**
- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/account.rs`
- 修改：`accounting-api/src/handlers/transaction.rs`

- [ ] **步骤 1：修改 `AccountDto`**

将 `full_name` 改为 `name`。

- [ ] **步骤 2：修改 `CreateAccountRequest`**

将 `full_name` 改为 `name`，增加 `parent_id: Option<i64>`。

- [ ] **步骤 3：修改 `RenameAccountRequest`**

将 `full_name` 改为 `name`。

- [ ] **步骤 4：修改 `accounting-api/src/handlers/account.rs`**

在 handler 中：
- 构造 `Account` 时使用 `name` 和 `parent_id`。
- 响应 DTO 使用 `name`。
- 重命名接口只更新 `name`。

- [ ] **步骤 5：修改 `accounting-api/src/handlers/transaction.rs`**

处理 `PostingRequest.account` 时，它仍然是路径字符串，但底层 `get_by_name()` 已支持路径查找，所以无需大改。只需确保不再引用已删除的字段。

- [ ] **步骤 6：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-api
git add -A
git commit -m "refactor: update API DTOs and handlers for account name model"
```

---

### 任务 3.6：修改 CLI 账户命令

**文件：**
- 修改：`accounting-cli/src/cmd/account.rs`
- 修改：`accounting-cli/src/cmd/mod.rs`

- [ ] **步骤 1：修改 `accounting-cli/src/cmd/account.rs`**

将创建/查找账户的参数从 `full_name` 改为 `name` + `parent_id`（或仍接受路径字符串但底层调用 `get_by_name`）。

- [ ] **步骤 2：修改显示输出**

显示账户时直接输出 `name` 而不是 `full_name`。

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-cli
git add -A
git commit -m "refactor: update CLI account commands for name model"
```

---

### 任务 3.7：修改前端 Web

**文件：**
- 修改：`accounting-web/src/components/AccountPicker.vue`
- 修改：`accounting-web/src/components/AccountCards.vue`
- 修改：`accounting-web/src/components/AccountTree.vue`
- 修改：`accounting-web/src/components/TransactionDetail.vue`
- 修改：`accounting-web/src/composables/useAccountTree.ts`
- 其他引用 `full_name.split(':')` 的文件

- [ ] **步骤 1：全局替换 `full_name.split(':').pop()` 为 `name`**

- [ ] **步骤 2：全局替换 `shortName(account.full_name)` 为 `account.name`**

- [ ] **步骤 3：修改账户树构建逻辑**

`useAccountTree.ts` 中如果通过 `full_name.split(':')` 构建树，改为通过 `parent_id` 关系构建（后端返回的数据可能已包含 `parent_id`）。

- [ ] **步骤 4：运行前端验证**

```bash
cd accounting-web
npm run type-check
npm run build
```

- [ ] **步骤 5：提交**

```bash
cd /home/mechdancer/repos/accounting
git add -A
git commit -m "refactor: update web frontend to use account name instead of full_name"
```

---

### 任务 3.8：阶段 3 整体验证

- [ ] **步骤 1：运行完整验证**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
cd accounting-web && npm run type-check && npm run build
```

- [ ] **步骤 2：如全部通过，本阶段结束**

如有失败，回到对应任务修复。

---

## 阶段 4：账户类型重构（删除 `Account.account_type`）

### 任务 4.1：修改 `AccountType` 只保留 `FromStr`

**文件：**
- 修改：`accounting/src/account_type.rs`

- [ ] **步骤 1：删除 `from_prefix()` 方法**

删除：

```rust
pub fn from_prefix(prefix: &str) -> Option<Self> { ... }
```

- [ ] **步骤 2：实现 `FromStr`**

新增：

```rust
impl std::str::FromStr for AccountType {
    type Err = String;

    fn from_str(root_name: &str) -> Result<Self, Self::Err> {
        let lower = root_name.to_lowercase();
        match lower.as_str() {
            "asset" | "assets" | "资产" => Ok(Self::Asset),
            "equity" | "权益" => Ok(Self::Equity),
            "income" | "收入" => Ok(Self::Income),
            "expense" | "expenses" | "支出" => Ok(Self::Expense),
            _ => Err(format!("unknown account root name: {}", root_name)),
        }
    }
}
```

- [ ] **步骤 3：删除 `from_prefix` 测试**

删除 `test_from_prefix()` 测试。

- [ ] **步骤 4：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting
git add -A
git commit -m "refactor: replace AccountType::from_prefix with FromStr"
```

---

### 任务 4.2：从 `Account` 中删除 `account_type` 字段

**文件：**
- 修改：`accounting/src/account.rs`
- 修改：`accounting/src/closure.rs`

- [ ] **步骤 1：修改 `accounting/src/account.rs`**

删除字段：

```rust
pub account_type: AccountType,
```

并更新测试构造。

- [ ] **步骤 2：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting
git add -A
git commit -m "refactor: remove account_type field from Account"
```

---

### 任务 4.3：修改 Schema 删除 `account_type` 列和约束

**文件：**
- 修改：`accounting-sql/src/schema.rs`

- [ ] **步骤 1：修改 `accounts` 表定义**

删除：

```sql
account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 4),
```

- [ ] **步骤 2：删除索引**

删除：

```sql
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type);
```

- [ ] **步骤 3：修改种子数据**

将 `SEED_ACCOUNTS_ROOT_*` 和 `SEED_ACCOUNTS_CHILD_*` 中的 `account_type` 列删除：

```sql
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('Assets', NULL, 1),
('Equity', NULL, 1),
('Income', NULL, 1),
('Expenses', NULL, 1);
```

子账户：

```sql
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('OpeningBalances', (SELECT id FROM accounts WHERE name = 'Equity'), 1),
...
```

- [ ] **步骤 4：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-sql
git add -A
git commit -m "refactor: remove account_type column from accounts table"
```

---

### 任务 4.4：修改 `AccountRepo` 删除 `account_type` 映射

**文件：**
- 修改：`accounting-sql/src/repo/account.rs`

- [ ] **步骤 1：更新 `create()` 的 SQL**

删除 `account_type` 列及参数。

- [ ] **步骤 2：更新所有 SELECT 列**

删除 SELECT 中的 `account_type`。

- [ ] **步骤 3：重写 `map_account()`**

删除 `account_type` 映射：

```rust
fn map_account(row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
    let closed_at: Option<String> = row.get(3)?;
    let closed_at = match closed_at {
        Some(s) => Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?),
        None => None,
    };

    Ok(Account {
        id: AccountId(row.get(0)?),
        name: row.get(1)?,
        parent_id: row.get::<_, Option<i64>>(2)?.map(AccountId),
        closed_at,
        is_system: row.get::<_, i32>(4)? != 0,
        billing_day: row.get::<_, Option<i32>>(5)?.map(|v| v as u8),
        repayment_day: row.get::<_, Option<i32>>(6)?.map(|v| v as u8),
        position: row.get::<_, i64>(7)?,
    })
}
```

- [ ] **步骤 4：更新测试**

删除测试中 `account_type` 字段和断言。

- [ ] **步骤 5：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-sql
git add -A
git commit -m "refactor: remove account_type mapping in AccountRepo"
```

---

### 任务 4.5：修改 `PostingRepo` 报表 SQL 从根节点推导类型

**文件：**
- 修改：`accounting-sql/src/repo/posting.rs`

- [ ] **步骤 1：修改 `sum_by_tag()`**

将 SQL：

```sql
SELECT tt.tag_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
JOIN transaction_tags tt ON tt.transaction_id = t.id
WHERE {}
GROUP BY tt.tag_id, p.commodity_id, a.account_type
```

改为通过闭包表 JOIN 根节点：

```sql
SELECT tt.tag_id, p.commodity_id, ra.name AS root_name, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN account_ancestors aa ON a.id = aa.account_id AND aa.depth = (
    SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id
)
JOIN accounts ra ON aa.ancestor_id = ra.id
JOIN transactions t ON p.transaction_id = t.id
JOIN transaction_tags tt ON tt.transaction_id = t.id
WHERE {}
GROUP BY tt.tag_id, p.commodity_id, ra.name
```

返回类型从 `(TagId, CommodityId, i64, Decimal)` 改为 `(TagId, CommodityId, String, Decimal)`。

- [ ] **步骤 2：修改 `sum_by_member()`**

同上，SELECT 中的 `a.account_type` 改为 `ra.name`，GROUP BY 相应调整。

- [ ] **步骤 3：修改 `sum_by_channel()`**

同上。

- [ ] **步骤 4：更新测试**

测试中按 `account_type` 整数值（3/4）判断的代码改为按 `root_name` 判断。

- [ ] **步骤 5：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-sql
git add -A
git commit -m "refactor: derive account type from root name in posting aggregation SQL"
```

---

### 任务 4.6：修改 `ReportService` 按根节点推导类型

**文件：**
- 修改：`accounting-service/src/report_service.rs`

- [ ] **步骤 1：修改 `balance_sheet()` 和 `income_statement()`**

当前按 `account.account_type` 分类。改为先通过根节点名称推导 `AccountType`：

```rust
let account_type = AccountType::from_str(&root_name).unwrap_or(AccountType::Asset);
```

其中 `root_name` 通过查询闭包表获得（可在 Repo 层新增 `find_root_name(account_id)` 方法，或在 Service 层直接查询）。

- [ ] **步骤 2：修改 `stats_by_tag()` / `stats_by_member()` / `stats_by_channel()`**

`sum_by_tag()` 返回的 `account_type` 现在是 `String`（根节点名），需要改为：

```rust
let account_type = AccountType::from_str(&root_name).unwrap_or(AccountType::Asset);
match account_type {
    AccountType::Income => entry.0.push((commodity_id, amount)),
    AccountType::Expense => entry.1.push((commodity_id, amount)),
    _ => {}
}
```

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-service
git add -A
git commit -m "refactor: update ReportService to derive AccountType from root name"
```

---

### 任务 4.7：修改 `AccountService` 删除类型校验

**文件：**
- 修改：`accounting-service/src/account_service.rs`

- [ ] **步骤 1：重写 `create_cascading()`**

现在 `Account` 没有 `account_type` 字段，但级联创建仍需要知道根节点类型以校验路径首段。可以通过 `AccountType::from_str(segments[0])` 推导。

由于类型不再存储，创建过程中无需再校验"已存在节点的 account_type 是否一致"。但仍需校验路径首段是否有效。

重写后的逻辑：

```rust
pub async fn create_cascading(
    &self,
    path: &str,
    billing_day: Option<u8>,
    repayment_day: Option<u8>,
    owner_ids: &[MemberId],
) -> Result<AccountId, AccountingError> {
    let segments: Vec<&str> = path.split(':').collect();
    if segments.is_empty() { ... }

    // 校验根节点名称有效（阶段 4 使用 FromStr）
    let _ = AccountType::from_str(segments[0]).map_err(|_| ...)?;

    // 逐级查找/创建
    ...
}
```

- [ ] **步骤 2：修改 `close()` 中的关户验证**

当前：

```rust
validate_account_close(account.account_type, &balances)?;
```

改为先查找根节点名称，再推导 `AccountType`：

```rust
let root_name = tx.account_repo().find_root_name(account.id)?;
let account_type = AccountType::from_str(&root_name)?;
validate_account_close(account_type, &balances)?;
```

如果 `AccountRepo` 没有 `find_root_name` 方法，需要新增：

```rust
fn find_root_name(&self, conn: &Connection, id: AccountId) -> Result<String, DbError>;
```

实现：

```rust
fn find_root_name(&self, conn: &Connection, id: AccountId) -> Result<String, DbError> {
    let name: String = conn.query_row(
        "SELECT a.name FROM accounts a
         JOIN account_ancestors aa ON a.id = aa.ancestor_id
         WHERE aa.account_id = ?1
         ORDER BY aa.depth DESC
         LIMIT 1",
        params![id.0],
        |row| row.get(0),
    )?;
    Ok(name)
}
```

- [ ] **步骤 3：修改 `list()` 中的按类型过滤**

当前按 `a.account_type == ty` 过滤。改为：

1. 列出所有账户。
2. 对每个账户找到根节点名称并推导 `AccountType`。
3. 按类型过滤。

- [ ] **步骤 4：更新测试**

删除测试中 `account_type` 字段。

- [ ] **步骤 5：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-service
git add -A
git commit -m "refactor: update AccountService to derive account type from root"
```

---

### 任务 4.8：修改 `validation.rs` 关户验证参数

**文件：**
- 修改：`accounting/src/validation.rs`

- [ ] **步骤 1：检查当前签名**

当前 `validate_account_close(account_type: AccountType, balances: &[(CommodityId, Decimal)])` 保持不变，因为调用方已改为推导 `AccountType` 后传入。

- [ ] **步骤 2：如需要，调整签名**

如果希望 `validation.rs` 完全不知道 `AccountType`，可以改为接收根节点名称或关闭条件字符串，但当前设计保持 `AccountType` 作为领域概念，所以无需改动。

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting
git add -A
git commit -m "refactor: keep validate_account_close using AccountType from caller"
```

---

### 任务 4.9：修改 API DTO 和 Handlers

**文件：**
- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/account.rs`

- [ ] **步骤 1：修改 `AccountDto`**

`account_type` 字段保留，但改为从根节点名称动态推导：

```rust
pub account_type: String,
```

handler 中：

```rust
let root_name = ...; // 查询根节点名称
let account_type = AccountType::from_str(&root_name)
    .map(|t| t.display_name())
    .unwrap_or_default();
```

- [ ] **步骤 2：修改 `CreateAccountRequest`**

不再需要 `account_type`，因为由 `name` + `parent_id` 决定（或在级联创建时由路径首段推导）。

- [ ] **步骤 3：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-api
git add -A
git commit -m "refactor: update API DTOs to derive account_type from root name"
```

---

### 任务 4.10：修改 CLI `--type` 过滤

**文件：**
- 修改：`accounting-cli/src/cmd/account.rs`

- [ ] **步骤 1：将 `--type` 改为按根节点 ID 过滤**

原 `--type` 接收 `AccountType` 字符串，过滤 `account_type` 字段。改为接收根节点 ID，过滤"账户是否属于该根节点子树"：

```sql
SELECT ... FROM accounts a
WHERE a.id IN (
    SELECT account_id FROM account_ancestors WHERE ancestor_id = ?
)
```

或在 Service 层列出所有账户后按根节点子树过滤。

- [ ] **步骤 2：运行验证并提交**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p accounting-cli
git add -A
git commit -m "refactor: update CLI account --type filter to use root account"
```

---

### 任务 4.11：阶段 4 整体验证

- [ ] **步骤 1：运行完整验证**

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
cd accounting-web && npm run type-check && npm run build
```

- [ ] **步骤 2：如全部通过，本阶段结束**

---

## 全局收尾

### 任务 5：最终全量验证

- [ ] **步骤 1：运行完整验证命令**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
cd accounting-web
npm run type-check
npm run build
```

- [ ] **步骤 2：检查 git 状态**

```bash
cd /home/mechdancer/repos/accounting
git status
git log --oneline -10
```

- [ ] **步骤 3：确认无未提交改动**

---

## 计划自检

### 1. 规格覆盖度

对照设计文档 `docs/superpowers/specs/2026-06-23-refactor-plans-design.md`：

| 设计章节 | 对应任务 |
|---------|---------|
| 前置迁移 `fcb8fea6` | 任务 0 |
| 阶段 1：移除冗余字段 | 任务 1.1 - 1.4 |
| 阶段 2：审计字段改进 | 任务 2.1 - 2.2 |
| 阶段 3：账户名称重构 | 任务 3.1 - 3.8 |
| 阶段 4：账户类型重构 | 任务 4.1 - 4.11 |
| 验证策略 | 每个任务末尾和任务 5 |

无遗漏。

### 2. 占位符扫描

检查以下红旗：
- "待定" / "TODO" / "后续实现" / "补充细节"：无
- "添加适当的错误处理" / "添加验证" / "处理边界情况"：无
- "为上述代码编写测试"（无实际测试代码）：无
- "类似任务 N"：无
- 只描述做什么而不展示代码的步骤：已尽量避免，所有关键变更都有代码示例
- 引用了未定义的类型/函数：`find_root_name` 在任务 4.7 中定义，之前未使用；`get_by_parent_and_name` 在任务 3.4 中提到但需实现。

### 3. 类型一致性

- `Account.name` 在任务 3.1 定义，后续任务一致使用 `name`。
- `AccountType::from_str` 在任务 4.1 定义，任务 4.6/4.7/4.9 一致使用。
- `find_root_name` 返回 `String`，与 `AccountType::from_str` 输入一致。

---

## 执行交接

**计划已完成并保存到 `docs/superpowers/plans/2026-06-23-refactor-plans.md`。两种执行方式：**

**1. 子代理驱动（推荐）** - 每个任务调度一个新的子代理，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 `executing-plans` 执行任务，批量执行并设有检查点

**选哪种方式？**
