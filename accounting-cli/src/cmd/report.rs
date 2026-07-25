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
        lang: &str,
    ) -> Result<(), AccountingError> {
        match self {
            ReportCmd::Bs => {
                let service =
                    accounting_service::report::balance_sheet::BalanceSheetService::new(db.clone());
                let bs = service.balance_sheet().await?;
                let account_ids: Vec<accounting::id::AccountId> =
                    bs.assets.iter().map(|item| item.account.id).collect();
                let names = db
                    .account_display_names(&account_ids, lang)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                let mut rows = Vec::new();
                for item in &bs.assets {
                    let account_name = names
                        .get(&item.account.id)
                        .cloned()
                        .unwrap_or_else(|| item.account.id.to_string());
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!(
                                "{}",
                                t!("report_account_asset", name = account_name)
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

                let service =
                    accounting_service::report::cash_flow::CashFlowService::new(db.clone());
                let report = service.cash_flow_report(date, period, commodity_id).await?;
                let account_ids: Vec<accounting::id::AccountId> = report
                    .income
                    .iter()
                    .chain(report.expense.iter())
                    .map(|item| item.account.id)
                    .collect();
                let names = db
                    .account_display_names(&account_ids, lang)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

                println!(
                    "{}",
                    t!(
                        "report_cash_flow_title",
                        start = report.period_start,
                        end = report.period_end
                    )
                );
                println!();
                print_cash_flow_section(&report.income, &names);
                print_cash_flow_section(&report.expense, &names);
            }
        }
        Ok(())
    }
}

/// 打印资金流量表一节（Income 或 Expenses）：树状缩进，每级按金额降序
fn print_cash_flow_section(
    items: &[accounting_service::report::cash_flow::CashFlowItem],
    names: &std::collections::HashMap<accounting::id::AccountId, String>,
) {
    use accounting::id::AccountId;
    use accounting_service::report::cash_flow::CashFlowItem;
    use std::collections::HashMap;

    let by_id: HashMap<AccountId, &CashFlowItem> =
        items.iter().map(|i| (i.account.id, i)).collect();
    let mut children: HashMap<AccountId, Vec<&CashFlowItem>> = HashMap::new();
    let mut roots: Vec<&CashFlowItem> = Vec::new();
    for item in items {
        match item.account.parent_id {
            Some(pid) if by_id.contains_key(&pid) => {
                children.entry(pid).or_default().push(item);
            }
            _ => roots.push(item),
        }
    }

    fn walk(
        item: &CashFlowItem,
        depth: usize,
        children: &HashMap<AccountId, Vec<&CashFlowItem>>,
        names: &HashMap<AccountId, String>,
    ) {
        let name = names
            .get(&item.account.id)
            .cloned()
            .unwrap_or_else(|| item.account.id.to_string());
        println!(
            "{:<30} {:>12}",
            format!("{}{}", "  ".repeat(depth), name),
            item.amount
        );
        let mut kids = children.get(&item.account.id).cloned().unwrap_or_default();
        kids.sort_by_key(|k| std::cmp::Reverse(k.amount));
        for kid in kids {
            walk(kid, depth + 1, children, names);
        }
    }

    for root in roots {
        walk(root, 0, &children, names);
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
