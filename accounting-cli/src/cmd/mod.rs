pub mod account;
pub mod commodity;
pub mod member;
pub mod report;
pub mod tag;
pub mod tx;

use crate::output::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// 记账 CLI
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// 数据库文件路径
    pub db: PathBuf,
    /// 输出格式
    #[arg(long, value_enum, default_value = "table")]
    pub format: OutputFormat,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 初始化数据库
    Initialize,
    /// 成员管理
    #[command(subcommand)]
    Member(member::MemberCmd),
    /// 账户管理
    #[command(subcommand)]
    Account(account::AccountCmd),
    /// 商品/货币管理
    #[command(subcommand)]
    Commodity(commodity::CommodityCmd),
    /// 交易管理
    #[command(subcommand)]
    Tx(tx::TxCmd),
    /// 标签管理
    #[command(subcommand)]
    Tag(tag::TagCmd),
    /// 报告查询
    #[command(subcommand)]
    Report(report::ReportCmd),
}

/// 账户类型参数（映射到 domain AccountType）
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum AccountTypeArg {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl From<AccountTypeArg> for accounting::account_type::AccountType {
    fn from(arg: AccountTypeArg) -> Self {
        match arg {
            AccountTypeArg::Asset => accounting::account_type::AccountType::Asset,
            AccountTypeArg::Liability => accounting::account_type::AccountType::Liability,
            AccountTypeArg::Equity => accounting::account_type::AccountType::Equity,
            AccountTypeArg::Income => accounting::account_type::AccountType::Income,
            AccountTypeArg::Expense => accounting::account_type::AccountType::Expense,
        }
    }
}

// --- Tabled + Serialize wrapper types ---

use serde::Serialize;
use tabled::Tabled;

/// 成员表格行
#[derive(Tabled, Serialize)]
pub struct MemberRow {
    pub id: i64,
    pub name: String,
}

impl From<&accounting::member::Member> for MemberRow {
    fn from(m: &accounting::member::Member) -> Self {
        Self {
            id: m.id.0,
            name: m.name.clone(),
        }
    }
}

/// 商品表格行
#[derive(Tabled, Serialize)]
pub struct CommodityRow {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub precision: u8,
}

impl From<&accounting::commodity::Commodity> for CommodityRow {
    fn from(c: &accounting::commodity::Commodity) -> Self {
        Self {
            id: c.id.0,
            symbol: c.symbol.clone(),
            name: c.name.clone(),
            precision: c.precision,
        }
    }
}

/// 标签表格行
#[derive(Tabled, Serialize)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_system: bool,
}

impl From<&accounting::tag::Tag> for TagRow {
    fn from(t: &accounting::tag::Tag) -> Self {
        Self {
            id: t.id.0,
            name: t.name.clone(),
            description: t.description.clone().unwrap_or_default(),
            is_system: t.is_system,
        }
    }
}

/// 账户表格行
#[derive(Tabled, Serialize)]
pub struct AccountRow {
    pub id: i64,
    pub full_name: String,
    pub account_type: String,
    pub parent_id: String,
    pub closed_at: String,
    pub is_system: bool,
}

impl From<&accounting::account::Account> for AccountRow {
    fn from(a: &accounting::account::Account) -> Self {
        Self {
            id: a.id.0,
            full_name: a.full_name.clone(),
            account_type: format!("{:?}", a.account_type),
            parent_id: a.parent_id.map(|id| id.0.to_string()).unwrap_or_default(),
            closed_at: a.closed_at.map(|d| d.to_string()).unwrap_or_default(),
            is_system: a.is_system,
        }
    }
}

/// 交易表格行
#[derive(Tabled, Serialize)]
pub struct TransactionRow {
    pub id: i64,
    pub date_time: String,
    pub description: String,
    pub member_id: String,
    pub is_template: bool,
}

impl From<&accounting::transaction::Transaction> for TransactionRow {
    fn from(t: &accounting::transaction::Transaction) -> Self {
        Self {
            id: t.id.0,
            date_time: t.date_time.to_string(),
            description: t.description.clone(),
            member_id: t.member_id.map(|id| id.0.to_string()).unwrap_or_default(),
            is_template: t.is_template,
        }
    }
}

/// 分录表格行
#[derive(Tabled, Serialize)]
pub struct PostingRow {
    pub id: i64,
    pub transaction_id: i64,
    pub account_id: i64,
    pub commodity_id: i64,
    pub amount: String,
    pub cost: String,
    pub cost_commodity_id: String,
    pub description: String,
}

impl From<&accounting::posting::Posting> for PostingRow {
    fn from(p: &accounting::posting::Posting) -> Self {
        Self {
            id: p.id.0,
            transaction_id: p.transaction_id.0,
            account_id: p.account_id.0,
            commodity_id: p.commodity_id.0,
            amount: p.amount.to_string(),
            cost: p.cost.map(|c| c.to_string()).unwrap_or_default(),
            cost_commodity_id: p
                .cost_commodity_id
                .map(|id| id.0.to_string())
                .unwrap_or_default(),
            description: p.description.clone().unwrap_or_default(),
        }
    }
}

/// 余额表格行
#[derive(Tabled, Serialize)]
pub struct BalanceRow {
    pub commodity_id: i64,
    pub amount: String,
}

/// 报告余额行（用于 BS/IS）
#[derive(Tabled, Serialize)]
pub struct ReportBalanceRow {
    pub account_id: i64,
    pub account_name: String,
    pub commodity_id: i64,
    pub amount: String,
}
