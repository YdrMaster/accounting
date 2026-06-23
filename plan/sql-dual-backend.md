# accounting-sql 双后端兼容改造方案

## 目标

在 accounting-sql 库中，通过相同 trait 的不同实现类型，兼容：
- **rusqlite**（个人自部署，SQLite 单文件）
- **sqlx + PostgreSQL**（产品化部署，schema-per-tenant 多租户）

---

## 一、现有代码与 rusqlite 的耦合点清单

### 1.1 Repo trait 的 `conn: &Connection` 参数

**这是最大的耦合点。** 所有 8 个 Repo trait 的每个方法都接收 `conn: &rusqlite::Connection` 作为第一个参数：

```rust
// 当前：每个 Repo 方法都绑定 rusqlite::Connection
pub trait AccountRepo {
    fn create(&self, conn: &Connection, account: &Account) -> Result<AccountId, DbError>;
    fn get(&self, conn: &Connection, id: AccountId) -> Result<Option<Account>, DbError>;
    // ...
}
```

sqlx 的查询执行依赖 `sqlx::PgPool` 或 `sqlx::Transaction<'_, Postgres>`，与 `rusqlite::Connection` 完全不兼容。这意味着 **Repo trait 的签名必须重构**。

### 1.2 Database trait 的 `connection()` 返回类型

```rust
fn connection(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection>;
```

直接返回 `MutexGuard<rusqlite::Connection>`，PostgreSQL 无法实现此方法。

### 1.3 Transaction trait 的 `conn()` 返回类型

```rust
fn conn(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection>;
```

同上，直接暴露 rusqlite 类型。

### 1.4 DbError 与 rusqlite::Error 的耦合

```rust
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}
```

`#[from] rusqlite::Error` 使得 `?` 运算符自动将 rusqlite 错误转为 DbError，但 PostgreSQL 实现需要转为 sqlx 错误。

### 1.5 pool.rs 全部是 rusqlite 实现

`ConnectionHandle` 封装 `Arc<Mutex<Connection>>`，这是 SQLite 专用的连接模型，PostgreSQL 需要完全不同的连接池。

### 1.6 schema.rs 是 SQLite 专属 DDL

`SCHEMA_SQL` 使用 SQLite 方言（`AUTOINCREMENT`、`date('now')`、`PRAGMA` 等），PostgreSQL 需要独立的 schema 定义。

### 1.7 Service 层测试直接使用 rusqlite

`account_service.rs` 测试中直接调用 `conn.query_row(..., rusqlite::params![...])`。

---

## 二、核心设计问题：async vs sync

这是整个改造方案最关键的决策点。

| 维度 | rusqlite（SQLite） | sqlx（PostgreSQL） |
|------|---|---|
| **API 风格** | 同步（`fn`） | 异步（`async fn`） |
| **连接模型** | `Arc<Mutex<Connection>>` | `PgPool`（内部管理连接池） |
| **事务模型** | `BEGIN/COMMIT/ROLLBACK` + Drop 守卫 | `Pool::begin()` 返回 `Transaction<'_, Postgres>` |
| **查询执行** | `conn.prepare().query().next()` | `sqlx::query_as().fetch_one().await` |
| **参数绑定** | `params![...]` 宏 | `query.bind()` 或 `.bind(value)` 链式调用 |

**两套 API 无法共享同一个 trait 签名**——一个 `fn`，一个 `async fn`，Rust 类型系统不允许。

---

## 三、改造方案

### 方案选择：Repo trait async 化 + rusqlite 用 spawn_blocking 适配

将所有 Repo trait 改为 `async fn`，rusqlite 实现通过 `tokio::task::spawn_blocking` 将同步调用包装为异步。

**为什么不用"两套独立 trait"**：如果定义 `SyncAccountRepo` + `AsyncAccountRepo`，Service 层就必须写两套逻辑或引入更多泛型约束，破坏现有的 `D: Database` 简洁模式。统一为 async 是最小侵入方案。

**为什么不用"trait 保持 sync，PG 用阻塞 API"**：sqlx 的阻塞模式（`sqlx::query_as().fetch_one().await` 不能在非 async 上下文调用）需要引入 `tokio::runtime::Runtime` 嵌套，且 PG 连接池本身是异步设计，强行同步化会丢失并发优势。

### 3.1 Repo trait 重构

**核心改动：移除 `conn` 参数，将连接管理内化到 Repo 实现中。**

```rust
// 改造前
pub trait AccountRepo {
    fn create(&self, conn: &Connection, account: &Account) -> Result<AccountId, DbError>;
}

// 改造后
#[async_trait]
pub trait AccountRepo: Send + Sync {
    async fn create(&self, account: &Account) -> Result<AccountId, DbError>;
}
```

