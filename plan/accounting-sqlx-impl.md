# accounting-sql 迁移到 sqlx 0.9 实施计划

## 状态跟踪

- [x] Step 1: 依赖替换
- [x] Step 2: error.rs 重构
- [x] Step 3: 删除旧 trait 和 pool.rs
- [x] Step 4: 重写 schema.rs（async）
- [x] Step 5: 重写 database.rs → SqliteDatabase
- [x] Step 6: 重写 transaction.rs → SqliteTransaction
- [x] Step 7a-h: 重写 8 个 repo 模块
- [x] Step 8: 新增 API 层缺失的 repo 方法
- [x] Step 9: Service 层适配
- [x] Step 10: API 层适配
- [x] Step 11: CLI 层适配
- [x] Step 12: 测试适配
- [x] Step 13: lib.rs 更新
- [x] Step 14: 验证

## 核心决策

1. **移除所有 trait 抽象**（Database、Transaction、8 个 Repo trait），直接暴露 sqlx 类型
2. **使用 sqlx 0.9.0**，repo 函数接受 `&mut SqliteConnection`，Pool 通过 `pool.acquire().await` 获取连接后 Deref，Transaction 通过 `&mut *tx` Deref
3. **两阶段实施**：Phase 1 完成 rusqlite → sqlx::Sqlite（含 trait 移除 + async 化），Phase 2 添加 PostgreSQL feature gate
4. **动态 SQL** 使用 `sqlx::QueryBuilder`
5. **行映射** 采用混合策略（简单查询用 `FromRow` derive，复杂逻辑用手动 `row.get()`）

## sqlx 0.9 关键 API

- `&Pool<DB>` 仍实现 `Executor`，可直接 `query(...).execute(&pool).await`
- `Transaction` 和 `PoolConnection` **不再直接实现 `Executor`**，但 `Deref` 到内部连接类型
- 解引用后 `&mut SqliteConnection` 是 Pool 和 Transaction 共同的 `Executor` 类型
- repo 函数统一接受 `&mut SqliteConnection` 参数：
  - Pool 场景：`pool.acquire().await?` → `PoolConnection` (Deref to `SqliteConnection`)
  - Transaction 场景：`&mut *tx` → `&mut SqliteConnection`

## 新架构

### 模块结构

```
accounting-sql/src/
├── lib.rs              # 模块声明 + 重新导出
├── error.rs            # DbError::Database(String)
├── database.rs         # SqliteDatabase struct（持有 SqlitePool）
├── transaction.rs      # SqliteTransaction struct（持有 Transaction<'static, Sqlite>）
├── schema.rs           # async initialize_schema + insert_seed_data
├── repo/
│   ├── mod.rs          # 模块声明
│   ├── account.rs      # 模块级 async 函数（无 trait）
│   ├── commodity.rs
│   ├── member.rs
│   ├── channel.rs
│   ├── tag.rs
│   ├── attachment.rs
│   ├── transaction.rs
│   └── posting.rs
```

### 连接/事务模型

**SqliteDatabase**：持有 `SqlitePool`
- 只读方法：`pool.acquire().await` 获取连接，调用 repo 函数
- 事务方法：`pool.begin().await` 返回 `SqliteTransaction`

**SqliteTransaction**：持有 `sqlx::Transaction<'static, Sqlite>`
- repo 方法通过 `&mut *self.tx` 获取 `&mut SqliteConnection`
- `commit(self)` 消费 self
- Drop 时 sqlx 自动 rollback

**Repo 函数统一签名**：
```rust
pub async fn account_create(
    conn: &mut SqliteConnection,
    account: &Account,
) -> Result<AccountId, DbError> { ... }
```

**SqliteDatabase 的方法（委托模式）**：
```rust
impl SqliteDatabase {
    pub async fn account_create(&self, account: &Account) -> Result<AccountId, DbError> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        repo::account::account_create(&mut conn, account).await
    }
}

impl SqliteTransaction<'_> {
    pub async fn account_create(&mut self, account: &Account) -> Result<AccountId, DbError> {
        repo::account::account_create(&mut *self.tx, account).await
    }
}
```

### Service 层变更

```rust
// 改前
pub struct AccountService<D: Database> { db: D }
let tx = self.db.transaction().await?;
tx.account_repo().create_with_closure(&tx.conn(), &account)?;

// 改后
pub struct AccountService { db: SqliteDatabase }
let mut tx = self.db.transaction().await?;
tx.account_create(&account).await?;
```

---

## Step 1: 依赖替换 ✅ 已完成

