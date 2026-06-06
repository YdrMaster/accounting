# CLI 设计文档

> 范围：accounting-cli 命令行工具
> 参考：~/repos/accounting-last/docs/superpowers/specs/2026-06-04-cli-design.md

## 1. 设计目标

1. **架构正确**：CLI 只依赖 Service 和 Domain 层，不直接调用 Repository。
2. **参数结构化**：每个子命令使用独立的 `clap::Args` struct 定义，避免参数挤在 enum variant 上。
3. **输出友好**：支持 JSON（管道友好）和 对齐表格（人类可读）两种格式，表格使用 `tabled` crate。
4. **显式初始化**：数据库文件通过 `initialize` 子命令显式创建，避免隐式行为。
5. **Async Runtime**：Service 层所有函数均为 `async fn`，CLI 使用 `tokio::main`。

## 2. 架构概述

```
accounting-cli/
├── Cargo.toml
└── src/
    ├── main.rs          # CLI 入口、db 存在性校验、--format 全局选项
    ├── cmd/
    │   ├── mod.rs       # Commands enum + Initialize 命令
    │   ├── member.rs
    │   ├── account.rs
    │   ├── commodity.rs
    │   ├── tx.rs
    │   ├── tag.rs
    │   └── report.rs
    └── output.rs        # JSON / Table（tabled）输出封装
```

### 2.1 Async Runtime

```rust
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    // ...
}
```

### 2.2 数据库入口

CLI 持有 `SqliteDatabase`（实现 `Database` trait），传递给各 Service 函数。

```rust
let db = SqliteDatabase::open(&cli.db.to_string_lossy()).unwrap();
```

### 2.3 错误处理

各 `Cmd::run` 返回 `Result<(), AccountingError>`，main 中统一匹配：

```rust
if let Err(e) = result {
    eprintln!("错误: {:?}", e);
    std::process::exit(1);
}
```

## 3. 数据库路径与初始化

### 3.1 路径规则

数据库文件路径作为第一个 positional argument 传入：

```
accounting <DB_PATH> <COMMAND>
```

**约束**：若 `DB_PATH` 不存在且命令不是 `initialize`，直接报错退出。

### 3.2 initialize 命令

```
accounting <DB_PATH> initialize
```

**行为**：

1. 若文件已存在，报错（防止覆盖）。
2. 创建 SQLite 文件并执行 schema + seed data。
3. 输出 `"数据库已初始化"`。

## 4. 全局命令结构

```rust
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// 数据库文件路径
    db: PathBuf,
    /// 输出格式
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化数据库
    Initialize,
    /// 成员管理
    #[command(subcommand)]
    Member(MemberCmd),
    /// 账户管理
    #[command(subcommand)]
    Account(AccountCmd),
    /// 商品/货币管理
    #[command(subcommand)]
    Commodity(CommodityCmd),
    /// 交易管理
    #[command(subcommand)]
    Tx(TxCmd),
    /// 标签管理
    #[command(subcommand)]
    Tag(TagCmd),
    /// 报告查询
    #[command(subcommand)]
    Report(ReportCmd),
}
```

## 5. 各实体命令设计

### 5.1 member

```
member list [--limit <N>] [--offset <N>]
member add <NAME>
member delete <ID>
```

```rust
#[derive(Subcommand)]
enum MemberCmd {
    List(MemberListArgs),
    Add(MemberAddArgs),
    Delete(MemberDeleteArgs),
}

#[derive(Args)]
struct MemberListArgs {
    #[arg(long)]
    limit: Option<i64>,
    #[arg(long)]
    offset: Option<i64>,
}

#[derive(Args)]
struct MemberAddArgs {
    name: String,
}

#[derive(Args)]
struct MemberDeleteArgs {
    id: i64,
}
```

### 5.2 account

```
account list [--type <TYPE>] [--limit <N>] [--offset <N>]
account add <FULL_NAME> --type <TYPE> [--parent <ID>] [--billing-day <D>] [--repayment-day <D>]
account show <ID>
account close <ID>
account reopen <ID>
account balance <ID>
```