**理由**：
- SQLite 实现将 `ConnectionHandle` 存为 Repo 字段，方法内部获取连接
- PostgreSQL 实现将 `PgPool` 存为 Repo 字段，方法内部通过 pool 获取连接
- 两种实现的连接获取方式完全不同，不应暴露在 trait 签名中

### 3.2 Database trait 重构

```rust
// 改造前
pub trait Database: Send + Sync {
    type Tx: Transaction;
    fn account_repo(&self) -> &dyn AccountRepo;
    fn connection(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection>;
    async fn transaction(&self) -> Result<Self::Tx, DbError>;
    fn initialize(&self, lang: &str) -> Result<(), DbError>;
}

// 改造后
#[async_trait]
pub trait Database: Send + Sync {
    type Tx: Transaction;

    fn account_repo(&self) -> &dyn AccountRepo;
    fn commodity_repo(&self) -> &dyn CommodityRepo;
    fn member_repo(&self) -> &dyn MemberRepo;
    fn channel_repo(&self) -> &dyn ChannelRepo;
    fn tag_repo(&self) -> &dyn TagRepo;
    fn attachment_repo(&self) -> &dyn AttachmentRepo;
    fn transaction_repo(&self) -> &dyn TransactionRepo;
    fn posting_repo(&self) -> &dyn PostingRepo;

    async fn transaction(&self) -> Result<Self::Tx, DbError>;
    async fn initialize(&self, lang: &str) -> Result<(), DbError>;
}
```

**关键变化**：
- 移除 `connection()` 方法——连接是实现细节，不应暴露
- `initialize()` 改为 `async fn`——PG 的 schema 初始化涉及网络 I/O
- Repo 的 `xxx_repo()` 返回类型不变（`&dyn XxxRepo`），但 Repo trait 本身已 async 化

### 3.3 Transaction trait 重构

```rust
// 改造前
pub trait Transaction: Send {
    fn account_repo(&self) -> &dyn AccountRepo;
    fn conn(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection>;
    async fn commit(self) -> Result<(), DbError>;
}

// 改造后
#[async_trait]
pub trait Transaction: Send {
    fn account_repo(&self) -> &dyn AccountRepo;
    fn commodity_repo(&self) -> &dyn CommodityRepo;
    fn member_repo(&self) -> &dyn MemberRepo;
    fn channel_repo(&self) -> &dyn ChannelRepo;
    fn tag_repo(&self) -> &dyn TagRepo;
    fn attachment_repo(&self) -> &dyn AttachmentRepo;
    fn transaction_repo(&self) -> &dyn TransactionRepo;
    fn posting_repo(&self) -> &dyn PostingRepo;

    async fn commit(self) -> Result<(), DbError>;
}
```

**关键变化**：移除 `conn()` 方法——与 `connection()` 同理，连接是实现细节。

### 3.4 DbError 重构

```rust
// 改造前
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}

// 改造后
pub enum DbError {
    #[error("database error: {0}")]
    Database(String),
}
```

**理由**：DbError 是跨后端的错误类型，不应绑定任何特定数据库驱动。rusqlite 和 sqlx 的错误都在各自实现中转为 `DbError::Database(e.to_string())`。

**代价**：失去 `#[from]` 的自动转换，需要在每个实现中显式 `.map_err(|e| DbError::Database(e.to_string()))`。但这正是正确做法——实现层负责将驱动特定错误统一为领域错误。

### 3.5 rusqlite 实现适配

每个 `SqliteXxxRepo` 需要持有 `ConnectionHandle`，并在方法内通过 `spawn_blocking` 包装：

```rust
pub struct SqliteAccountRepo {
    pool: ConnectionHandle,
}

#[async_trait]
impl AccountRepo for SqliteAccountRepo {
    async fn create(&self, account: &Account) -> Result<AccountId, DbError> {
        let pool = self.pool.clone();
        let account = account.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get();
            conn.execute(
                "INSERT INTO accounts (...) VALUES (...)",
                params![...],
            ).map_err(|e| DbError::Database(e.to_string()))?;
            Ok(AccountId(conn.last_insert_rowid()))
        }).await.map_err(|e| DbError::Database(e.to_string()))?
    }
}
```

**关键点**：
- `Account` 需要实现 `Clone`（当前已派生 `Clone`）
- `ConnectionHandle` 需要 `Send + Sync`（当前已满足，`Arc<Mutex<Connection>>` 是 `Send + Sync`）
- `spawn_blocking` 将同步的 rusqlite 调用移到独立线程，不阻塞 tokio 运行时

### 3.6 SqliteDatabase / SqliteTransaction 适配

```rust
pub struct SqliteDatabase {
    pool: ConnectionHandle,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    // ... 其他 repo
}

pub struct SqliteTransaction {
    pool: ConnectionHandle,
    committed: bool,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    // ... 其他 repo
}
```

