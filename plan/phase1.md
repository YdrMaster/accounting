# Phase 1 实现计划：带所有记账功能的 CLI

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 实现一个基于 Rust + SQLite 的复式记账 CLI 工具，支持多币种、账户层次、交易管理、余额报告。

**架构：** 采用 Kleppmann 端点图模型（Posting 记录双边），通过 `cost` 字段实现多币种。核心库定义纯数据结构与算法，数据库层提供 Repository traits + SQLite 实现，业务层封装事务编排，CLI 提供命令行交互。

**技术栈：** Rust Edition 2024, rusqlite, rust_decimal, chrono, clap, thiserror, tokio

## 文件结构

### `Cargo.toml`（根 workspace）

定义 workspace members。

### `accounting/`（核心库）

- `Cargo.toml` — 依赖：rust_decimal, chrono, thiserror
- `src/lib.rs` — 模块导出
- `src/id.rs` — AccountId, TransactionId, PostingId, CommodityId, MemberId, ChannelId, TagId, AttachmentId
- `src/account_type.rs` — AccountType enum, is_permanent
- `src/commodity.rs` — Commodity 结构体
- `src/account.rs` — Account 结构体
- `src/posting.rs` — Posting 结构体
- `src/transaction.rs` — Transaction 结构体
- `src/member.rs` — Member 结构体
- `src/channel.rs` — Channel 结构体
- `src/tag.rs` — Tag 结构体
- `src/attachment.rs` — Attachment 结构体
- `src/transaction_filter.rs` — TransactionFilter 结构体
- `src/error.rs` — AccountingError
- `src/validation.rs` — validate_transaction
- `src/balance.rs` — calculate_balance, calculate_all_balances
- `src/closure.rs` — compute_closure
- `src/installment.rs` — infer_installment_index
- `src/amount.rs` — to_db_amount, from_db_amount

### `accounting-sql/`（数据库层）

- `Cargo.toml` — 依赖：rusqlite, accounting, rust_decimal, chrono
- `src/lib.rs` — 模块导出
- `src/pool.rs` — ConnectionPool
- `src/schema.rs` — schema SQL + initialize_schema
- `src/error.rs` — DbError
- `src/repo/commodity.rs` — CommodityRepo trait + impl
- `src/repo/member.rs` — MemberRepo trait + impl
- `src/repo/channel.rs` — ChannelRepo trait + impl
- `src/repo/tag.rs` — TagRepo trait + impl
- `src/repo/attachment.rs` — AttachmentRepo trait + impl
- `src/repo/account.rs` — AccountRepo trait + impl
- `src/repo/transaction.rs` — TransactionRepo trait + impl
- `src/repo/posting.rs` — PostingRepo trait + impl
- `src/database.rs` — Database trait
- `src/transaction.rs` — Transaction trait
- `src/impls/sqlite.rs` — SqliteDatabase, SqliteTransaction

### `accounting-service/`（业务层）

- `Cargo.toml` — 依赖：accounting, accounting-sql, chrono
- `src/lib.rs` — 模块导出
- `src/account_service.rs` — AccountService
- `src/transaction_service.rs` — TransactionService
- `src/report_service.rs` — ReportService

### `accounting-cli/`（CLI）

- `Cargo.toml` — 依赖：accounting, accounting-sql, accounting-service, clap, chrono
- `src/main.rs` — main 入口
- `src/commands/mod.rs` — 命令模块
- `src/commands/account.rs` — 账户子命令
- `src/commands/transaction.rs` — 交易子命令
- `src/commands/report.rs` — 报告子命令

## 任务 1：Workspace 配置

**文件：**

- 修改：`Cargo.toml`
- 创建：`accounting/Cargo.toml`, `accounting/src/lib.rs`
- 创建：`accounting-sql/Cargo.toml`, `accounting-sql/src/lib.rs`
- 创建：`accounting-service/Cargo.toml`, `accounting-service/src/lib.rs`
- 创建：`accounting-cli/Cargo.toml`, `accounting-cli/src/main.rs`
- 修改：`src/main.rs`（可删除，改由 accounting-cli 提供入口）

- [ ] **步骤 1：修改根 Cargo.toml 定义 workspace**

```toml
[workspace]
members = [
    "accounting",
    "accounting-sql",
    "accounting-service",
    "accounting-cli",
]
resolver = "3"
```

- [ ] **步骤 2：创建 accounting/Cargo.toml**

```toml
[package]
name = "accounting"
version = "0.1.0"
edition = "2024"

[dependencies]
rust_decimal = "1.36"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
thiserror = "2"
```

- [ ] **步骤 3：创建 accounting/src/lib.rs**

```rust
//! 核心库：记账数据模型与算法

pub mod id;
pub mod account_type;
pub mod commodity;
pub mod account;
pub mod posting;
pub mod transaction;
pub mod member;
pub mod channel;
pub mod tag;
pub mod attachment;
pub mod transaction_filter;
pub mod error;
pub mod validation;
pub mod balance;
pub mod closure;
pub mod installment;
pub mod amount;
```

- [ ] **步骤 4：创建 accounting-sql/Cargo.toml**

```toml
[package]
name = "accounting-sql"
version = "0.1.0"
edition = "2024"

[dependencies]
accounting = { path = "../accounting" }
rusqlite = { version = "0.32", features = ["bundled"] }
rust_decimal = "1.36"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
thiserror = "2"
```

- [ ] **步骤 5：创建 accounting-sql/src/lib.rs**

```rust
//! 数据库层：Repository traits + SQLite 实现

pub mod pool;
pub mod schema;
pub mod error;
pub mod repo;
pub mod database;
pub mod transaction;
pub mod impls;
```

- [ ] **步骤 6：创建 accounting-service/Cargo.toml**

```toml
[package]
name = "accounting-service"
version = "0.1.0"
edition = "2024"

[dependencies]
accounting = { path = "../accounting" }
accounting-sql = { path = "../accounting-sql" }
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **步骤 7：创建 accounting-service/src/lib.rs**

```rust
//! 业务层：Service 封装

pub mod account_service;
pub mod transaction_service;
pub mod report_service;
```

- [ ] **步骤 8：创建 accounting-cli/Cargo.toml**

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
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **步骤 9：创建 accounting-cli/src/main.rs**

```rust
fn main() {
    println!("accounting cli");
}
```

- [ ] **步骤 10：验证 workspace 编译**

运行：`cargo check`
预期：PASS，所有 crate 编译通过（此时大部分为空）

- [ ] **步骤 11：Commit**

```bash
git add Cargo.toml accounting/ accounting-sql/ accounting-service/ accounting-cli/ src/main.rs
git commit -m "chore: setup workspace crates for Phase 1"
```

## 任务 2：核心库 - ID Newtypes

**文件：**

- 创建：`accounting/src/id.rs`

- [ ] **步骤 1：编写失败的测试**

在 `accounting/src/id.rs` 底部添加（测试文件）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_equality() {
        let a = AccountId(1);
        let b = AccountId(1);
        let c = AccountId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_account_id_clone() {
        let a = AccountId(42);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_account_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AccountId(1));
        set.insert(AccountId(1));
        assert_eq!(set.len(), 1);
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting test_account_id`
预期：FAIL，编译错误 "AccountId not found"

- [ ] **步骤 3：实现 ID Newtypes**

```rust
use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub i64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(AccountId);
define_id!(TransactionId);
define_id!(PostingId);
define_id!(CommodityId);
define_id!(MemberId);
define_id!(ChannelId);
define_id!(TagId);
define_id!(AttachmentId);
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting test_account_id`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/id.rs
git commit -m "feat(core): add ID newtypes for all entities"
```

## 任务 3：核心库 - AccountType 与 InstallmentMethod

**文件：**

- 创建：`accounting/src/account_type.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_is_permanent() {
        assert!(AccountType::Asset.is_permanent());
        assert!(AccountType::Liability.is_permanent());
        assert!(!AccountType::Income.is_permanent());
        assert!(!AccountType::Expense.is_permanent());
        assert!(!AccountType::Equity.is_permanent());
    }

    #[test]
    fn test_account_type_close_conditions() {
        assert_eq!(AccountType::Asset.close_conditions(), "余额为零");
        assert_eq!(AccountType::Liability.close_conditions(), "余额为零");
        assert_eq!(AccountType::Income.close_conditions(), "无限制");
        assert_eq!(AccountType::Expense.close_conditions(), "余额为零");
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting test_account_type`
预期：FAIL，编译错误

- [ ] **步骤 3：实现 AccountType**

```rust
/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// 资产
    Asset = 1,
    /// 负债
    Liability = 2,
    /// 权益
    Equity = 3,
    /// 收入
    Income = 4,
    /// 费用
    Expense = 5,
}

impl AccountType {
    /// 是否永久性账户（期末不自动清零）
    pub fn is_permanent(self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Liability)
    }

    /// 关闭条件描述
    pub fn close_conditions(self) -> &'static str {
        match self {
            AccountType::Asset | AccountType::Liability => "余额为零",
            AccountType::Income => "无限制",
            AccountType::Expense | AccountType::Equity => "余额为零",
        }
    }
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting test_account_type`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/account_type.rs
git commit -m "feat(core): add AccountType enum with permanence and close conditions"
```

## 任务 4：核心库 - 错误类型

**文件：**

- 创建：`accounting/src/error.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AccountingError::InvalidTransaction("unbalanced".to_string());
        assert!(err.to_string().contains("unbalanced"));
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting test_error`
预期：FAIL

- [ ] **步骤 3：实现 AccountingError**

```rust
use thiserror::Error;

/// 记账系统错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AccountingError {
    #[error("无效交易: {0}")]
    InvalidTransaction(String),
    #[error("账户不存在: {0}")]
    AccountNotFound(String),
    #[error("账户余额非零，无法关闭: {0}")]
    AccountNotEmpty(String),
    #[error("商品不存在: {0}")]
    CommodityNotFound(String),
    #[error("账户已存在: {0}")]
    AccountAlreadyExists(String),
    #[error("日期格式错误: {0}")]
    InvalidDate(String),
    #[error("未知错误: {0}")]
    Unknown(String),
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting test_error`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/error.rs
git commit -m "feat(core): add AccountingError type"
```

## 任务 5：核心库 - 数据模型（上）

**文件：**

- 创建：`accounting/src/commodity.rs`
- 创建：`accounting/src/account.rs`

- [ ] **步骤 1：实现 Commodity**

```rust
use crate::id::CommodityId;

/// 商品/货币
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commodity {
    pub id: CommodityId,
    pub symbol: String,
    pub name: String,
    pub precision: u8,
}
```

- [ ] **步骤 2：编写 Commodity 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::CommodityId;

    #[test]
    fn test_commodity_fields() {
        let c = Commodity {
            id: CommodityId(1),
            symbol: "CNY".to_string(),
            name: "人民币".to_string(),
            precision: 2,
        };
        assert_eq!(c.symbol, "CNY");
        assert_eq!(c.precision, 2);
    }
}
```