**已修改文件**：
- `Cargo.toml`（workspace）：`rusqlite` → `sqlx = { version = "0.9", features = ["runtime-tokio", "sqlite", "chrono", "rust_decimal"] }`
- `accounting-sql/Cargo.toml`：移除 `rusqlite`，加 `sqlx` + `tokio`（从 dev-dep 升到 dep）
- `accounting-service/Cargo.toml`：移除 `rusqlite`

---

## Step 2: error.rs 重构

**文件**: `accounting-sql/src/error.rs`

**改前**：
```rust
#[derive(Error, Debug)]
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}
```

**改后**：
```rust
#[derive(Error, Debug)]
pub enum DbError {
    #[error("database error: {0}")]
    Database(String),
}
```

**注意事项**：
- 所有 `DbError::Sqlite(e)` 或 `DbError::Other(s)` 的 match arm 都要改为 `DbError::Database(s)`
- 所有 `?` 操作符不再有 `#[from]` 自动转换，需改为 `.map_err(|e| DbError::Database(e.to_string()))?`
- accounting-service 中 `AccountingError::DatabaseError` 可能引用 `DbError`，检查 match arm

---

## Step 3: 删除旧 trait 和 pool.rs

**删除内容**：
- `accounting-sql/src/pool.rs`：整个文件（`ConnectionHandle` 不再需要）
- `accounting-sql/src/database.rs`：删除 `Database` trait 定义
- `accounting-sql/src/transaction.rs`：删除 `Transaction` trait 定义
- `accounting-sql/src/repo/*.rs`：删除所有 `pub trait XxxRepo` trait 定义
- `accounting-sql/src/impls/sqlite.rs`：删除 `impl Database for SqliteDatabase` 和 `impl Transaction for SqliteTransaction`
- `accounting-sql/src/impls/mod.rs`：删除（如果只包含 `pub mod sqlite`）

**保留**：每个 repo 文件中的 `SqliteXxxRepo` 实现体（SQL 逻辑）作为重写参考，后续 Step 7 会替换

**注意**：此步和 Step 5-7 需同步进行，否则编译会失败。建议将 Step 3 与 Step 5-7 合并执行，一步到位重写所有文件。

---

## Step 4: 重写 schema.rs（async）

**文件**: `accounting-sql/src/schema.rs`

**改前**：`fn initialize_schema(conn: &Connection)` + `fn insert_seed_data(conn: &Connection, lang: &str)`，内部使用 `conn.execute_batch(SCHEMA_SQL)` 和 `conn.execute(sql, params![...])`

**改后**：
```rust
pub async fn initialize_schema(conn: &mut SqliteConnection) -> Result<(), DbError> {
    // 逐条执行 DDL（sqlx 不支持 execute_batch）
    for sql in SCHEMA_STATEMENTS {
        sqlx::query(sql).execute(conn).await.map_err(|e| DbError::Database(e.to_string()))?;
    }
    Ok(())
}

pub async fn insert_seed_data(conn: &mut SqliteConnection, lang: &str) -> Result<(), DbError> {
    // 种子数据使用 sqlx::query + bind 执行
}
```

**关键变更**：
- `PRAGMA foreign_keys = ON` / `PRAGMA journal_mode = WAL` 移到 `SqliteConnectOptions` 设置（Step 5 的 `SqliteDatabase::open` 中）
- `execute_batch()` 拆分为逐条 SQL 执行
- 将 `SCHEMA_SQL` 常量从单个字符串改为 `SCHEMA_STATEMENTS: &[&str]` 数组
- 触发器中的分号需正确拆分（每个 CREATE TRIGGER 是一条完整语句）
- 种子数据中的 `format!` 拼接改为参数绑定
- 测试中的 `#[test]` 改为 `#[tokio::test]`

---

## Step 5: 重写 database.rs → SqliteDatabase

**文件**: `accounting-sql/src/database.rs`

**改后结构**：
```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use sqlx::SqliteConnection;
use std::str::FromStr;

use crate::error::DbError;
use crate::transaction::SqliteTransaction;
use crate::repo;
use accounting::*;

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn open(path: &str) -> Result<Self, DbError> {
        let options = SqliteConnectOptions::from_str(path)?
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        let mut conn = pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        crate::schema::initialize_schema(&mut conn).await?;
        Ok(Self { pool })
    }

    pub async fn open_in_memory() -> Result<Self, DbError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        let mut conn = pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        crate::schema::initialize_schema(&mut conn).await?;
        Ok(Self { pool })
    }

    pub async fn transaction(&self) -> Result<SqliteTransaction, DbError> {
        let tx = self.pool.begin().await.map_err(|e| DbError::Database(e.to_string()))?;
        Ok(SqliteTransaction::new(tx))
    }

    pub async fn initialize(&self, lang: &str) -> Result<(), DbError> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        crate::schema::insert_seed_data(&mut conn, lang).await?;
        Ok(())
    }

    // === 只读 repo 方法（通过 pool.acquire 获取连接） ===
    pub async fn account_list(&self) -> Result<Vec<Account>, DbError> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        repo::account::account_list(&mut conn).await
    }
    // ... 其他所有 repo 方法 ...

    // === 新增：settings 相关方法 ===
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        repo::get_setting(&mut conn, key).await
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Database(e.to_string()))?;
        repo::set_setting(&mut conn, key, value).await
    }
}
```