**关键变化**：
- Repo 现在持有 `ConnectionHandle`，而非无状态的 `SqliteAccountRepo`
- `Database::transaction()` 中 `BEGIN` 需要在 `spawn_blocking` 中执行
- `Transaction::commit()` 中 `COMMIT` 需要在 `spawn_blocking` 中执行
- Drop 守卫中的 `ROLLBACK` 不变（同步 Drop，但 SQLite 事务超时会自动回滚）

### 3.7 PostgreSQL 实现新增

```rust
pub struct PostgresDatabase {
    pool: PgPool,
    account_repo: PostgresAccountRepo,
    commodity_repo: PostgresCommodityRepo,
    // ... 其他 repo
}

pub struct PostgresTransaction<'a> {
    tx: sqlx::Transaction<'a, Postgres>,
    // 或使用 sqlx::Transaction 的方式管理
}
```

**PG 实现要点**：
- 连接池使用 `sqlx::PgPool`
- 事务使用 `pool.begin().await` 返回 `sqlx::Transaction<'_, Postgres>`
- Schema 初始化使用 `CREATE SCHEMA IF NOT EXISTS {tenant}` + `SET search_path = {tenant}` + 标准 PG DDL
- 金额存储使用 `BIGINT`（等价于 SQLite 的 `INTEGER`）
- `AUTOINCREMENT` → `GENERATED ALWAYS AS IDENTITY` 或 `SERIAL`
- `date('now')` → `CURRENT_DATE`
- `BLOB` → `BYTEA`
- 触发器语法适配（PG 触发器需要显式 `RETURN NEW` 等，但逻辑结构一致）
- 种子数据适配（PG 的 `INSERT ... SELECT` 子查询语法略有差异）

### 3.8 Service 层适配

Service 层现有的 `D: Database` 泛型模式不需要改变。但调用模式需要调整：

```rust
// 改造前
pub async fn create(&self, ...) -> Result<...> {
    let tx = self.db.transaction().await?;
    tx.account_repo().create_with_closure(&tx.conn(), &account)?;
    tx.commit().await?;
}

// 改造后
pub async fn create(&self, ...) -> Result<...> {
    let tx = self.db.transaction().await?;
    tx.account_repo().create_with_closure(&account).await?;
    tx.commit().await?;
}
```

**关键变化**：
- 移除所有 `&conn` 和 `&tx.conn()` 参数传递
- Repo 方法调用后加 `.await`
- `self.db.connection()` 中的只读操作改为 `self.db.xxx_repo().xxx().await`

### 3.9 依赖结构调整

```
accounting-sql/Cargo.toml 改造后：

[dependencies]
accounting = { workspace = true }
async-trait = "0.1"
tokio = { workspace = true, features = ["rt", "macros"] }  # spawn_blocking 用
chrono = { workspace = true }
rust_decimal = { workspace = true }
thiserror = { workspace = true }

# SQLite 后端（可选）
rusqlite = { workspace = true, optional = true }

# PostgreSQL 后端（可选）
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "rust_decimal"], optional = true }

[features]
default = ["sqlite"]
sqlite = ["rusqlite"]
postgres = ["sqlx"]
```

**注意**：`async-trait` crate 是必须依赖，不是可选的——因为所有 Repo trait 都需要 `#[async_trait]`。

---

## 四、需要修改的文件清单

### accounting-sql 库内

| 文件 | 改动类型 | 说明 |
|------|---------|------|
| `src/repo/account.rs` | **重构** | `AccountRepo` trait 改 async、移除 conn 参数；`SqliteAccountRepo` 持有 ConnectionHandle、spawn_blocking 适配 |
| `src/repo/commodity.rs` | **重构** | 同上模式 |
| `src/repo/member.rs` | **重构** | 同上模式 |
| `src/repo/channel.rs` | **重构** | 同上模式 |
| `src/repo/tag.rs` | **重构** | 同上模式 |
| `src/repo/attachment.rs` | **重构** | 同上模式 |
| `src/repo/transaction.rs` | **重构** | 同上模式 |
| `src/repo/posting.rs` | **重构** | 同上模式（最复杂，动态 SQL 构建逻辑需全部异步化） |
| `src/database.rs` | **重构** | 移除 `connection()`、`initialize` 改 async、加 `#[async_trait]` |
| `src/transaction.rs` | **重构** | 移除 `conn()`、加 `#[async_trait]` |
| `src/pool.rs` | **保留** | SQLite 专用，仅在 `sqlite` feature 下编译 |
| `src/schema.rs` | **保留** | SQLite 专用 DDL，仅在 `sqlite` feature 下编译 |
| `src/impls/sqlite.rs` | **重构** | Repo 持有 ConnectionHandle、transaction/commit 用 spawn_blocking |
| `src/error.rs` | **重构** | `Sqlite(#[from])` → `Database(String)` |
| `src/lib.rs` | **修改** | 加 `cfg` feature gate、导出 PG 模块 |
| `src/impls/mod.rs` | **修改** | 加 `cfg` feature gate |
| `src/repo/mod.rs` | **修改** | 加 `cfg` feature gate |
| **新增** `src/pg/` 目录 | **新增** | PostgreSQL 实现的全部代码 |
| **新增** `src/pg/schema.rs` | **新增** | PostgreSQL 专属 DDL（SERIAL、CURRENT_DATE、BYTEA 等） |
| **新增** `src/pg/pool.rs` | **新增** | PgPool 封装 + 租户 search_path 管理 |
| **新增** `src/pg/impls.rs` | **新增** | PostgresDatabase / PostgresTransaction 实现 |
| **新增** `src/pg/repo/*.rs` | **新增** | 8 个 PostgresXxxRepo 实现 |
| `Cargo.toml` | **修改** | 加 async-trait、tokio、sqlx 依赖、feature gate |