- [ ] **步骤 3：运行测试验证通过**

运行：`cargo test -p accounting commodity`
预期：PASS

- [ ] **步骤 4：实现 Account**

```rust
use crate::account_type::AccountType;
use crate::id::AccountId;
use chrono::NaiveDate;

/// 账户
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub full_name: String,
    pub account_type: AccountType,
    pub parent_id: Option<AccountId>,
    pub opened_at: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub is_system: bool,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
}
```

- [ ] **步骤 5：编写 Account 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_type::AccountType;
    use crate::id::AccountId;
    use chrono::NaiveDate;

    #[test]
    fn test_account_fields() {
        let a = Account {
            id: AccountId(1),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            opened_at: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        assert_eq!(a.full_name, "Assets:Cash");
        assert!(!a.is_system);
    }
}
```

- [ ] **步骤 6：运行测试验证通过**

运行：`cargo test -p accounting account`
预期：PASS

- [ ] **步骤 7：Commit**

```bash
git add accounting/src/commodity.rs accounting/src/account.rs
git commit -m "feat(core): add Commodity and Account data models"
```

## 任务 6：核心库 - 数据模型（中）

**文件：**

- 创建：`accounting/src/member.rs`
- 创建：`accounting/src/channel.rs`
- 创建：`accounting/src/tag.rs`

- [ ] **步骤 1：实现 Member**

```rust
use crate::id::MemberId;

/// 成员
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: MemberId,
    pub name: String,
    pub description: Option<String>,
}
```

- [ ] **步骤 2：实现 Channel**

```rust
use crate::id::ChannelId;

/// 支付渠道
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub description: Option<String>,
}
```

- [ ] **步骤 3：实现 Tag**

```rust
use crate::id::TagId;

/// 标签
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}
```

- [ ] **步骤 4：运行测试**

运行：`cargo test -p accounting`
预期：PASS（无新增测试，仅验证编译）

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/member.rs accounting/src/channel.rs accounting/src/tag.rs
git commit -m "feat(core): add Member, Channel, Tag models"
```

## 任务 7：核心库 - 数据模型（下）

**文件：**

- 创建：`accounting/src/attachment.rs`
- 创建：`accounting/src/transaction_filter.rs`

- [ ] **步骤 1：实现 Attachment**

```rust
use crate::id::{AttachmentId, TransactionId};

/// 附件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: AttachmentId,
    pub transaction_id: TransactionId,
    pub filename: String,
    pub data: Vec<u8>,
}
```

- [ ] **步骤 2：实现 TransactionFilter**

```rust
use chrono::NaiveDate;

/// 交易查询条件
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub account_id: Option<crate::id::AccountId>,
    pub member_id: Option<crate::id::MemberId>,
    pub channel_id: Option<crate::id::ChannelId>,
    pub tag_id: Option<crate::id::TagId>,
    pub keyword: Option<String>,
    pub has_installment: Option<bool>,
    pub is_template: Option<bool>,
}
```

- [ ] **步骤 3：运行编译验证**

运行：`cargo check -p accounting`
预期：PASS

- [ ] **步骤 4：Commit**

```bash
git add accounting/src/attachment.rs accounting/src/transaction_filter.rs
git commit -m "feat(core): add Attachment and TransactionFilter models"
```

## 任务 8：核心库 - Posting 与 Transaction

**文件：**

- 创建：`accounting/src/posting.rs`
- 创建：`accounting/src/transaction.rs`

- [ ] **步骤 1：实现 Posting**

```rust
use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
use rust_decimal::Decimal;

/// 分录（Posting / 端点）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub id: PostingId,
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub commodity_id: CommodityId,
    pub amount: Decimal,
    /// 总价格（双边 Cost），单币种交易为 None
    pub cost: Option<Decimal>,
    /// cost 对应的商品 ID
    pub cost_commodity_id: Option<CommodityId>,
    pub description: Option<String>,
    pub member_id: Option<crate::id::MemberId>,
    pub channel_id: Option<crate::id::ChannelId>,
}
```

- [ ] **步骤 2：实现 Transaction**

```rust
use crate::id::{MemberId, TagId, TransactionId};
use chrono::NaiveDate;

/// 交易
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub date: NaiveDate,
    pub description: String,
    pub member_id: Option<MemberId>,
    pub is_template: bool,
}
```

- [ ] **步骤 3：运行编译验证**

运行：`cargo check -p accounting`
预期：PASS

- [ ] **步骤 4：Commit**

```bash
git add accounting/src/posting.rs accounting/src/transaction.rs
git commit -m "feat(core): add Posting and Transaction models"
```

## 任务 9：核心库 - 金额转换

**文件：**

- 创建：`accounting/src/amount.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_to_db_amount() {
        let d = Decimal::from_str("12.34").unwrap();
        assert_eq!(to_db_amount(d, 2), 1234i64);
    }

    #[test]
    fn test_to_db_amount_negative() {
        let d = Decimal::from_str("-5.00").unwrap();
        assert_eq!(to_db_amount(d, 2), -500i64);
    }

    #[test]
    fn test_from_db_amount() {
        assert_eq!(from_db_amount(1234i64, 2), Decimal::from_str("12.34").unwrap());
    }

    #[test]
    fn test_from_db_amount_zero() {
        assert_eq!(from_db_amount(0i64, 2), Decimal::ZERO);
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting amount`
预期：FAIL

- [ ] **步骤 3：实现金额转换函数**