**方法命名约定**：`模块名_方法名`，如 `account_create`, `account_list`, `posting_insert`, `transaction_list` 等

---

## Step 6: 重写 transaction.rs → SqliteTransaction

**文件**: `accounting-sql/src/transaction.rs`

**改后结构**：
```rust
use sqlx::{Sqlite, Transaction};
use sqlx::SqliteConnection;
use crate::error::DbError;
use crate::repo;
use accounting::*;

pub struct SqliteTransaction<'a> {
    tx: Transaction<'a, Sqlite>,
}

impl<'a> SqliteTransaction<'a> {
    pub(crate) fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    pub async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await.map_err(|e| DbError::Database(e.to_string()))
    }

    // === repo 方法（通过 &mut *self.tx 获取连接） ===
    pub async fn account_create(&mut self, account: &Account) -> Result<AccountId, DbError> {
        repo::account::account_create(&mut *self.tx, account).await
    }
    // ... 其他所有 repo 方法 ...
}
```

**注意**：`SqliteDatabase::transaction()` 返回 `SqliteTransaction<'static>`，因为 `Pool::begin()` 返回 `Transaction<'static, Sqlite>`

---

## Step 7: 重写 8 个 repo 模块

### 7a: commodity.rs（3 方法，最简单）

**改后**：
```rust
use sqlx::SqliteConnection;
use crate::error::DbError;
use accounting::*;

pub async fn commodity_get_by_symbol(
    conn: &mut SqliteConnection,
    symbol: &str,
) -> Result<Option<Commodity>, DbError> {
    sqlx::query_as::<_, CommodityRow>("SELECT id, symbol, name, precision, created_at, updated_at FROM commodities WHERE symbol = ?1")
        .bind(symbol)
        .fetch_optional(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?
        .map(|r| r.into_commodity())
        .transpose()
}
// ... create, list 类似
```

**行映射**：简单查询用 `FromRow` derive + 辅助 struct，`closed_at` 等有自定义逻辑的用手动 `row.get()`

### 7b: member.rs（4 方法）
### 7c: channel.rs（3 方法）
### 7d: tag.rs（4 方法）
### 7e: attachment.rs（3 方法，涉及 BLOB Vec<u8>）

**BLOB 处理**：sqlx-sqlite 对 `Vec<u8>` 原生支持，`row.get::<Vec<u8>, _>("data")` 直接工作

### 7f: account.rs（16 方法，最复杂之一）

**关键方法**：
- `create_with_closure`：维护闭包表，需多条 SQL 在同一事务中执行
- `map_account`：`closed_at` 需要字符串→日期解析，用 `row.get::<Option<String>, _>("closed_at")` 后手动解析
- `find_root_name` / `find_root_id`：通过闭包表递归查询

**`last_insert_rowid()` 替换**：使用 `RETURNING id` 子句：
```rust
let row: (i64,) = sqlx::query_as("INSERT INTO account (...) VALUES (...) RETURNING id")
    .bind(...)
    .fetch_one(conn)
    .await?;
Ok(AccountId(row.0))
```

### 7g: transaction.rs（6 方法，含动态 SQL）

**动态 SQL 使用 QueryBuilder**：
```rust
use sqlx::QueryBuilder;

pub async fn transaction_list(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
    limit: usize,
    offset: usize,
) -> Result<Vec<Transaction>, DbError> {
    let mut builder = QueryBuilder::new("SELECT DISTINCT t.id, t.date_time, ... FROM transactions t ");
    // 动态 JOIN
    if filter.has_reimbursable == Some(true) {
        builder.push("JOIN postings p ON p.transaction_id = t.id AND p.is_reimbursable = 1 ");
    }
    // 动态 JOIN tags
    if !filter.tag_ids.is_empty() {
        builder.push("JOIN transaction_tags tt ON tt.transaction_id = t.id ");
    }
    builder.push("WHERE 1=1 ");
    // 动态条件
    if let Some(start) = filter.start_date {
        builder.push("AND t.date_time >= ").push_bind(start.naive_local());
    }
    if !filter.member_ids.is_empty() {
        builder.push("AND t.member_id IN (");
        let mut separated = builder.separated(", ");
        for id in &filter.member_ids {
            separated.push_bind(id.0);
        }
        builder.push(") ");
    }
    // ... 其他条件 ...
    builder.push("ORDER BY t.date_time DESC, t.id DESC LIMIT ").push_bind(limit as i64);
    builder.push(" OFFSET ").push_bind(offset as i64);

    let rows = builder.build().fetch_all(conn).await.map_err(|e| DbError::Database(e.to_string()))?;
    // 手动映射行
}
```

