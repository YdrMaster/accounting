use crate::cmd::ReportBalanceRow;
use crate::cmd::resolver::resolve_commodity;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

#[derive(Subcommand)]
pub enum ReportCmd {
    /// 资产负债表
    Bs,
    /// 资金流量表
    CashFlow(CashFlowArgs),
}

#[derive(Args)]
pub struct CashFlowArgs {
    /// 查询日期（默认今天）
    #[arg(long)]
    pub date: Option<String>,
    /// 周期类型 (daily | weekly-sun | weekly-mon | monthly | yearly)
    #[arg(long)]
    pub period: Option<String>,
    /// 币种符号
    #[arg(long)]
    pub commodity: Option<String>,
}

impl ReportCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        match self {
            ReportCmd::Bs => {
                let service =
                    accounting_service::report::balance_sheet::BalanceSheetService::new(db);
                let bs = service.balance_sheet().await?;
                let mut rows = Vec::new();
                for item in &bs.assets {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!(
                                "{}",
                                t!("report_account_asset", name = item.account.name)
                            ),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line(t!("no_data").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::CashFlow(args) => {
                let today = chrono::Local::now().date_naive();
                let date = match &args.date {
                    Some(d) => parse_date(d)?,
                    None => today,
                };
                let period = match &args.period {
                    Some(p) => parse_period(p)?,
                    None => FinancePeriod::Monthly,
                };
                let commodity_id = match args.commodity {
                    Some(ref symbol) => resolve_commodity(&db, symbol).await?,
                    None => resolve_commodity(&db, "CNY").await?,
                };

                let service = accounting_service::report::cash_flow::CashFlowService::new(db);
                let report = service.cash_flow_report(date, period, commodity_id).await?;

                println!(
                    "{}",
                    t!(
                        "report_cash_flow_title",
                        start = report.period_start,
                        end = report.period_end
                    )
                );
                println!();
                println!(
                    "{:<30} {:>12} {:>12} {:>12}",
                    "Account", "Inflow", "Outflow", "Net"
                );
                for item in &report.items {
                    println!(
                        "{:<30} {:>12} {:>12} {:>12}",
                        item.account.name, item.inflow, item.outflow, item.net
                    );
                }
                println!("{}", "-".repeat(70));
                println!(
                    "{:<30} {:>12} {:>12} {:>12}",
                    "Total", report.total.inflow, report.total.outflow, report.total.net
                );
            }
        }
        Ok(())
    }
}

fn parse_date(s: &str) -> Result<chrono::NaiveDate, AccountingError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = s)))
    })
}

fn parse_period(s: &str) -> Result<FinancePeriod, AccountingError> {
    match s.to_lowercase().as_str() {
        "daily" => Ok(FinancePeriod::Daily),
        "weekly-sun" => Ok(FinancePeriod::WeeklyFromSunday),
        "weekly-mon" => Ok(FinancePeriod::WeeklyFromMonday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(AccountingError::InvalidDate(format!(
            "{}",
            t!("unknown_period_type", period = s)
        ))),
    }
}