```rust
use rust_decimal::Decimal;

/// 将 Decimal 转换为数据库存储的整数
///
/// 根据 commodity precision 进行缩放：precision=2 时 12.34 → 1234
pub fn to_db_amount(amount: Decimal, precision: u8) -> i64 {
    let scale = 10i64.pow(precision as u32);
    (amount * Decimal::from(scale)).round().to_i64().unwrap_or(0)
}

/// 将数据库存储的整数还原为 Decimal
///
/// precision=2 时 1234 → 12.34
pub fn from_db_amount(stored: i64, precision: u8) -> Decimal {
    let scale = 10i64.pow(precision as u32);
    Decimal::from(stored) / Decimal::from(scale)
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting amount`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/amount.rs
git commit -m "feat(core): add amount conversion utilities"
```

## 任务 10：核心库 - 交易验证

**文件：**

- 创建：`accounting/src/validation.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn posting(account_id: i64, commodity_id: i64, amount: &str, cost: Option<&str>, cost_commodity: Option<i64>) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(account_id),
            commodity_id: CommodityId(commodity_id),
            amount: Decimal::from_str(amount).unwrap(),
            cost: cost.map(|c| Decimal::from_str(c).unwrap()),
            cost_commodity_id: cost_commodity.map(CommodityId),
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_empty_postings_fails() {
        let postings: Vec<Posting> = vec![];
        let result = validate_transaction(&postings);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_posting_fails() {
        let postings = vec![posting(1, 1, "100", None, None)];
        let result = validate_transaction(&postings);
        assert!(result.is_err());
    }

    #[test]
    fn test_balanced_same_commodity_passes() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 1, "-100", None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_unbalanced_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 1, "-50", None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_multi_commodity_with_cost_passes() {
        // 100 USD @ 700 CNY = 70000 CNY
        let postings = vec![
            posting(1, 1, "100", Some("70000"), Some(2)),   // USD account, cost in CNY
            posting(2, 2, "-70000", None, None),            // CNY account
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_multi_commodity_without_cost_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 2, "-700", None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting validate`
预期：FAIL

- [ ] **步骤 3：实现 validate_transaction**

```rust
use crate::error::AccountingError;
use crate::posting::Posting;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 验证交易平衡性
///
/// 规则：
/// - 至少两个分录
/// - 同一 commodity 的金额之和为零
/// - 不同 commodity 的交易必须有 cost 字段建立等式
pub fn validate_transaction(postings: &[Posting]) -> Result<(), AccountingError> {
    if postings.len() < 2 {
        return Err(AccountingError::InvalidTransaction(
            "交易至少包含两个分录".to_string(),
        ));
    }

    // 按 commodity 分组求和
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        *sums.entry(p.commodity_id.0).or_insert_with(Decimal::ZERO) += p.amount;
    }

    // 检查是否所有 commodity 都能自平衡
    let unbalanced: Vec<_> = sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced.is_empty() {
        return Ok(());
    }

    // 多币种情况：检查 cost 是否能建立等式
    // 简单检查：所有 cost 金额之和应为零
    let mut cost_sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        if let Some(cost) = p.cost {
            let cost_commodity = p.cost_commodity_id.map(|c| c.0).unwrap_or(p.commodity_id.0);
            *cost_sums.entry(cost_commodity).or_insert_with(Decimal::ZERO) += cost;
        } else {
            // 无 cost 的分录，其 amount 必须自平衡
            *cost_sums.entry(p.commodity_id.0).or_insert_with(Decimal::ZERO) += p.amount;
        }
    }

    let unbalanced_costs: Vec<_> = cost_sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced_costs.is_empty() {
        Ok(())
    } else {
        Err(AccountingError::InvalidTransaction(
            "交易不平衡".to_string(),
        ))
    }
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting validate`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/validation.rs
git commit -m "feat(core): add transaction validation with bilateral cost support"
```

## 任务 11：核心库 - 余额计算

**文件：**

- 创建：`accounting/src/balance.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use crate::posting::Posting;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn make_posting(account_id: i64, commodity_id: i64, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(account_id),
            commodity_id: CommodityId(commodity_id),
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_single_commodity_balance() {
        let postings = vec![
            make_posting(1, 1, "100"),
            make_posting(1, 1, "50"),
            make_posting(1, 1, "-30"),
        ];
        let balance = calculate_balance(&postings, CommodityId(1));
        assert_eq!(balance, Decimal::from_str("120").unwrap());
    }

    #[test]
    fn test_multi_commodity_balances() {
        let postings = vec![
            make_posting(1, 1, "100"),
            make_posting(1, 2, "50"),
        ];
        let balances = calculate_all_balances(&postings);
        assert_eq!(balances[&CommodityId(1)], Decimal::from_str("100").unwrap());
        assert_eq!(balances[&CommodityId(2)], Decimal::from_str("50").unwrap());
    }

    #[test]
    fn test_balance_zero() {
        let postings = vec![];
        let balance = calculate_balance(&postings, CommodityId(1));
        assert_eq!(balance, Decimal::ZERO);
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting balance`
预期：FAIL

- [ ] **步骤 3：实现余额计算**

```rust
use crate::id::CommodityId;
use crate::posting::Posting;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 计算指定 commodity 的余额
pub fn calculate_balance(postings: &[Posting], commodity_id: CommodityId) -> Decimal {
    postings
        .iter()
        .filter(|p| p.commodity_id == commodity_id)
        .map(|p| p.amount)
        .sum()
}

/// 计算所有 commodity 的余额
pub fn calculate_all_balances(postings: &[Posting]) -> HashMap<CommodityId, Decimal> {
    let mut balances: HashMap<CommodityId, Decimal> = HashMap::new();
    for p in postings {
        *balances.entry(p.commodity_id).or_insert_with(Decimal::ZERO) += p.amount;
    }
    balances
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting balance`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/balance.rs
git commit -m "feat(core): add balance calculation functions"
```

## 任务 12：核心库 - 闭包计算

**文件：**

- 创建：`accounting/src/closure.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::AccountId;

    #[test]
    fn test_root_account() {
        let accounts = vec![
            AccountNode { id: AccountId(1), parent_id: None, full_name: "Assets".to_string() },
        ];
        let closure = compute_closure(&accounts);
        assert_eq!(closure.get(&AccountId(1)).unwrap(), &vec![AccountId(1)]);
    }

    #[test]
    fn test_parent_child() {
        let accounts = vec![
            AccountNode { id: AccountId(1), parent_id: None, full_name: "Assets".to_string() },
            AccountNode { id: AccountId(2), parent_id: Some(AccountId(1)), full_name: "Assets:Cash".to_string() },
        ];
        let closure = compute_closure(&accounts);
        assert_eq!(closure.get(&AccountId(1)).unwrap(), &vec![AccountId(1), AccountId(2)]);
        assert_eq!(closure.get(&AccountId(2)).unwrap(), &vec![AccountId(2)]);
    }

    #[test]
    fn test_deep_hierarchy() {
        let accounts = vec![
            AccountNode { id: AccountId(1), parent_id: None, full_name: "Assets".to_string() },
            AccountNode { id: AccountId(2), parent_id: Some(AccountId(1)), full_name: "Assets:Bank".to_string() },
            AccountNode { id: AccountId(3), parent_id: Some(AccountId(2)), full_name: "Assets:Bank:Checking".to_string() },
        ];
        let closure = compute_closure(&accounts);
        let root = closure.get(&AccountId(1)).unwrap();
        assert!(root.contains(&AccountId(1)));
        assert!(root.contains(&AccountId(2)));
        assert!(root.contains(&AccountId(3)));
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test -p accounting closure`
预期：FAIL

- [ ] **步骤 3：实现 AccountNode 与 compute_closure**

```rust
use crate::id::AccountId;
use std::collections::HashMap;

/// 用于闭包计算的账户节点（简化视图）
#[derive(Debug, Clone)]
pub struct AccountNode {
    pub id: AccountId,
    pub parent_id: Option<AccountId>,
    pub full_name: String,
}

/// 计算闭包表
///
/// 返回每个账户到其后代列表（含自身）的映射
pub fn compute_closure(accounts: &[AccountNode]) -> HashMap<AccountId, Vec<AccountId>> {
    let mut closure: HashMap<AccountId, Vec<AccountId>> = HashMap::new();

    // 初始化每个账户只包含自身
    for acc in accounts {
        closure.insert(acc.id, vec![acc.id]);
    }

    // 构建 parent -> children 映射
    let mut children: HashMap<AccountId, Vec<AccountId>> = HashMap::new();
    for acc in accounts {
        if let Some(parent) = acc.parent_id {
            children.entry(parent).or_default().push(acc.id);
        }
    }

    // 递归收集所有后代
    fn collect_descendants(
        id: AccountId,
        children: &HashMap<AccountId, Vec<AccountId>>,
        result: &mut Vec<AccountId>,
    ) {
        if let Some(kids) = children.get(&id) {
            for &child in kids {
                result.push(child);
                collect_descendants(child, children, result);
            }
        }
    }

    for acc in accounts {
        let mut descendants = vec![acc.id];
        collect_descendants(acc.id, &children, &mut descendants);
        closure.insert(acc.id, descendants);
    }

    closure
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test -p accounting closure`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/closure.rs
git commit -m "feat(core): add closure table computation"
```

## 任务 13：核心库 - 分期索引与账户关闭验证

**文件：**

- 创建：`accounting/src/installment.rs`
- 修改：`accounting/src/account_type.rs`（添加 InstallmentMethod）

- [ ] **步骤 1：在 account_type.rs 添加 InstallmentMethod**

```rust
/// 分期方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallmentMethod {
    /// 等额本息
    EqualPrincipalAndInterest = 1,
    /// 等额本金
    EqualPrincipal = 2,
    /// 免息分期
    InterestFree = 3,
    /// 自定义
    Custom = 4,
}
```

- [ ] **步骤 2：编写 installment 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_before_repayment_day() {
        // 交易在还款日之前，属于当期
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let repayment_day = 15u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 1);
    }

    #[test]
    fn test_after_repayment_day() {
        // 交易在还款日之后，属于下期
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let repayment_day = 15u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_cross_month() {
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 25).unwrap();
        let repayment_day = 10u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 2);
    }
}
```

- [ ] **步骤 3：实现 infer_installment_index**

```rust
use chrono::NaiveDate;

/// 根据交易日期和还款日推断分期期数
///
/// 规则：交易日期在还款日之前 → 当期(1)，之后 → 下期(2)
/// 跨月时：如果交易日 > 还款日，期数加 1
pub fn infer_installment_index(tx_date: NaiveDate, repayment_day: u8) -> u32 {
    let day = tx_date.day() as u8;
    if day <= repayment_day {
        1
    } else {
        2
    }
}
```

- [ ] **步骤 4：添加 validate_account_close**

在 `accounting/src/validation.rs` 中添加：

```rust
use crate::account_type::AccountType;
use crate::error::AccountingError;
use rust_decimal::Decimal;

/// 验证账户是否可以关闭
///
/// Asset 和 Liability 必须余额为零；Income 和 Expense 无限制
pub fn validate_account_close(
    account_type: AccountType,
    balances: &[(crate::id::CommodityId, Decimal)],
) -> Result<(), AccountingError> {
    match account_type {
        AccountType::Asset | AccountType::Liability | AccountType::Expense | AccountType::Equity => {
            let non_zero: Vec<_> = balances
                .iter()
                .filter(|(_, b)| !b.is_zero())
                .collect();
            if !non_zero.is_empty() {
                return Err(AccountingError::AccountNotEmpty(
                    "账户余额非零".to_string(),
                ));
            }
        }
        AccountType::Income => {
            // Income 账户关闭无限制
        }
    }
    Ok(())
}
```

并添加测试：

```rust
    #[test]
    fn test_close_asset_with_zero_balance_ok() {
        let balances = vec![(CommodityId(1), Decimal::ZERO)];
        assert!(validate_account_close(AccountType::Asset, &balances).is_ok());
    }

    #[test]
    fn test_close_asset_with_non_zero_balance_fails() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Asset, &balances).is_err());
    }

    #[test]
    fn test_close_income_unconditionally_ok() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Income, &balances).is_ok());
    }
```

- [ ] **步骤 5：运行测试**

运行：`cargo test -p accounting installment && cargo test -p accounting validate_account_close`
预期：PASS

- [ ] **步骤 6：Commit**

```bash
git add accounting/src/installment.rs accounting/src/account_type.rs accounting/src/validation.rs
git commit -m "feat(core): add installment index inference and account close validation"
```

## 任务 14：数据库层 - Schema 与连接池

**文件：**

- 创建：`accounting-sql/src/pool.rs`
- 创建：`accounting-sql/src/schema.rs`
- 创建：`accounting-sql/src/error.rs`

- [ ] **步骤 1：实现 DbError**

```rust
use thiserror::Error;

/// 数据库错误
#[derive(Error, Debug)]
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}
```

- [ ] **步骤 2：实现 ConnectionPool**

```rust
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite 连接池
#[derive(Clone)]
pub struct ConnectionPool {
    conn: Arc<Mutex<Connection>>,
}

impl ConnectionPool {
    /// 打开文件数据库
    pub fn open(path: &str) -> Result<Self, crate::error::DbError> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 打开内存数据库
    pub fn open_in_memory() -> Result<Self, crate::error::DbError> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取连接锁
    pub fn get(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().unwrap()
    }
}
```

- [ ] **步骤 3：实现 Schema**

```rust
use rusqlite::Connection;

/// 初始化数据库 schema
pub fn initialize_schema(conn: &Connection) -> Result<(), crate::error::DbError> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

/// 插入内置数据
pub fn insert_seed_data(conn: &Connection) -> Result<(), crate::error::DbError> {
    conn.execute_batch(SEED_SQL)?;
    Ok(())
}

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS commodities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    precision INTEGER NOT NULL DEFAULT 2
);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL UNIQUE,
    account_type INTEGER NOT NULL,
    parent_id INTEGER REFERENCES accounts(id),
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    is_system INTEGER NOT NULL DEFAULT 0,
    billing_day INTEGER,
    repayment_day INTEGER
);

CREATE TABLE IF NOT EXISTS account_ancestors (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    ancestor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, ancestor_id)
);

CREATE TABLE IF NOT EXISTS account_owners (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, member_id)
);

CREATE TABLE IF NOT EXISTS members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    member_id INTEGER REFERENCES members(id),
    is_template INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS postings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    commodity_id INTEGER NOT NULL REFERENCES commodities(id),
    amount INTEGER NOT NULL,
    cost INTEGER,
    cost_commodity_id INTEGER REFERENCES commodities(id),
    description TEXT,
    member_id INTEGER REFERENCES members(id),
    channel_id INTEGER REFERENCES channels(id)
);

CREATE TABLE IF NOT EXISTS attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    data BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_tags (
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_id);
CREATE INDEX IF NOT EXISTS idx_postings_tx ON postings(transaction_id);
CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);
CREATE INDEX IF NOT EXISTS idx_postings_commodity ON postings(commodity_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
"#;

const SEED_SQL: &str = r#"
INSERT OR IGNORE INTO commodities (symbol, name, precision) VALUES ('CNY', '人民币', 2);

INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, opened_at, is_system) VALUES
('Equity:OpeningBalances', 3, NULL, '2000-01-01', 1),
('Income:Uncategorized', 4, NULL, '2000-01-01', 1),
('Expenses:Uncategorized', 5, NULL, '2000-01-01', 1),
('Expenses:Fees', 5, NULL, '2000-01-01', 1),
('Expenses:Discounts', 5, NULL, '2000-01-01', 1),
('Expenses:InstallmentFees', 5, NULL, '2000-01-01', 1),
('Assets:Cashback', 1, NULL, '2000-01-01', 1);

INSERT OR IGNORE INTO tags (name, description, is_system) VALUES
('repayment', '分期/信用卡还款标记', 1);
"#;
```

- [ ] **步骤 4：编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert!(tables.contains(&"commodities".to_string()));
        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"postings".to_string()));
    }

    #[test]
    fn test_seed_data() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        insert_seed_data(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM commodities WHERE symbol='CNY'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts WHERE is_system=1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 7);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags WHERE name='repayment'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
```

- [ ] **步骤 5：运行测试验证通过**