### accounting-sql 外部

| Crate / 文件 | 改动类型 | 说明 |
|------|---------|------|
| `accounting-service/src/*.rs` | **重构** | 移除所有 `&conn` / `&tx.conn()` 参数传递，Repo 调用加 `.await` |
| `accounting-api/src/*.rs` | **小改** | 构造 Database 实例时根据配置选择 sqlite 或 postgres |
| `accounting-cli/src/*.rs` | **小改** | 同上 |
| `accounting/Cargo.toml` | **不改** | 核心库零改动 |

---

## 五、风险与注意事项

### 5.1 spawn_blocking 的线程池压力

rusqlite 的所有数据库操作（包括读）都通过 `spawn_blocking` 在阻塞线程池执行。tokio 的阻塞线程池默认大小为 512，对于个人自部署场景足够。但如果并发请求很高，需要调大 `max_blocking_threads`。

### 5.2 SqliteTransaction 的 Drop 守卫

当前 `Drop for SqliteTransaction` 在未 commit 时自动 `ROLLBACK`。改为 async 后，Drop 仍然是同步的，`ROLLBACK` 可以在同步 Drop 中执行（`conn.execute("ROLLBACK", [])` 是同步调用，不需要 spawn_blocking）。

### 5.3 PG 事务的生命周期

sqlx 的 `Transaction<'_, Postgres>` 借用了 `PgPool`，生命周期与 SQLite 的 `Arc<Mutex<Connection>>` 模式不同。PostgresTransaction 的实现需要仔细处理 `sqlx::Transaction` 的借用规则——可能需要用 `'static` 事务（通过 `Pool::begin()` 返回 `'static` 事务）或调整 `Transaction` trait 的设计。

### 5.4 PG 触发器适配工作量

当前 14 个触发器全部是 SQLite 方言，需要逐个改写为 PostgreSQL 语法。主要差异：
- `AFTER UPDATE ON xxx FOR EACH ROW WHEN ...` → PG 需要显式 `RETURN NEW` 或 `RETURN NULL`
- `date('now')` → `CURRENT_DATE`
- `NEW.updated_at = NEW.updated_at` 的 WHEN 条件在 PG 中语法相同
- reversal_total 触发器的逻辑结构可以保留，但语法需适配

### 5.5 种子数据的递归 CTE

`SEED_CLOSURE` 中的 `WITH RECURSIVE` 在 PG 中语法相同，但 `INSERT OR IGNORE` 需改为 `INSERT ... ON CONFLICT DO NOTHING`。

### 5.6 测试策略

- SQLite 实现的测试：保持内存数据库测试，但需要改为 async 测试（`#[tokio::test]`）
- PostgreSQL 实现的测试：需要 PG 测试实例（CI 中用 Docker 或 `sqlx::test` 宏）
- Service 层测试中直接使用 `rusqlite::params!` 的代码需要改为通过 Repo trait 操作

---

## 六、结论

**方案可行，但改造范围较大。**

核心改动集中在三处：
1. **Repo trait async 化 + 移除 conn 参数**（8 个 trait × ~10 个方法 = ~80 个方法签名变更）
2. **rusqlite 实现适配 spawn_blocking**（8 个 Repo 实现需要重构）
3. **新增 PostgreSQL 全套实现**（8 个 Repo + Database + Transaction + schema + pool）

accounting crate 本身**零改动**，这验证了之前的结论：核心数据结构不需要修改。

**建议实施顺序**：
1. 先重构 trait（async 化 + 移除 conn），此时只有 sqlite 实现，确保全部测试通过
2. 再新增 postgres 实现，通过 feature gate 隔离
3. 最后适配 Service 层和 API 层
