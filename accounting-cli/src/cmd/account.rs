use crate::cmd::resolver::resolve_account;
use crate::cmd::{AccountRow, BalanceRow};
use crate::output::{OutputFormat, print, print_line, print_vec};
use accounting::account_type::AccountType;
use accounting_sql::SqliteDatabase;
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
    /// 根账户路径（如 Assets、支出），仅列出该根账户子树下的账户
    #[arg(long)]
    pub r#type: Option<String>,
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
}

#[derive(Args)]
pub struct AccountAddArgs {
    /// 账户完整路径（如 Assets:Cash、支出:餐饮），中间父级自动级联创建
    pub path: String,
    #[arg(long)]
    pub billing_day: Option<u8>,
    #[arg(long)]
    pub repayment_day: Option<u8>,
}

#[derive(Args)]
pub struct AccountShowArgs {
    pub path: String,
}

#[derive(Args)]
pub struct AccountCloseArgs {
    pub path: String,
}

#[derive(Args)]
pub struct AccountReopenArgs {
    pub path: String,
}

#[derive(Args)]
pub struct AccountBalanceArgs {
    pub path: String,
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
                let root_id = match args.r#type {
                    Some(ref path) => Some(resolve_account(&db, path).await?),
                    None => None,
                };
                let accounts = service.list(root_id, args.limit, args.offset).await?;
                let mut rows: Vec<AccountRow> = Vec::new();
                for a in &accounts {
                    let account_type = db
                        .account_find_root_name(a.id)
                        .await
                        .ok()
                        .and_then(|root_name| AccountType::from_str(&root_name).ok())
                        .map(|ty| ty.display_name())
                        .unwrap_or_default();
                    rows.push(AccountRow::new(a, account_type));
                }
                print_vec(&rows, format);
            }
            AccountCmd::Add(args) => {
                let service = accounting_service::account_service::AccountService::new(db);
                let id = service
                    .create_cascading(
                        &args.path,
                        args.billing_day,
                        args.repayment_day,
                        &[],
                    )
                    .await?;
                print_line(&format!("{}", t!("account_created", id = id.0)), format);
            }
            AccountCmd::Show(args) => {
                let service = accounting_service::account_service::AccountService::new(db.clone());
                let account_id = resolve_account(&db, &args.path).await?;
                let account = service.get(account_id).await?;
                match account {
                    Some(a) => {
                        let account_type = db
                            .account_find_root_name(a.id)
                            .await
                            .ok()
                            .and_then(|root_name| AccountType::from_str(&root_name).ok())
                            .map(|ty| ty.display_name())
                            .unwrap_or_default();
                        let row = AccountRow::new(&a, account_type);
                        print(&row, format);
                    }
                    None => print_line(
                        &format!("{}", t!("account_not_found", name = args.path)),
                        format,
                    ),
                }
            }
            AccountCmd::Close(args) => {
                let account_id = resolve_account(&db, &args.path).await?;
                let service = accounting_service::account_service::AccountService::new(db);
                service.close(account_id).await?;
                print_line(&format!("{}", t!("account_closed", name = args.path)), format);
            }
            AccountCmd::Reopen(args) => {
                let account_id = resolve_account(&db, &args.path).await?;
                let service = accounting_service::account_service::AccountService::new(db);
                service.reopen(account_id).await?;
                print_line(&format!("{}", t!("account_reopened", name = args.path)), format);
            }
            AccountCmd::Balance(args) => {
                let account_id = resolve_account(&db, &args.path).await?;
                let service = accounting_service::account_service::AccountService::new(db);
                let balances = service.balance(account_id).await?;
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