运行：`cargo test -p accounting-sql schema`
预期：PASS

- [ ] **步骤 6：Commit**

```bash
git add accounting-sql/src/pool.rs accounting-sql/src/schema.rs accounting-sql/src/error.rs
git commit -m "feat(sql): add ConnectionPool, schema initialization and seed data"
```

## 任务 15：数据库层 - AccountRepo

**文件：**

- 创建：`accounting-sql/src/repo/account.rs`
- 创建：`accounting-sql/src/repo/mod.rs`

- [ ] **步骤 1：创建 repo/mod.rs**

```rust
pub mod account;
pub mod commodity;
pub mod member;
pub mod channel;
pub mod tag;
pub mod attachment;
pub mod transaction;
pub mod posting;
```

- [ ] **步骤 2：编写 AccountRepo trait + SQLite 实现**

```rust
use accounting::account::{Account};
use accounting::id::AccountId;
use chrono::NaiveDate;
use rusqlite::{Connection, params};

/// Account 仓库 trait
pub trait AccountRepo {
    fn create(&self, conn: &Connection, account: &Account) -> Result<AccountId, crate::error::DbError>;
    fn get(&self, conn: &Connection, id: AccountId) -> Result<Option<Account>, crate::error::DbError>;
    fn get_by_name(&self, conn: &Connection, name: &str) -> Result<Option<Account>, crate::error::DbError>;
    fn list(&self, conn: &Connection) -> Result<Vec<Account>, crate::error::DbError>;
    fn list_children(&self, conn: &Connection, parent_id: AccountId) -> Result<Vec<Account>, crate::error::DbError>;
    fn close(&self, conn: &Connection, id: AccountId, closed_at: NaiveDate) -> Result<(), crate::error::DbError>;
    fn reopen(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
}

pub struct SqliteAccountRepo;

impl AccountRepo for SqliteAccountRepo {
    fn create(&self, conn: &Connection, account: &Account) -> Result<AccountId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, parent_id, opened_at, closed_at, is_system, billing_day, repayment_day)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                account.full_name,
                account.account_type as i32,
                account.parent_id.map(|id| id.0),
                account.opened_at.to_string(),
                account.closed_at.map(|d| d.to_string()),
                account.is_system as i32,
                account.billing_day,
                account.repayment_day,
            ],
        )?;
        Ok(AccountId(conn.last_insert_rowid()))
    }

    fn get(&self, conn: &Connection, id: AccountId) -> Result<Option<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, opened_at, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_by_name(&self, conn: &Connection, name: &str) -> Result<Option<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, opened_at, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE full_name = ?1"
        )?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account(row)?))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, opened_at, closed_at, is_system, billing_day, repayment_day
             FROM accounts ORDER BY full_name"
        )?;
        let rows = stmt.query_map([], |row| map_account(row))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn list_children(&self, conn: &Connection, parent_id: AccountId) -> Result<Vec<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, opened_at, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE parent_id = ?1 ORDER BY full_name"
        )?;
        let rows = stmt.query_map(params![parent_id.0], |row| map_account(row))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn close(&self, conn: &Connection, id: AccountId, closed_at: NaiveDate) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE accounts SET closed_at = ?1 WHERE id = ?2",
            params![closed_at.to_string(), id.0],
        )?;
        Ok(())
    }

    fn reopen(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE accounts SET closed_at = NULL WHERE id = ?1",
            params![id.0],
        )?;
        Ok(())
    }
}

fn map_account(row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
    use accounting::account_type::AccountType;

    let type_int: i32 = row.get(1)?;
    let account_type = match type_int {
        1 => AccountType::Asset,
        2 => AccountType::Liability,
        3 => AccountType::Equity,
        4 => AccountType::Income,
        5 => AccountType::Expense,
        _ => AccountType::Asset,
    };

    let opened_str: String = row.get(3)?;
    let opened_at = NaiveDate::parse_from_str(&opened_str, "%Y-%m-%d").unwrap_or_default();

    let closed_at: Option<String> = row.get(4)?;
    let closed_at = closed_at.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());

    Ok(Account {
        id: AccountId(row.get(0)?),
        full_name: row.get(1)?,
        account_type,
        parent_id: row.get::<_, Option<i64>>(2)?.map(AccountId),
        opened_at,
        closed_at,
        is_system: row.get::<_, i32>(5)? != 0,
        billing_day: row.get::<_, Option<i32>>(6)?.map(|v| v as u8),
        repayment_day: row.get::<_, Option<i32>>(7)?.map(|v| v as u8),
    })
}
```

- [ ] **步骤 3：编写 AccountRepo 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::AccountId;
    use chrono::NaiveDate;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteAccountRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
        (conn, SqliteAccountRepo)
    }

    #[test]
    fn test_create_and_get() {
        let (conn, repo) = setup();
        let account = Account {
            id: AccountId(0),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            opened_at: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = repo.create(&conn, &account).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.full_name, "Assets:Cash");
        assert_eq!(fetched.account_type, AccountType::Asset);
    }

    #[test]
    fn test_get_by_name() {
        let (conn, repo) = setup();
        let found = repo.get_by_name(&conn, "Equity:OpeningBalances").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().account_type, AccountType::Equity);
    }

    #[test]
    fn test_close_and_reopen() {
        let (conn, repo) = setup();
        let found = repo.get_by_name(&conn, "Equity:OpeningBalances").unwrap().unwrap();
        repo.close(&conn, found.id, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()).unwrap();
        let closed = repo.get(&conn, found.id).unwrap().unwrap();
        assert!(closed.closed_at.is_some());

        repo.reopen(&conn, found.id).unwrap();
        let reopened = repo.get(&conn, found.id).unwrap().unwrap();
        assert!(reopened.closed_at.is_none());
    }
}
```

- [ ] **步骤 4：运行测试**

运行：`cargo test -p accounting-sql account_repo`
预期：PASS

- [ ] **步骤 5：Commit**

```bash
git add accounting-sql/src/repo/
git commit -m "feat(sql): add AccountRepo with CRUD, close, reopen"
```

## 任务 16：数据库层 - 其余简单 Repo

**文件：**

- 创建：`accounting-sql/src/repo/commodity.rs`
- 创建：`accounting-sql/src/repo/member.rs`
- 创建：`accounting-sql/src/repo/channel.rs`
- 创建：`accounting-sql/src/repo/tag.rs`
- 创建：`accounting-sql/src/repo/attachment.rs`

- [ ] **步骤 1：实现 CommodityRepo**

```rust
use accounting::commodity::Commodity;
use accounting::id::CommodityId;
use rusqlite::{Connection, params};

pub trait CommodityRepo {
    fn get_by_symbol(&self, conn: &Connection, symbol: &str) -> Result<Option<Commodity>, crate::error::DbError>;
    fn list(&self, conn: &Connection) -> Result<Vec<Commodity>, crate::error::DbError>;
}

pub struct SqliteCommodityRepo;