### 7h: posting.rs（12 方法，最复杂）

**N+1 查询优化**：
```rust
/// 预加载所有 commodity 精度到 HashMap
async fn load_precisions(
    conn: &mut SqliteConnection,
) -> Result<HashMap<CommodityId, u8>, DbError> {
    let rows: Vec<(i64, i32)> = sqlx::query_as("SELECT id, precision FROM commodities")
        .fetch_all(conn)
        .await?;
    Ok(rows.into_iter().map(|(id, p)| (CommodityId(id), p as u8)).collect())
}
```

**`sum_by_tag`/`sum_by_member`/`sum_by_channel`**：与 transaction.rs 类似的 QueryBuilder 模式，3 个方法共享动态 WHERE 构建逻辑，可提取为辅助函数。

---

## Step 8: 新增 API 层缺失的 repo 方法

当前 API 层有 4 处直接 SQL，需迁入 repo：

1. **`channel_count_transactions_by_id(conn, channel_id)`**：`SELECT COUNT(*) FROM transactions WHERE channel_id = ?1`
2. **`channel_force_delete_by_id(conn, channel_id)`**：`DELETE FROM channels WHERE id = ?1`
3. **`get_setting(conn, key)`**：`SELECT value FROM settings WHERE key = ?1`
4. **`set_setting(conn, key, value)`**：`INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)`

这些函数放在 `repo/mod.rs` 或新建 `repo/settings.rs` 中。

---

## Step 9: Service 层适配

**6 个 service 文件统一变更**：

1. `<D: Database>` 泛型参数 → 直接使用 `SqliteDatabase`
2. `self.db.transaction().await?` → 返回 `SqliteTransaction<'static>`
3. `tx.xxx_repo().method(&tx.conn(), ...)` → `tx.xxx_method(...).await`
4. `self.db.connection()` 只读操作 → `self.db.xxx_method(...).await`
5. `tx.commit().await?` 保持不变
6. 测试中 `rusqlite::params!` 直接 SQL → 通过 repo 方法或 `sqlx::query`
7. `accounting-service/Cargo.toml`：移除 `rusqlite`（已做）

**具体文件**：
- `account_service.rs`：最多改动，58 处调用
- `transaction_service.rs`
- `report_service.rs`
- `commodity_service.rs`
- `member_service.rs`
- `tag_service.rs`

---

## Step 10: API 层适配

**关键变更**：
- `AppState` 改为持有 `SqliteDatabase` 而非 `db_path: String`
- `AppState::db()` 返回 `&SqliteDatabase`（启动时创建一次）
- 所有 `db.connection()` + `db.xxx_repo().method(&conn, ...)` → `db.xxx_method(...).await`
- 移除 `use accounting_sql::database::Database;`
- 移除 `use accounting_sql::transaction::Transaction;`
- `handlers/channel.rs`：直接 SQL 改为 `db.channel_count_transactions_by_id(id).await` + `db.channel_force_delete_by_id(id).await`
- `handlers/me.rs`：直接 SQL 改为 `db.get_setting("current_member_id").await` + `db.set_setting("current_member_id", &id).await`

**文件**：
- `handlers/account.rs`
- `handlers/transaction.rs`
- `handlers/member.rs`
- `handlers/channel.rs`
- `handlers/tag.rs`
- `handlers/commodity.rs`
- `handlers/report.rs`
- `handlers/me.rs`
- `main.rs`

---

## Step 11: CLI 层适配

- `db.account_repo().find_root_name(&conn, ...)` → `db.account_find_root_name(...).await`
- `db.commodity_repo().get_by_symbol(&conn, ...)` → `db.commodity_get_by_symbol(...).await`
- `db.tag_repo().get_by_name(&conn, ...)` → `db.tag_get_by_name(...).await`
- `main.rs`：`db.connection().query_row(...)` → `db.get_setting("language").await`
- 移除 `use accounting_sql::database::Database;`
- `db.initialize(&lang)` → `db.initialize(&lang).await`

