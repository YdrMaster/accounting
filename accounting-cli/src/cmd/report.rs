use crate::cmd::{BalanceRow, ReportBalanceRow};
use crate::output::{OutputFormat, print_line, print_vec};
use accounting::id::AccountId;
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum ReportCmd {
    /// 查询账户余额
    Balance(ReportBalanceArgs),
    /// 资产负债表
    Bs,
    /// 损益表
    Is,
}

#[derive(Args)]
pub struct ReportBalanceArgs {
    pub account_id: i64,
}

impl ReportCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            ReportCmd::Balance(args) => {
                // 查询指定账户余额并转换为表格行
                let service = accounting_service::report_service::ReportService::new(db);
                let balances = service.get_balance(AccountId(args.account_id)).await?;
                let rows: Vec<BalanceRow> = balances
                    .iter()
                    .map(|(cid, amount)| BalanceRow {
                        commodity_id: cid.0,
                        amount: amount.to_string(),
                    })
                    .collect();
                if rows.is_empty() {
                    print_line("余额为零", format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::Bs => {
                // 生成资产负债表并按资产/负债/权益分类输出
                let service = accounting_service::report_service::ReportService::new(db);
                let bs = service.balance_sheet().await?;
                let mut rows = Vec::new();
                for item in &bs.assets {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[资产] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &bs.liabilities {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[负债] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &bs.equity {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[权益] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line("暂无数据", format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::Is => {
                // 生成损益表并按收入/费用分类输出
                let service = accounting_service::report_service::ReportService::new(db);
                let is = service.income_statement().await?;
                let mut rows = Vec::new();
                for item in &is.income {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[收入] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &is.expenses {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[费用] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line("暂无数据", format);
                } else {
                    print_vec(&rows, format);
                }
            }
        }
        Ok(())
    }
}