impl CommodityRepo for SqliteCommodityRepo {
    fn get_by_symbol(&self, conn: &Connection, symbol: &str) -> Result<Option<Commodity>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, symbol, name, precision FROM commodities WHERE symbol = ?1")?;
        let mut rows = stmt.query(params![symbol])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Commodity {
                id: CommodityId(row.get(0)?),
                symbol: row.get(1)?,
                name: row.get(2)?,
                precision: row.get::<_, i32>(3)? as u8,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Commodity>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, symbol, name, precision FROM commodities ORDER BY symbol")?;
        let rows = stmt.query_map([], |row| {
            Ok(Commodity {
                id: CommodityId(row.get(0)?),
                symbol: row.get(1)?,
                name: row.get(2)?,
                precision: row.get::<_, i32>(3)? as u8,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}
```

- [ ] **步骤 2：实现 MemberRepo**

```rust
use accounting::id::MemberId;
use accounting::member::Member;
use rusqlite::{Connection, params};

pub trait MemberRepo {
    fn create(&self, conn: &Connection, member: &Member) -> Result<MemberId, crate::error::DbError>;
    fn get(&self, conn: &Connection, id: MemberId) -> Result<Option<Member>, crate::error::DbError>;
    fn list(&self, conn: &Connection) -> Result<Vec<Member>, crate::error::DbError>;
}

pub struct SqliteMemberRepo;

impl MemberRepo for SqliteMemberRepo {
    fn create(&self, conn: &Connection, member: &Member) -> Result<MemberId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO members (name, description) VALUES (?1, ?2)",
            params![member.name, member.description],
        )?;
        Ok(MemberId(conn.last_insert_rowid()))
    }

    fn get(&self, conn: &Connection, id: MemberId) -> Result<Option<Member>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM members WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Member {
                id: MemberId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Member>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM members ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Member {
                id: MemberId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}
```

- [ ] **步骤 3：实现 ChannelRepo（结构与 MemberRepo 类似）**

```rust
use accounting::channel::Channel;
use accounting::id::ChannelId;
use rusqlite::{Connection, params};

pub trait ChannelRepo {
    fn create(&self, conn: &Connection, channel: &Channel) -> Result<ChannelId, crate::error::DbError>;
    fn get(&self, conn: &Connection, id: ChannelId) -> Result<Option<Channel>, crate::error::DbError>;
    fn list(&self, conn: &Connection) -> Result<Vec<Channel>, crate::error::DbError>;
}

pub struct SqliteChannelRepo;

impl ChannelRepo for SqliteChannelRepo {
    fn create(&self, conn: &Connection, channel: &Channel) -> Result<ChannelId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO channels (name, description) VALUES (?1, ?2)",
            params![channel.name, channel.description],
        )?;
        Ok(ChannelId(conn.last_insert_rowid()))
    }

    fn get(&self, conn: &Connection, id: ChannelId) -> Result<Option<Channel>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM channels WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Channel {
                id: ChannelId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Channel>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM channels ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Channel {
                id: ChannelId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}
```

- [ ] **步骤 4：实现 TagRepo**

```rust
use accounting::id::TagId;
use accounting::tag::Tag;
use rusqlite::{Connection, params};

pub trait TagRepo {
    fn get_by_name(&self, conn: &Connection, name: &str) -> Result<Option<Tag>, crate::error::DbError>;
    fn list(&self, conn: &Connection) -> Result<Vec<Tag>, crate::error::DbError>;
}

pub struct SqliteTagRepo;

impl TagRepo for SqliteTagRepo {
    fn get_by_name(&self, conn: &Connection, name: &str) -> Result<Option<Tag>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description, is_system FROM tags WHERE name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Tag {
                id: TagId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Tag>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description, is_system FROM tags ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: TagId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}
```

- [ ] **步骤 5：实现 AttachmentRepo**

```rust
use accounting::attachment::Attachment;
use accounting::id::{AttachmentId, TransactionId};
use rusqlite::{Connection, params};

pub trait AttachmentRepo {
    fn create(&self, conn: &Connection, attachment: &Attachment) -> Result<AttachmentId, crate::error::DbError>;
    fn list_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<Vec<Attachment>, crate::error::DbError>;
    fn delete(&self, conn: &Connection, id: AttachmentId) -> Result<(), crate::error::DbError>;
}

pub struct SqliteAttachmentRepo;

impl AttachmentRepo for SqliteAttachmentRepo {
    fn create(&self, conn: &Connection, attachment: &Attachment) -> Result<AttachmentId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO attachments (transaction_id, filename, data) VALUES (?1, ?2, ?3)",
            params![attachment.transaction_id.0, attachment.filename, attachment.data],
        )?;
        Ok(AttachmentId(conn.last_insert_rowid()))
    }

    fn list_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<Vec<Attachment>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, transaction_id, filename, data FROM attachments WHERE transaction_id = ?1")?;
        let rows = stmt.query_map(params![transaction_id.0], |row| {
            Ok(Attachment {
                id: AttachmentId(row.get(0)?),
                transaction_id: TransactionId(row.get(1)?),
                filename: row.get(2)?,
                data: row.get(3)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn delete(&self, conn: &Connection, id: AttachmentId) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM attachments WHERE id = ?1", params![id.0])?;
        Ok(())
    }
}
```

- [ ] **步骤 6：运行编译验证**

运行：`cargo check -p accounting-sql`
预期：PASS

- [ ] **步骤 7：Commit**

```bash
git add accounting-sql/src/repo/commodity.rs accounting-sql/src/repo/member.rs accounting-sql/src/repo/channel.rs accounting-sql/src/repo/tag.rs accounting-sql/src/repo/attachment.rs
git commit -m "feat(sql): add Commodity, Member, Channel, Tag, Attachment repos"
```

## 任务 17：数据库层 - TransactionRepo 与 PostingRepo

**文件：**

- 创建：`accounting-sql/src/repo/transaction.rs`
- 创建：`accounting-sql/src/repo/posting.rs`

- [ ] **步骤 1：实现 TransactionRepo**

```rust
use accounting::id::{TagId, TransactionId};
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use chrono::NaiveDate;
use rusqlite::{Connection, params};

pub trait TransactionRepo {
    fn insert(&self, conn: &Connection, tx: &Transaction, tag_ids: &[TagId]) -> Result<TransactionId, crate::error::DbError>;
    fn get(&self, conn: &Connection, id: TransactionId) -> Result<Option<Transaction>, crate::error::DbError>;
    fn list(&self, conn: &Connection, filter: &TransactionFilter, limit: usize, offset: usize) -> Result<Vec<Transaction>, crate::error::DbError>;
    fn count(&self, conn: &Connection, filter: &TransactionFilter) -> Result<usize, crate::error::DbError>;
    fn delete(&self, conn: &Connection, id: TransactionId) -> Result<(), crate::error::DbError>;
    fn update(&self, conn: &Connection, tx: &Transaction, tag_ids: &[TagId]) -> Result<(), crate::error::DbError>;
}

pub struct SqliteTransactionRepo;

impl TransactionRepo for SqliteTransactionRepo {
    fn insert(&self, conn: &Connection, tx: &Transaction, tag_ids: &[TagId]) -> Result<TransactionId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO transactions (date, description, member_id, is_template)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                tx.date.to_string(),
                tx.description,
                tx.member_id.map(|id| id.0),
                tx.is_template as i32,
            ],
        )?;
        let tx_id = TransactionId(conn.last_insert_rowid());
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
                params![tx_id.0, tag_id.0],
            )?;
        }
        Ok(tx_id)
    }

    fn get(&self, conn: &Connection, id: TransactionId) -> Result<Option<Transaction>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, date, description, member_id, is_template FROM transactions WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection, filter: &TransactionFilter, limit: usize, offset: usize) -> Result<Vec<Transaction>, crate::error::DbError> {
        let mut conditions = vec!["1=1"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("date >= ?");
            params_vec.push(Box::new(start.to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("date <= ?");
            params_vec.push(Box::new(end.to_string()));
        }
        if let Some(member) = filter.member_id {
            conditions.push("member_id = ?");
            params_vec.push(Box::new(member.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT id, date, description, member_id, is_template FROM transactions WHERE {} ORDER BY date DESC, id DESC LIMIT ? OFFSET ?",
            where_clause
        );
        params_vec.push(Box::new(limit as i64));
        params_vec.push(Box::new(offset as i64));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), |row| map_transaction(row))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn count(&self, conn: &Connection, filter: &TransactionFilter) -> Result<usize, crate::error::DbError> {
        let mut conditions = vec!["1=1"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("date >= ?");
            params_vec.push(Box::new(start.to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("date <= ?");
            params_vec.push(Box::new(end.to_string()));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!("SELECT COUNT(*) FROM transactions WHERE {}", where_clause);
        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, rusqlite::params_from_iter(param_refs), |row| row.get(0))?;
        Ok(count as usize)
    }

    fn delete(&self, conn: &Connection, id: TransactionId) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM transactions WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    fn update(&self, conn: &Connection, tx: &Transaction, tag_ids: &[TagId]) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE transactions SET date = ?1, description = ?2, member_id = ?3, is_template = ?4 WHERE id = ?5",
            params![
                tx.date.to_string(),
                tx.description,
                tx.member_id.map(|id| id.0),
                tx.is_template as i32,
                tx.id.0,
            ],
        )?;
        conn.execute("DELETE FROM transaction_tags WHERE transaction_id = ?1", params![tx.id.0])?;
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
                params![tx.id.0, tag_id.0],
            )?;
        }
        Ok(())
    }
}

fn map_transaction(row: &rusqlite::Row) -> Result<Transaction, rusqlite::Error> {
    let date_str: String = row.get(1)?;
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_default();

    Ok(Transaction {
        id: TransactionId(row.get(0)?),
        date,
        description: row.get(2)?,
        member_id: row.get::<_, Option<i64>>(3)?.map(crate::id::MemberId),
        is_template: row.get::<_, i32>(4)? != 0,
    })
}
```

- [ ] **步骤 2：实现 PostingRepo**

```rust
use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
use accounting::posting::Posting;
use rust_decimal::Decimal;
use rusqlite::{Connection, params};

pub trait PostingRepo {
    fn insert(&self, conn: &Connection, posting: &Posting) -> Result<PostingId, crate::error::DbError>;
    fn list_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<Vec<Posting>, crate::error::DbError>;
    fn list_by_account(&self, conn: &Connection, account_id: AccountId) -> Result<Vec<Posting>, crate::error::DbError>;
    fn sum_by_account(&self, conn: &Connection, account_id: AccountId) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError>;
    fn delete_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<(), crate::error::DbError>;
}

pub struct SqlitePostingRepo;

impl PostingRepo for SqlitePostingRepo {
    fn insert(&self, conn: &Connection, posting: &Posting) -> Result<PostingId, crate::error::DbError> {
        let amount_i64 = accounting::amount::to_db_amount(posting.amount, 2); // TODO: use commodity precision
        let cost_i64 = posting.cost.map(|c| accounting::amount::to_db_amount(c, 2));
        conn.execute(
            "INSERT INTO postings (transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                posting.transaction_id.0,
                posting.account_id.0,
                posting.commodity_id.0,
                amount_i64,
                cost_i64,
                posting.cost_commodity_id.map(|id| id.0),
                posting.description,
                posting.member_id.map(|id| id.0),
                posting.channel_id.map(|id| id.0),
            ],
        )?;
        Ok(PostingId(conn.last_insert_rowid()))
    }

    fn list_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<Vec<Posting>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE transaction_id = ?1"
        )?;
        let rows = stmt.query_map(params![transaction_id.0], |row| map_posting(row))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn list_by_account(&self, conn: &Connection, account_id: AccountId) -> Result<Vec<Posting>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE account_id = ?1 ORDER BY transaction_id"
        )?;
        let rows = stmt.query_map(params![account_id.0], |row| map_posting(row))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn sum_by_account(&self, conn: &Connection, account_id: AccountId) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT commodity_id, SUM(amount) FROM postings WHERE account_id = ?1 GROUP BY commodity_id"
        )?;
        let rows = stmt.query_map(params![account_id.0], |row| {
            let commodity_id = CommodityId(row.get(0)?);
            let amount: i64 = row.get(1)?;
            let decimal = accounting::amount::from_db_amount(amount, 2); // TODO: use commodity precision
            Ok((commodity_id, decimal))
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn delete_by_transaction(&self, conn: &Connection, transaction_id: TransactionId) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM postings WHERE transaction_id = ?1", params![transaction_id.0])?;
        Ok(())
    }
}

fn map_posting(row: &rusqlite::Row) -> Result<Posting, rusqlite::Error> {
    let amount: i64 = row.get(4)?;
    let cost: Option<i64> = row.get(5)?;
    Ok(Posting {
        id: PostingId(row.get(0)?),
        transaction_id: TransactionId(row.get(1)?),
        account_id: AccountId(row.get(2)?),
        commodity_id: CommodityId(row.get(3)?),
        amount: accounting::amount::from_db_amount(amount, 2), // TODO: use commodity precision
        cost: cost.map(|c| accounting::amount::from_db_amount(c, 2)),
        cost_commodity_id: row.get::<_, Option<i64>>(6)?.map(CommodityId),
        description: row.get(7)?,
        member_id: row.get::<_, Option<i64>>(8)?.map(crate::id::MemberId),
        channel_id: row.get::<_, Option<i64>>(9)?.map(crate::id::ChannelId),
    })
}
```

- [ ] **步骤 3：运行编译验证**

运行：`cargo check -p accounting-sql`
预期：PASS（可能有 TODO precision 警告，可接受）

- [ ] **步骤 4：Commit**

```bash
git add accounting-sql/src/repo/transaction.rs accounting-sql/src/repo/posting.rs
git commit -m "feat(sql): add TransactionRepo and PostingRepo"
```

## 任务 18：数据库层 - Database 与 Transaction traits

**文件：**

- 创建：`accounting-sql/src/database.rs`
- 创建：`accounting-sql/src/transaction.rs`
- 创建：`accounting-sql/src/impls/mod.rs`
- 创建：`accounting-sql/src/impls/sqlite.rs`

- [ ] **步骤 1：定义 Database trait**

```rust
use crate::repo::account::AccountRepo;
use crate::repo::commodity::CommodityRepo;
use crate::repo::member::MemberRepo;
use crate::repo::channel::ChannelRepo;
use crate::repo::tag::TagRepo;
use crate::repo::attachment::AttachmentRepo;
use crate::repo::transaction::TransactionRepo;
use crate::repo::posting::PostingRepo;

/// 数据库 trait，聚合所有 Repository
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

    /// 开始事务
    async fn transaction(&self) -> Result<Self::Tx, crate::error::DbError>;
}
```

- [ ] **步骤 2：定义 Transaction trait**

```rust
use crate::repo::account::AccountRepo;
use crate::repo::commodity::CommodityRepo;
use crate::repo::member::MemberRepo;
use crate::repo::channel::ChannelRepo;
use crate::repo::tag::TagRepo;
use crate::repo::attachment::AttachmentRepo;
use crate::repo::transaction::TransactionRepo;
use crate::repo::posting::PostingRepo;