```rust
#[derive(Subcommand)]
enum AccountCmd {
    List(AccountListArgs),
    Add(AccountAddArgs),
    Show(AccountShowArgs),
    Close(AccountCloseArgs),
    Reopen(AccountReopenArgs),
    Balance(AccountBalanceArgs),
}

#[derive(Args)]
struct AccountListArgs {
    #[arg(long, value_enum)]
    r#type: Option<AccountTypeArg>,
    #[arg(long)]
    limit: Option<i64>,
    #[arg(long)]
    offset: Option<i64>,
}

#[derive(Args)]
struct AccountAddArgs {
    full_name: String,
    #[arg(long, value_enum)]
    r#type: AccountTypeArg,
    #[arg(long)]
    parent: Option<i64>,
    #[arg(long)]
    billing_day: Option<u8>,
    #[arg(long)]
    repayment_day: Option<u8>,
}

#[derive(Args)]
struct AccountShowArgs { id: i64 }

#[derive(Args)]
struct AccountCloseArgs { id: i64 }

#[derive(Args)]
struct AccountReopenArgs { id: i64 }

#[derive(Args)]
struct AccountBalanceArgs { id: i64 }
```

> `AccountTypeArg` 是 `clap::ValueEnum`，映射到 `accounting::account_type::AccountType`：
> `asset`, `liability`, `equity`, `income`, `expense`。

### 5.3 commodity

```
commodity list
commodity add <SYMBOL> --name <NAME> --precision <N>
```

```rust
#[derive(Subcommand)]
enum CommodityCmd {
    List,
    Add(CommodityAddArgs),
}

#[derive(Args)]
struct CommodityAddArgs {
    symbol: String,
    #[arg(long)]
    name: String,
    #[arg(long, default_value = "2")]
    precision: u8,
}
```

### 5.4 tx

```
tx add --date <DATE> --description <DESC>
  --posting <ACCOUNT>:<COMMODITY>:<AMOUNT>[:<COST_COMMODITY>:<COST>]
  [--posting <ACCOUNT>:<COMMODITY>:<AMOUNT> ...]
  [--tag <TAG>] [--member <ID>] [--channel <ID>]

tx list
  [--from <DATE>] [--to <DATE>]
  [--account <ID>] [--member <ID>] [--tag <TAG>] [--keyword <TEXT>]
  [--limit <N>] [--offset <N>]

tx show <ID>
tx delete <ID>
tx update <ID>  # 全量替换，参数同 add
```

```rust
#[derive(Subcommand)]
enum TxCmd {
    Add(TxAddArgs),
    List(TxListArgs),
    Show(TxShowArgs),
    Delete(TxDeleteArgs),
    Update(TxUpdateArgs),
}

#[derive(Args)]
struct TxAddArgs {
    #[arg(long)]
    date: String,
    #[arg(long)]
    description: String,
    #[arg(long, value_delimiter = ';')]
    posting: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    tag: Vec<String>,
    #[arg(long)]
    member: Option<i64>,
    #[arg(long)]
    channel: Option<i64>,
}

#[derive(Args)]
struct TxListArgs {
    #[arg(long)]
    from: Option<String>,
    #[arg(long)]
    to: Option<String>,
    #[arg(long)]
    account: Option<i64>,
    #[arg(long)]
    member: Option<i64>,
    #[arg(long)]
    tag: Option<String>,
    #[arg(long)]
    keyword: Option<String>,
    #[arg(long)]
    limit: Option<i64>,
    #[arg(long)]
    offset: Option<i64>,
}

#[derive(Args)]
struct TxShowArgs { id: i64 }

#[derive(Args)]
struct TxDeleteArgs { id: i64 }

#[derive(Args)]
struct TxUpdateArgs {
    id: i64,
    #[arg(long)]
    date: String,
    #[arg(long)]
    description: String,
    #[arg(long, value_delimiter = ';')]
    posting: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    tag: Vec<String>,
    #[arg(long)]
    member: Option<i64>,
    #[arg(long)]
    channel: Option<i64>,
}
```

> `--posting` 格式：`account_full_name:commodity_symbol:amount` 或 `account_full_name:commodity_symbol:amount:cost_commodity:cost`。
> 多个 posting 用分号 `;` 分隔，如 `--posting "Assets:Cash:CNY:-100;Expenses:Food:CNY:100"`。

### 5.5 tag

```
tag list
tag add <NAME>
tag delete <NAME>
```

