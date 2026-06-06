use crate::cmd::{AccountRow, AccountTypeArg, BalanceRow};
use crate::output::{OutputFormat, print, print_line, print_vec};
use accounting::id::AccountId;
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

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
    #[arg(long, value_enum)]
    pub r#type: Option<AccountTypeArg>,
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
}

#[derive(Args)]
pub struct AccountAddArgs {
    /// 账户全名，第一段自动解析为账户类型（如 Assets:Cash、支出:餐饮）
    pub full_name: String,
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
                let service = accounting_service::account_service::AccountService::new(db);
                let account_type = args.r#type.map(|t| t.into());
                let accounts = service.list(account_type, args.limit, args.offset).await?;
                let rows: Vec<AccountRow> = accounts.iter().map(|a| a.into()).collect();
                print_vec(&rows, format);
            }
            AccountCmd::Add(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let id = service
                    .create_cascading(&args.full_name, args.billing_day, args.repayment_day)
                    .await?;
                print_line(&format!("{}", t!("account_created", id = id.0)), format);
            }
            AccountCmd::Show(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let account = service.get(AccountId(args.id)).await?;
                match account {
                    Some(a) => {
                        let row: AccountRow = (&a).into();
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