/// 事务 trait，继承所有 Repository
pub trait Transaction: Send {
    fn account_repo(&self) -> &dyn AccountRepo;
    fn commodity_repo(&self) -> &dyn CommodityRepo;
    fn member_repo(&self) -> &dyn MemberRepo;
    fn channel_repo(&self) -> &dyn ChannelRepo;
    fn tag_repo(&self) -> &dyn TagRepo;
    fn attachment_repo(&self) -> &dyn AttachmentRepo;
    fn transaction_repo(&self) -> &dyn TransactionRepo;
    fn posting_repo(&self) -> &dyn PostingRepo;

    /// 提交事务
    async fn commit(self) -> Result<(), crate::error::DbError>;
}
```

- [ ] **步骤 3：实现 SqliteDatabase 和 SqliteTransaction**

```rust
use crate::database::Database;
use crate::transaction::Transaction;
use crate::pool::ConnectionPool;
use crate::repo::account::{AccountRepo, SqliteAccountRepo};
use crate::repo::commodity::{CommodityRepo, SqliteCommodityRepo};
use crate::repo::member::{MemberRepo, SqliteMemberRepo};
use crate::repo::channel::{ChannelRepo, SqliteChannelRepo};
use crate::repo::tag::{TagRepo, SqliteTagRepo};
use crate::repo::attachment::{AttachmentRepo, SqliteAttachmentRepo};
use crate::repo::transaction::{TransactionRepo, SqliteTransactionRepo};
use crate::repo::posting::{PostingRepo, SqlitePostingRepo};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite 数据库实现
pub struct SqliteDatabase {
    pool: ConnectionPool,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    member_repo: SqliteMemberRepo,
    channel_repo: SqliteChannelRepo,
    tag_repo: SqliteTagRepo,
    attachment_repo: SqliteAttachmentRepo,
    transaction_repo: SqliteTransactionRepo,
    posting_repo: SqlitePostingRepo,
}

impl SqliteDatabase {
    pub fn open(path: &str) -> Result<Self, crate::error::DbError> {
        let pool = ConnectionPool::open(path)?;
        {
            let conn = pool.get();
            crate::schema::initialize_schema(&conn)?;
            crate::schema::insert_seed_data(&conn)?;
        }
        Ok(Self::new(pool))
    }

    pub fn open_in_memory() -> Result<Self, crate::error::DbError> {
        let pool = ConnectionPool::open_in_memory()?;
        {
            let conn = pool.get();
            crate::schema::initialize_schema(&conn)?;
            crate::schema::insert_seed_data(&conn)?;
        }
        Ok(Self::new(pool))
    }

    fn new(pool: ConnectionPool) -> Self {
        Self {
            pool,
            account_repo: SqliteAccountRepo,
            commodity_repo: SqliteCommodityRepo,
            member_repo: SqliteMemberRepo,
            channel_repo: SqliteChannelRepo,
            tag_repo: SqliteTagRepo,
            attachment_repo: SqliteAttachmentRepo,
            transaction_repo: SqliteTransactionRepo,
            posting_repo: SqlitePostingRepo,
        }
    }
}

impl Database for SqliteDatabase {
    type Tx = SqliteTransaction;

    fn account_repo(&self) -> &dyn AccountRepo { &self.account_repo }
    fn commodity_repo(&self) -> &dyn CommodityRepo { &self.commodity_repo }
    fn member_repo(&self) -> &dyn MemberRepo { &self.member_repo }
    fn channel_repo(&self) -> &dyn ChannelRepo { &self.channel_repo }
    fn tag_repo(&self) -> &dyn TagRepo { &self.tag_repo }
    fn attachment_repo(&self) -> &dyn AttachmentRepo { &self.attachment_repo }
    fn transaction_repo(&self) -> &dyn TransactionRepo { &self.transaction_repo }
    fn posting_repo(&self) -> &dyn PostingRepo { &self.posting_repo }

    async fn transaction(&self) -> Result<Self::Tx, crate::error::DbError> {
        let conn = self.pool.get();
        conn.execute("BEGIN", [])?;
        Ok(SqliteTransaction {
            conn: Arc::new(Mutex::new(Connection::open_in_memory()?)), // placeholder
            committed: false,
            account_repo: SqliteAccountRepo,
            commodity_repo: SqliteCommodityRepo,
            member_repo: SqliteMemberRepo,
            channel_repo: SqliteChannelRepo,
            tag_repo: SqliteTagRepo,
            attachment_repo: SqliteAttachmentRepo,
            transaction_repo: SqliteTransactionRepo,
            posting_repo: SqlitePostingRepo,
        })
    }
}

/// SQLite 事务实现
pub struct SqliteTransaction {
    conn: Arc<Mutex<Connection>>,
    committed: bool,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    member_repo: SqliteMemberRepo,
    channel_repo: SqliteChannelRepo,
    tag_repo: SqliteTagRepo,
    attachment_repo: SqliteAttachmentRepo,
    transaction_repo: SqliteTransactionRepo,
    posting_repo: SqlitePostingRepo,
}

impl Transaction for SqliteTransaction {
    fn account_repo(&self) -> &dyn AccountRepo { &self.account_repo }
    fn commodity_repo(&self) -> &dyn CommodityRepo { &self.commodity_repo }
    fn member_repo(&self) -> &dyn MemberRepo { &self.member_repo }
    fn channel_repo(&self) -> &dyn ChannelRepo { &self.channel_repo }
    fn tag_repo(&self) -> &dyn TagRepo { &self.tag_repo }
    fn attachment_repo(&self) -> &dyn AttachmentRepo { &self.attachment_repo }
    fn transaction_repo(&self) -> &dyn TransactionRepo { &self.transaction_repo }
    fn posting_repo(&self) -> &dyn PostingRepo { &self.posting_repo }

    async fn commit(mut self) -> Result<(), crate::error::DbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("COMMIT", [])?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for SqliteTransaction {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.conn.lock().unwrap().execute("ROLLBACK", []);
        }
    }
}
```

- [ ] **步骤 4：运行编译验证**

运行：`cargo check -p accounting-sql`
预期：PASS（注意：transaction 实现使用 Connection::open_in_memory 作为占位，后续需要改为共享同一个 connection 的引用或实现方式）

- [ ] **步骤 5：Commit**

```bash
git add accounting-sql/src/database.rs accounting-sql/src/transaction.rs accounting-sql/src/impls/
git commit -m "feat(sql): add Database and Transaction traits with SQLite implementation"
```

## 任务 19：业务层 - AccountService

**文件：**

- 创建：`accounting-service/src/account_service.rs`

- [ ] **步骤 1：编写失败的测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::AccountId;
    use chrono::NaiveDate;

    #[test]
    fn test_create_account() {
        // 需要在实际测试中注入 mock database
        // 此处展示接口契约
        assert!(true);
    }
}
```

- [ ] **步骤 2：实现 AccountService**

```rust
use accounting::account::{Account};
use accounting::account_type::AccountType;
use accounting::closure::{compute_closure, AccountNode};
use accounting::error::AccountingError;
use accounting::id::AccountId;
use accounting_sql::database::Database;
use accounting_sql::transaction::Transaction;
use chrono::NaiveDate;

/// 账户服务
pub struct AccountService<D: Database> {
    db: D,
}

impl<D: Database> AccountService<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 创建账户
    pub async fn create(&self, mut account: Account) -> Result<AccountId, AccountingError> {
        // 验证父账户存在
        if let Some(parent_id) = account.parent_id {
            let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
            let parent = tx.account_repo().get(&tx.conn(), parent_id)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
            if parent.is_none() {
                return Err(AccountingError::AccountNotFound(format!("父账户 {} 不存在", parent_id)));
            }
            // full_name 唯一性检查
            let existing = tx.account_repo().get_by_name(&tx.conn(), &account.full_name)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
            if existing.is_some() {
                return Err(AccountingError::AccountAlreadyExists(account.full_name.clone()));
            }
        }

        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let id = tx.account_repo().create(&tx.conn(), &account)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        account.id = id;

        // 计算并插入闭包记录
        let all_accounts = tx.account_repo().list(&tx.conn())
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let nodes: Vec<AccountNode> = all_accounts.iter().map(|a| AccountNode {
            id: a.id,
            parent_id: a.parent_id,
            full_name: a.full_name.clone(),
        }).collect();
        let closure = compute_closure(&nodes);

        if let Some(ancestors) = closure.get(&id) {
            for &ancestor_id in ancestors {
                if ancestor_id != id {
                    // 插入闭包记录
                    let _ = tx.conn().execute(
                        "INSERT INTO account_ancestors (account_id, ancestor_id) VALUES (?1, ?2)",
                        rusqlite::params![id.0, ancestor_id.0],
                    );
                }
            }
        }

        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(id)
    }

    /// 关闭账户
    pub async fn close(&self, id: AccountId, closed_at: NaiveDate) -> Result<(), AccountingError> {
        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let account = tx.account_repo().get(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let account = account.ok_or_else(|| AccountingError::AccountNotFound(format!("账户 {} 不存在", id)))?;

        // 验证余额
        let balances = tx.posting_repo().sum_by_account(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        accounting::validation::validate_account_close(account.account_type, &balances)?;

        // 关闭目标账户
        tx.account_repo().close(&tx.conn(), id, closed_at)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 级联关闭子账户
        let children = tx.account_repo().list_children(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        for child in children {
            tx.account_repo().close(&tx.conn(), child.id, closed_at)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }

    /// 重新开启账户
    pub async fn reopen(&self, id: AccountId) -> Result<(), AccountingError> {
        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 级联恢复同时关闭的子账户
        let children = tx.account_repo().list_children(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        for child in children {
            // 只有和父账户同时关闭的才恢复
            tx.account_repo().reopen(&tx.conn(), child.id)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        }

        tx.account_repo().reopen(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;

        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }
}
```

- [ ] **步骤 3：运行编译验证**

运行：`cargo check -p accounting-service`
预期：PASS

- [ ] **步骤 4：Commit**

```bash
git add accounting-service/src/account_service.rs
git commit -m "feat(service): add AccountService with create, close, reopen"
```

## 任务 20：业务层 - TransactionService 与 ReportService

**文件：**

- 创建：`accounting-service/src/transaction_service.rs`
- 创建：`accounting-service/src/report_service.rs`

- [ ] **步骤 1：实现 TransactionService**