```rust
#[derive(Subcommand)]
enum TagCmd {
    List,
    Add(TagAddArgs),
    Delete(TagDeleteArgs),
}

#[derive(Args)]
struct TagAddArgs {
    name: String,
}

#[derive(Args)]
struct TagDeleteArgs {
    name: String,
}
```

### 5.6 report

```
report balance <ACCOUNT_ID>
report bs                    # 资产负债表
report is                    # 损益表
```

```rust
#[derive(Subcommand)]
enum ReportCmd {
    Balance(ReportBalanceArgs),
    Bs,
    Is,
}

#[derive(Args)]
struct ReportBalanceArgs {
    account_id: i64,
}
```

## 6. Service 层需新增的 CLI 查询接口

| 模块 | 新增函数 | 说明 |
|------|---------|------|
| `MemberService` | `list(db, limit, offset) -> Vec<Member>` | 列出成员 |
| `CommodityService` | `list(db) -> Vec<Commodity>` | 列出商品 |
| `TagService` | `list(db) -> Vec<Tag>` | 列出标签 |
| `AccountService` | `list(db, filter) -> Vec<Account>` | 按条件列出账户 |
| `TransactionService` | `list(db, filter, limit, offset) -> Vec<(Transaction, Vec<Posting>)>` | 列出交易含分录 |
| `TransactionService` | `get(db, id) -> Option<(Transaction, Vec<Posting>)>` | 查询单笔交易含分录 |
| `ReportService` | `balance_sheet(db) -> ...` | 资产负债表 |
| `ReportService` | `income_statement(db) -> ...` | 损益表 |

## 7. 输出模块

```rust
use tabled::{Table, Tabled};

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// 打印单个对象
pub fn print<T: Tabled + serde::Serialize>(value: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value).unwrap()),
        OutputFormat::Table => println!("{}", Table::new([value]).to_string()),
    }
}

/// 打印对象列表
pub fn print_vec<T: Tabled + serde::Serialize>(values: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(values).unwrap()),
        OutputFormat::Table => println!("{}", Table::new(values).to_string()),
    }
}

/// 打印单行消息
pub fn print_line(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{{\"result\":\"{}\"}}", msg),
        OutputFormat::Table => println!("{}", msg),
    }
}
```

### 7.1 Tabled 实现

Domain 实体需实现 `Tabled` trait。对于简单实体：

```rust
use tabled::Tabled;

impl Tabled for Member {
    const LENGTH: usize = 2;
    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        vec![self.id.0.to_string().into(), self.name.clone().into()]
    }
    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec!["ID".into(), "名称".into()]
    }
}
```

对于 `Account` 等字段较多的实体，可精简输出列（如省略 `billing_day`、`repayment_day`）。

## 8. Cargo.toml 依赖

```toml
[package]
name = "accounting-cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "accounting"
path = "src/main.rs"

[dependencies]
accounting = { path = "../accounting" }
accounting-sql = { path = "../accounting-sql" }
accounting-service = { path = "../accounting-service" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tabled = "0.17"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

## 9. 与现有代码的关系

### 9.1 不破坏的层面

- **Domain 层**：完全不动。
- **Repository 层**：trait 定义完全保留。
- **Service 写操作**：完全保留。
- **数据库 Schema**：不动。

### 9.2 需要改动的层面

- **Service 层新增**：MemberService、CommodityService、TagService、TransactionService 查询函数。
- **accounting-cli crate**：完全重写，从同步调用改为 async + Service 层调用。
- **Cargo.toml**：新增 `tokio`、`tabled`、`serde`、`serde_json` 依赖。

### 9.3 依赖关系

```
accounting-cli
├── accounting (domain)
├── accounting-service (service)
├── tokio
├── clap
├── tabled
├── serde
└── serde_json
```

## 10. 规格自检

| 检查项 | 状态 | 说明 |
|-------|------|------|
| 占位符 | ✅ | 无 "TODO"、"待定"、未完成章节 |
| 内部一致性 | ✅ | CLI → Service → Repository 架构与各命令设计一致 |
| 范围 | ✅ | 聚焦 CLI 交互层，Service 新增查询函数范围明确 |
| 模糊性 | ⚠️ | `Tabled` 实现细节（列选择）可在实现时根据实体调整 |
| 依赖 | ✅ | 依赖 crate（tokio、tabled、clap、serde）均已列出 |