**文件**：
- `cmd/account.rs`
- `cmd/tx.rs`
- `cmd/report.rs`
- `cmd/member.rs`
- `cmd/commodity.rs`
- `cmd/tag.rs`
- `main.rs`

---

## Step 12: 测试适配

- 所有 `#[test]` → `#[tokio::test]`
- `setup()` 返回 `(Connection, XxxRepo)` → `async fn setup() -> SqliteDatabase`
- 所有 repo 调用加 `.await`
- `SqliteDatabase::open_in_memory()` → `SqliteDatabase::open_in_memory().await`
- Service 测试中的 `rusqlite::params!` 直接 SQL → 通过 repo 方法或 `sqlx::query`

---

## Step 13: lib.rs 更新

```rust
// 改后
pub mod database;    // SqliteDatabase
pub mod transaction; // SqliteTransaction
pub mod error;       // DbError
pub mod schema;      // initialize_schema, insert_seed_data
pub mod repo;        // 模块级函数

pub use database::SqliteDatabase;
pub use transaction::SqliteTransaction;
pub use error::DbError;
```

**删除**：
- `pub mod pool;`（删除文件）
- `pub mod impls;`（删除目录）

---

## Step 14: 验证

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

确认：
- 所有 `rusqlite` 引用已移除
- 所有 `DbError::Sqlite` / `DbError::Other` 改为 `DbError::Database`
- 所有 `Connection` / `MutexGuard` 引用已移除
- 所有同步调用改为 async + `.await`

---

## Phase 2：PostgreSQL feature gate（概要）

1. `Cargo.toml` 添加 feature gate：`default = ["sqlite"]`，`sqlite = ["sqlx/sqlite"]`，`postgres = ["sqlx/postgres"]`
2. 新增 `src/pg_schema.rs`：SQLite DDL → PostgreSQL DDL 转换
3. 新增 `src/pg/` 目录：PostgreSQL 版本的 repo 函数（`$1, $2` 参数占位符等）
4. 新增 `PostgresDatabase` / `PostgresTransaction` 结构体
5. `src/database.rs` / `src/transaction.rs` 用 `cfg` 条件编译选择实现
6. PostgreSQL 测试需要 Docker 或 CI service container

---

## 关键文件清单

| 文件 | 操作 |
|------|------|
| `Cargo.toml`（workspace） | ✅ 已改：rusqlite → sqlx 0.9 |
| `accounting-sql/Cargo.toml` | ✅ 已改：rusqlite → sqlx |
| `accounting-service/Cargo.toml` | ✅ 已改：移除 rusqlite |
| `accounting-sql/src/error.rs` | 重写 DbError |
| `accounting-sql/src/pool.rs` | 删除 |
| `accounting-sql/src/database.rs` | 重写为 SqliteDatabase |
| `accounting-sql/src/transaction.rs` | 重写为 SqliteTransaction |
| `accounting-sql/src/schema.rs` | 改为 async |
| `accounting-sql/src/impls/sqlite.rs` | 删除 |
| `accounting-sql/src/impls/mod.rs` | 删除 |
| `accounting-sql/src/repo/*.rs`（8 个） | trait + impl → 模块级 async 函数 |
| `accounting-sql/src/repo/mod.rs` | 更新模块声明 |
| `accounting-sql/src/lib.rs` | 更新模块声明和导出 |
| `accounting-service/src/*.rs`（6 个） | 移除泛型，async 适配 |
| `accounting-api/src/handlers/*.rs`（8 个） | async 适配 |
| `accounting-api/src/main.rs` | async 初始化 |
| `accounting-cli/src/cmd/*.rs`（6 个） | async 适配 |
| `accounting-cli/src/main.rs` | async 初始化 |

## 风险与注意事项

1. **sqlx 0.9 Transaction Deref 模式**：`&mut *tx` 解引用到 `&mut SqliteConnection`，需验证此模式可用
2. **QueryBuilder 动态 SQL**：TransactionRepo 和 PostingRepo 的复杂动态 WHERE 需仔细迁移，`separated()` API 用于 IN 子句
3. **chrono 类型映射**：sqlx 对 `NaiveDateTime` 的 TEXT 存储可能需要手动处理（SQLite 中 datetime 以 TEXT 存储）
4. **BLOB 数据**：sqlx-sqlite 对 `Vec<u8>` 的映射需验证
5. **编译时间**：不启用 sqlx 的 `macros` feature（不用 `query!`），使用运行时 `sqlx::query()` / `sqlx::query_as()`
6. **sqlx 0.9 稳定性**：0.9.0 可能是 alpha/beta，需关注正式发布状态；如果 0.9 不稳定可回退到 0.8