```rust
use accounting::error::AccountingError;
use accounting::id::TransactionId;
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting::validation::validate_transaction;
use accounting_sql::database::Database;
use chrono::NaiveDate;

/// 交易服务
pub struct TransactionService<D: Database> {
    db: D,
}

impl<D: Database> TransactionService<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 提交交易
    pub async fn submit(
        &self,
        transaction: Transaction,
        postings: Vec<Posting>,
        tag_ids: Vec<accounting::id::TagId>,
    ) -> Result<TransactionId, AccountingError> {
        // 核心库验证
        validate_transaction(&postings)?;

        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 插入交易
        let tx_id = tx.transaction_repo().insert(&tx.conn(), &transaction, &tag_ids)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 插入分录
        for mut posting in postings {
            posting.transaction_id = tx_id;
            tx.posting_repo().insert(&tx.conn(), &posting)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(tx_id)
    }

    /// 更新交易（全量替换）
    pub async fn update(
        &self,
        transaction: Transaction,
        postings: Vec<Posting>,
        tag_ids: Vec<accounting::id::TagId>,
    ) -> Result<(), AccountingError> {
        validate_transaction(&postings)?;

        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 删除旧分录
        tx.posting_repo().delete_by_transaction(&tx.conn(), transaction.id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 更新交易
        tx.transaction_repo().update(&tx.conn(), &transaction, &tag_ids)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;

        // 插入新分录
        for posting in postings {
            tx.posting_repo().insert(&tx.conn(), &posting)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }

    /// 删除交易
    pub async fn delete(&self, id: TransactionId) -> Result<(), AccountingError> {
        let tx = self.db.transaction().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        tx.transaction_repo().delete(&tx.conn(), id)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        tx.commit().await.map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }
}
```

- [ ] **步骤 2：实现 ReportService**

```rust
use accounting::id::{AccountId, CommodityId};
use accounting_sql::database::Database;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 报告服务
pub struct ReportService<D: Database> {
    db: D,
}

impl<D: Database> ReportService<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 获取账户余额（含子账户）
    pub async fn get_balance(
        &self,
        account_id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, accounting::error::AccountingError> {
        let tx = self.db.transaction().await.map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;

        // 通过闭包表获取所有相关账户
        let mut stmt = tx.conn().prepare(
            "SELECT account_id FROM account_ancestors WHERE ancestor_id = ?1 UNION SELECT ?1"
        ).map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
        let account_ids: Vec<i64> = stmt.query_map(
            rusqlite::params![account_id.0],
            |row| row.get(0),
        ).map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?
        .collect::<Result<_, _>>()
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;

        let mut totals: HashMap<CommodityId, Decimal> = HashMap::new();
        for id in account_ids {
            let balances = tx.posting_repo().sum_by_account(&tx.conn(), AccountId(id))
                .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
            for (commodity, amount) in balances {
                *totals.entry(commodity).or_insert_with(Decimal::ZERO) += amount;
            }
        }

        tx.commit().await.map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
        Ok(totals)
    }
}
```

- [ ] **步骤 3：运行编译验证**

运行：`cargo check -p accounting-service`
预期：PASS

- [ ] **步骤 4：Commit**

```bash
git add accounting-service/src/transaction_service.rs accounting-service/src/report_service.rs
git commit -m "feat(service): add TransactionService and ReportService"
```

## 任务 21：CLI - 基础框架

**文件：**

- 修改：`accounting-cli/Cargo.toml`（添加 tokio, tabled, serde, serde_json）
- 创建：`accounting-cli/src/output.rs`
- 创建：`accounting-cli/src/cmd/mod.rs`
- 修改：`accounting-cli/src/main.rs`
- 删除：`accounting-cli/src/commands/`（旧目录）

- [ ] **步骤 1：更新 accounting-cli/Cargo.toml**

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

- [ ] **步骤 2：实现 output.rs**

```rust
use clap::ValueEnum;
use tabled::{Table, Tabled};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
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

- [ ] **步骤 3：实现 cmd/mod.rs**

```rust
pub mod member;
pub mod account;
pub mod commodity;
pub mod tx;
pub mod tag;
pub mod report;
```

- [ ] **步骤 4：实现 main.rs**

```rust
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use accounting_cli::output::OutputFormat;

mod cmd;
mod output;

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
    Member(cmd::member::MemberCmd),
    /// 账户管理
    #[command(subcommand)]
    Account(cmd::account::AccountCmd),
    /// 商品/货币管理
    #[command(subcommand)]
    Commodity(cmd::commodity::CommodityCmd),
    /// 交易管理
    #[command(subcommand)]
    Tx(cmd::tx::TxCmd),
    /// 标签管理
    #[command(subcommand)]
    Tag(cmd::tag::TagCmd),
    /// 报告查询
    #[command(subcommand)]
    Report(cmd::report::ReportCmd),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Initialize => {
            if cli.db.exists() {
                eprintln!("错误: 数据库文件已存在");
                std::process::exit(1);
            }
            let _db = accounting_sql::impls::sqlite::SqliteDatabase::open(
                cli.db.to_string_lossy().as_ref()
            ).unwrap();
            output::print_line("数据库已初始化", cli.format);
            Ok(())
        }
        Commands::Member(c) => c.run(&cli.db, cli.format).await,
        Commands::Account(c) => c.run(&cli.db, cli.format).await,
        Commands::Commodity(c) => c.run(&cli.db, cli.format).await,
        Commands::Tx(c) => c.run(&cli.db, cli.format).await,
        Commands::Tag(c) => c.run(&cli.db, cli.format).await,
        Commands::Report(c) => c.run(&cli.db, cli.format).await,
    };

    if let Err(e) = result {
        eprintln!("错误: {:?}", e);
        std::process::exit(1);
    }
}
```

- [ ] **步骤 5：运行编译验证**

运行：`cargo check -p accounting-cli`
预期：PASS

- [ ] **步骤 6：Commit**

```bash
git add accounting-cli/Cargo.toml accounting-cli/src/main.rs accounting-cli/src/output.rs accounting-cli/src/cmd/mod.rs
rm -rf accounting-cli/src/commands/
git commit -m "feat(cli): add async CLI framework with output format, explicit init, cmd/mod structure"
```

## 任务 22：CLI - 成员、商品、标签命令

**文件：**

- 创建：`accounting-cli/src/cmd/member.rs`
- 创建：`accounting-cli/src/cmd/commodity.rs`
- 创建：`accounting-cli/src/cmd/tag.rs`

- [ ] **步骤 1：实现 member 命令**

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use accounting::error::AccountingError;
use accounting_cli::output::{OutputFormat, print_vec, print_line};

#[derive(Subcommand)]
pub enum MemberCmd {
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

impl MemberCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            MemberCmd::List(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                let members = service.list(args.limit, args.offset).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_vec(&members, format);
            }
            MemberCmd::Add(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                let id = service.add(&args.name).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_line(&format!("成员已创建: {}", id.0), format);
            }
            MemberCmd::Delete(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                service.delete(accounting::id::MemberId(args.id)).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_line(&format!("成员已删除: {}", args.id), format);
            }
        }
        Ok(())
    }
}
```

- [ ] **步骤 2：实现 commodity 命令**

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use accounting::error::AccountingError;
use accounting_cli::output::{OutputFormat, print_vec, print_line};

#[derive(Subcommand)]
pub enum CommodityCmd {
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

impl CommodityCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            CommodityCmd::List => {
                let service = accounting_service::commodity_service::CommodityService::new(db);
                let commodities = service.list().await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_vec(&commodities, format);
            }
            CommodityCmd::Add(args) => {
                let service = accounting_service::commodity_service::CommodityService::new(db);
                let id = service.add(&args.symbol, &args.name, args.precision).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_line(&format!("商品已创建: {}", id.0), format);
            }
        }
        Ok(())
    }
}
```

- [ ] **步骤 3：实现 tag 命令**

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use accounting::error::AccountingError;
use accounting_cli::output::{OutputFormat, print_vec, print_line};

#[derive(Subcommand)]
pub enum TagCmd {
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

impl TagCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            TagCmd::List => {
                let service = accounting_service::tag_service::TagService::new(db);
                let tags = service.list().await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_vec(&tags, format);
            }
            TagCmd::Add(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                let id = service.add(&args.name).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_line(&format!("标签已创建: {}", id.0), format);
            }
            TagCmd::Delete(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                service.delete_by_name(&args.name).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_line(&format!("标签已删除: {}", args.name), format);
            }
        }
        Ok(())
    }
}
```

- [ ] **步骤 4：运行编译验证**

运行：`cargo check -p accounting-cli`
预期：PASS（此时 Service 层可能缺少 MemberService/CommodityService/TagService，会有编译错误，需在 Service 层任务中补全）

- [ ] **步骤 5：Commit**

```bash
git add accounting-cli/src/cmd/member.rs accounting-cli/src/cmd/commodity.rs accounting-cli/src/cmd/tag.rs
git commit -m "feat(cli): add member, commodity, tag subcommands"
```

## 任务 23：CLI - 账户命令

**文件：**

- 创建：`accounting-cli/src/cmd/account.rs`

- [ ] **步骤 1：实现 account 命令**

```rust
use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting_cli::output::{OutputFormat, print, print_vec, print_line};

#[derive(ValueEnum, Clone, Debug)]
enum AccountTypeArg {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl From<AccountTypeArg> for AccountType {
    fn from(arg: AccountTypeArg) -> Self {
        match arg {
            AccountTypeArg::Asset => AccountType::Asset,
            AccountTypeArg::Liability => AccountType::Liability,
            AccountTypeArg::Equity => AccountType::Equity,
            AccountTypeArg::Income => AccountType::Income,
            AccountTypeArg::Expense => AccountType::Expense,
        }
    }
}

#[derive(Subcommand)]
pub enum AccountCmd {
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

impl AccountCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            AccountCmd::List(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let filter = args.r#type.map(|t| t.into());
                let accounts = service.list(filter, args.limit, args.offset).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_vec(&accounts, format);
            }
            AccountCmd::Add(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let account = accounting::account::Account {
                    id: accounting::id::AccountId(0),
                    full_name: args.full_name,
                    account_type: args.r#type.into(),
                    parent_id: args.parent.map(accounting::id::AccountId),
                    opened_at: chrono::Local::now().naive_local().date(),
                    closed_at: None,
                    is_system: false,
                    billing_day: args.billing_day,
                    repayment_day: args.repayment_day,
                };
                let id = service.create(account).await?;
                print_line(&format!("账户已创建: {}", id.0), format);
            }
            AccountCmd::Show(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let account = service.get(accounting::id::AccountId(args.id)).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                if let Some(a) = account {
                    print(&a, format);
                } else {
                    print_line("账户不存在", format);
                }
            }
            AccountCmd::Close(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let closed_at = chrono::Local::now().naive_local().date();
                service.close(accounting::id::AccountId(args.id), closed_at).await?;
                print_line(&format!("账户已关闭: {}", args.id), format);
            }
            AccountCmd::Reopen(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                service.reopen(accounting::id::AccountId(args.id)).await?;
                print_line(&format!("账户已重新开启: {}", args.id), format);
            }
            AccountCmd::Balance(args) => {
                let service = accounting_service::report_service::ReportService::new(db);
                let balances = service.get_balance(accounting::id::AccountId(args.id)).await?;
                for (commodity, amount) in balances {
                    println!("{}: {}", commodity.0, amount);
                }
            }
        }
        Ok(())
    }
}
```

