use crate::cmd::{AccountRow, BalanceRow};
use crate::output::{OutputFormat, print, print_line, print_vec};
use accounting::account_type::AccountType;
use accounting::id::AccountId;
use accounting_sql::database::Database;
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum AccountCmd {
    /// 列出账户
    List(AccountListArgs),
    /// 添加账户
    Add(AccountAddArgs),
    /// 查看账户
    Show(AccountShowArgs),
    /// 关闭账户
    Close(AccountCloseArgs),
    /// 重新开启账户
    Reopen(AccountReopenArgs),
    /// 查询余额
    Balance(AccountBalanceArgs),
}

#[derive(Args)]
pub struct AccountListArgs {
    /// 根账户 ID，仅列出该根账户子树下的账户
    #[arg(long)]
    pub r#type: Option<i64>,
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
}

#[derive(Args)]
pub struct AccountAddArgs {
    /// 账户名（本级名称），根账户需能用前缀推导类型（如 Assets、支出）
    pub name: String,
    /// 父账户 ID（创建子账户时必填）
    #[arg(long)]
    pub parent_id: Option<i64>,
    #[arg(long)]
    pub billing_day: Option<u8>,
    #[arg(long)]
    pub repayment_day: Option<u8>,
}

#[derive(Args)]
pub struct AccountShowArgs {
    pub id: i64,
}

#[derive(Args)]
pub struct AccountCloseArgs {
    pub id: i64,
}

#[derive(Args)]
pub struct AccountReopenArgs {
    pub id: i64,
}

#[derive(Args)]
pub struct AccountBalanceArgs {
    pub id: i64,
}

impl AccountCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            AccountCmd::List(args) => {
                let service = accounting_service::account_service::AccountService::new(db.clone());
                let root_id = args.r#type.map(AccountId);
                let accounts = service.list(root_id, args.limit, args.offset).await?;
                let conn = db.connection();
                let rows: Vec<AccountRow> = accounts
                    .iter()
                    .map(|a| {
                        let account_type = db
                            .account_repo()
                            .find_root_name(&conn, a.id)
                            .ok()
                            .and_then(|root_name| AccountType::from_str(&root_name).ok())
                            .map(|ty| ty.display_name())
                            .unwrap_or_default();
                        AccountRow::new(a, account_type)
                    })
                    .collect();
                print_vec(&rows, format);
            }
            AccountCmd::Add(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let account = accounting::account::Account {
                    id: AccountId(0),
                    name: args.name,
                    parent_id: args.parent_id.map(AccountId),
                    closed_at: None,
                    is_system: false,
                    billing_day: args.billing_day,
                    repayment_day: args.repayment_day,
                };
                let id = service.create(account).await?;
                print_line(&format!("{}", t!("account_created", id = id.0)), format);
            }
            AccountCmd::Show(args) => {
                let service = accounting_service::account_service::AccountService::new(db.clone());
                let account = service.get(AccountId(args.id)).await?;
                match account {
                    Some(a) => {
                        let conn = db.connection();
                        let account_type = db
                            .account_repo()
                            .find_root_name(&conn, a.id)
                            .ok()
                            .and_then(|root_name| AccountType::from_str(&root_name).ok())
                            .map(|ty| ty.display_name())
                            .unwrap_or_default();
                        let row = AccountRow::new(&a, account_type);
                        print(&row, format);
                    }
                    None => print_line(
                        &format!("{}", t!("account_not_found", id = args.id)),
                        format,
                    ),
                }
            }
            AccountCmd::Close(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                service.close(AccountId(args.id)).await?;
                print_line(&format!("{}", t!("account_closed", id = args.id)), format);
            }
            AccountCmd::Reopen(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                service.reopen(AccountId(args.id)).await?;
                print_line(&format!("{}", t!("account_reopened", id = args.id)), format);
            }
            AccountCmd::Balance(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let balances = service.balance(AccountId(args.id)).await?;
                let rows: Vec<BalanceRow> = balances
                    .iter()
                    .map(|(cid, amount)| BalanceRow {
                        commodity_id: cid.0,
                        amount: amount.to_string(),
                    })
                    .collect();
                if rows.is_empty() {
                    print_line(t!("balance_zero").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
        }
        Ok(())
    }
}