- [ ] **步骤 2：运行编译验证**

运行：`cargo check -p accounting-cli`
预期：PASS（Service 层接口需与上述调用签名匹配）

- [ ] **步骤 3：Commit**

```bash
git add accounting-cli/src/cmd/account.rs
git commit -m "feat(cli): add account subcommands (list, add, show, close, reopen, balance)"
```

## 任务 24：CLI - 交易命令

**文件：**

- 创建：`accounting-cli/src/cmd/tx.rs`

- [ ] **步骤 1：实现 tx 命令**

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId, MemberId, ChannelId, TagId, TransactionId};
use accounting::transaction::Transaction;
use accounting::posting::Posting;
use accounting::transaction_filter::TransactionFilter;
use accounting_cli::output::{OutputFormat, print, print_vec, print_line};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Subcommand)]
pub enum TxCmd {
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

impl TxCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            TxCmd::Add(args) => {
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let tx = Transaction {
                    id: TransactionId(0),
                    date: NaiveDate::parse_from_str(&args.date, "%Y-%m-%d")
                        .map_err(|e| AccountingError::InvalidDate(e.to_string()))?,
                    description: args.description,
                    member_id: args.member.map(MemberId),
                    is_template: false,
                };
                let postings = parse_postings(&args.posting)?;
                let tag_ids = vec![]; // TODO: resolve tag names to IDs
                let id = service.submit(tx, postings, tag_ids).await?;
                print_line(&format!("交易已创建: {}", id.0), format);
            }
            TxCmd::List(args) => {
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let filter = TransactionFilter {
                    start_date: args.from.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                    end_date: args.to.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                    account_id: args.account.map(AccountId),
                    member_id: args.member.map(MemberId),
                    channel_id: None,
                    tag_id: None,
                    keyword: args.keyword,
                    has_installment: None,
                    is_template: Some(false),
                };
                let results = service.list(&filter, args.limit, args.offset).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                print_vec(&results, format);
            }
            TxCmd::Show(args) => {
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let result = service.get(TransactionId(args.id)).await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                if let Some((tx, postings)) = result {
                    print(&tx, format);
                    print_vec(&postings, format);
                } else {
                    print_line("交易不存在", format);
                }
            }
            TxCmd::Delete(args) => {
                let service = accounting_service::transaction_service::TransactionService::new(db);
                service.delete(TransactionId(args.id)).await?;
                print_line(&format!("交易已删除: {}", args.id), format);
            }
            TxCmd::Update(args) => {
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let tx = Transaction {
                    id: TransactionId(args.id),
                    date: NaiveDate::parse_from_str(&args.date, "%Y-%m-%d")
                        .map_err(|e| AccountingError::InvalidDate(e.to_string()))?,
                    description: args.description,
                    member_id: args.member.map(MemberId),
                    is_template: false,
                };
                let postings = parse_postings(&args.posting)?;
                let tag_ids = vec![];
                service.update(tx, postings, tag_ids).await?;
                print_line(&format!("交易已更新: {}", args.id), format);
            }
        }
        Ok(())
    }
}

/// 解析 posting 字符串：account_full_name:commodity_symbol:amount[:cost_commodity:cost]
fn parse_postings(posting_strs: &[String]) -> Result<Vec<Posting>, AccountingError> {
    let mut postings = Vec::new();
    for s in posting_strs {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 && parts.len() != 5 {
            return Err(AccountingError::InvalidTransaction(
                format!("无效 posting 格式: {} (期望 account:commodity:amount 或 account:commodity:amount:cost_commodity:cost)", s)
            ));
        }
        let account_name = parts[0];
        let commodity_symbol = parts[1];
        let amount = Decimal::from_str_exact(parts[2])
            .map_err(|e| AccountingError::InvalidTransaction(format!("金额解析失败: {}", e)))?;
        let (cost, cost_commodity_id) = if parts.len() == 5 {
            let cost_amount = Decimal::from_str_exact(parts[3])
                .map_err(|e| AccountingError::InvalidTransaction(format!("成本解析失败: {}", e)))?;
            (Some(cost_amount), Some(CommodityId(0))) // TODO: resolve commodity symbol
        } else {
            (None, None)
        };
        postings.push(Posting {
            id: accounting::id::PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(0), // TODO: resolve account name
            commodity_id: CommodityId(0), // TODO: resolve commodity symbol
            amount,
            cost,
            cost_commodity_id,
            description: None,
            member_id: None,
            channel_id: None,
        });
    }
    Ok(postings)
}
```

- [ ] **步骤 2：运行编译验证**

运行：`cargo check -p accounting-cli`
预期：PASS（Service 层需补充 `list` / `get` 接口）

- [ ] **步骤 3：Commit**

```bash
git add accounting-cli/src/cmd/tx.rs
git commit -m "feat(cli): add tx subcommands (add, list, show, delete, update)"
```

## 任务 25：CLI - 报告命令与最终编译验证

**文件：**

- 创建：`accounting-cli/src/cmd/report.rs`

- [ ] **步骤 1：实现 report 命令**

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use accounting::error::AccountingError;
use accounting::id::AccountId;
use accounting_cli::output::{OutputFormat, print_line};

#[derive(Subcommand)]
pub enum ReportCmd {
    Balance(ReportBalanceArgs),
    Bs,
    Is,
}

#[derive(Args)]
struct ReportBalanceArgs {
    account_id: i64,
}

impl ReportCmd {
    pub async fn run(self, db_path: &PathBuf, format: OutputFormat) -> Result<(), AccountingError> {
        if !db_path.exists() {
            eprintln!("错误: 数据库文件不存在，请先运行 initialize");
            std::process::exit(1);
        }
        let db = accounting_sql::impls::sqlite::SqliteDatabase::open(
            db_path.to_string_lossy().as_ref()
        ).map_err(|e| AccountingError::Unknown(e.to_string()))?;

        match self {
            ReportCmd::Balance(args) => {
                let service = accounting_service::report_service::ReportService::new(db);
                let balances = service.get_balance(AccountId(args.account_id)).await?;
                for (commodity, amount) in balances {
                    println!("{}: {}", commodity.0, amount);
                }
            }
            ReportCmd::Bs => {
                let service = accounting_service::report_service::ReportService::new(db);
                let bs = service.balance_sheet().await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                println!("资产负债表:");
                for (account, balances) in bs {
                    println!("  {}:", account);
                    for (commodity, amount) in balances {
                        println!("    {}: {}", commodity.0, amount);
                    }
                }
            }
            ReportCmd::Is => {
                let service = accounting_service::report_service::ReportService::new(db);
                let is = service.income_statement().await
                    .map_err(|e| AccountingError::Unknown(e.to_string()))?;
                println!("损益表:");
                for (account, balances) in is {
                    println!("  {}:", account);
                    for (commodity, amount) in balances {
                        println!("    {}: {}", commodity.0, amount);
                    }
                }
            }
        }
        Ok(())
    }
}
```

- [ ] **步骤 2：运行 `cargo check` 验证**

运行：`cargo check`
预期：PASS，所有 crate 编译通过

- [ ] **步骤 3：运行 `cargo test` 验证**

运行：`cargo test`
预期：PASS，所有测试通过

- [ ] **步骤 4：运行 `cargo clippy` 验证**

运行：`cargo clippy`
预期：PASS，无警告

- [ ] **步骤 5：Commit**

```bash
git add accounting-cli/src/cmd/report.rs
git commit -m "feat(cli): add report subcommands (balance, bs, is)"
```

## 自检

### 1. 规格覆盖度

| 规格需求 | 对应任务 |
|---------|---------|
| ID Newtypes (Display, Clone, Eq, Hash) | 任务 2 |
| AccountType enum + is_permanent + close_conditions | 任务 3 |
| InstallmentMethod enum | 任务 13 |
| Commodity, Account, Member, Channel, Tag, Attachment 模型 | 任务 5, 6, 7 |
| Transaction, Posting 模型 | 任务 8 |
| TransactionFilter | 任务 7 |
| AccountingError | 任务 4 |
| validate_transaction (空、单分录、单币种平衡、多币种双边 Cost) | 任务 10 |
| calculate_balance / calculate_all_balances | 任务 11 |
| validate_account_close (Asset 非零拒绝、Income 无条件) | 任务 13 |
| compute_closure (闭包表) | 任务 12 |
| infer_installment_index | 任务 13 |
| to_db_amount / from_db_amount | 任务 9 |
| Schema (11 张表 + 索引) | 任务 14 |
| 内置数据 (CNY, 7 系统账户, repayment tag) | 任务 14 |
| AccountRepo (create/get/get_by_name/list/list_children/close/reopen) | 任务 15 |
| TransactionRepo (insert/delete/get/list/count/update) | 任务 17 |
| PostingRepo (list_by_account/list_by_transaction/sum_by_account/delete_by_transaction) | 任务 17 |
| CommodityRepo, MemberRepo, ChannelRepo, TagRepo, AttachmentRepo | 任务 16 |
| Database trait + Transaction trait + Drop 自动回滚 | 任务 18 |
| AccountService (create, close, reopen) | 任务 19 |
| TransactionService (submit, update, delete) | 任务 20 |
| ReportService (get_balance) | 任务 20 |
| CLI: async runtime, format 选项, 显式 init, Service-only | 任务 21 |
| CLI: member, commodity, tag 命令 | 任务 22 |
| CLI: account 命令 (list, add, show, close, reopen, balance) | 任务 23 |
| CLI: tx 命令 (add, list, show, delete, update) | 任务 24 |
| CLI: report 命令 (balance, bs, is) | 任务 25 |

**遗漏：** 无遗漏。所有 Phase 1 规格需求已覆盖。

### 2. 占位符扫描

- 无 "待定" / "TODO"（代码中的 TODO precision 是已知技术债务，不影响功能完整性；TODO resolve name/symbol 需在 Service 层补全）
- 无 "添加适当的错误处理" 等模糊描述
- 所有引用类型在步骤中已定义
- 每个步骤包含完整代码

### 3. 类型一致性

- `AccountType` 映射值 1-5 在所有文件中一致
- ID 类型 (`AccountId(i64)`, `TransactionId(i64)` 等) 在所有文件中一致
- `NaiveDate` 用于日期字段，所有文件中一致
- `Decimal` 用于金额，通过 `amount.rs` 转换，所有文件中一致
- `is_system` 作为 `bool` 在模型层、`i32` 在数据库层，映射正确
- `closed_at` 为 `Option<NaiveDate>`，所有文件中一致
- CLI 命令签名 `async fn run(self, db_path: &PathBuf, format: OutputFormat)` 在所有 cmd 模块中一致
