use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum BudgetCmd {
    /// 创建预算表
    Create(BudgetCreateArgs),
    /// 列出所有预算表
    List,
    /// 显示预算执行情况
    Show(BudgetShowArgs),
    /// 更新预算表
    Update(BudgetUpdateArgs),
    /// 删除预算表
    Delete(BudgetDeleteArgs),
}

#[derive(Args)]
pub struct BudgetCreateArgs {
    /// 预算表名称
    #[arg(long)]
    pub name: String,
    /// 周期类型 (daily | weekly-sun | weekly-mon | monthly | yearly)
    #[arg(long)]
    pub period: String,
    /// 币种 ID
    #[arg(long)]
    pub commodity: i64,
    /// 限额映射 (账户路径:金额)，可多次指定
    #[arg(long = "limit")]
    pub limits: Vec<String>,
}

#[derive(Args)]
pub struct BudgetShowArgs {
    /// 预算表 ID
    pub budget_id: i64,
    /// 查询日期（默认今天）
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Args)]
pub struct BudgetUpdateArgs {
    /// 预算表 ID
    pub budget_id: i64,
    /// 新名称
    #[arg(long)]
    pub name: Option<String>,
    /// 新周期类型
    #[arg(long)]
    pub period: Option<String>,
    /// 新币种 ID
    #[arg(long)]
    pub commodity: Option<i64>,
    /// 替换限额映射（指定后将替换所有限额）
    #[arg(long = "limit")]
    pub limits: Vec<String>,
}

#[derive(Args)]
pub struct BudgetDeleteArgs {
    /// 预算表 ID
    pub budget_id: i64,
}

impl BudgetCmd {
    pub async fn run(
        &self,
        db: &SqliteDatabase,
        _format: &OutputFormat,
    ) -> Result<(), AccountingError> {
        match self {
            BudgetCmd::Create(args) => self::create(db, args).await,
            BudgetCmd::List => self::list(db).await,
            BudgetCmd::Show(args) => self::show(db, args).await,
            BudgetCmd::Update(args) => self::update(db, args).await,
            BudgetCmd::Delete(args) => self::delete(db, args).await,
        }
    }
}

fn parse_period(s: &str) -> Result<FinancePeriod, AccountingError> {
    match s.to_lowercase().as_str() {
        "daily" => Ok(FinancePeriod::Daily),
        "weekly-sun" => Ok(FinancePeriod::WeeklyFromSunday),
        "weekly-mon" => Ok(FinancePeriod::WeeklyFromMonday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(AccountingError::InvalidDate(format!(
            "未知周期类型: {}，可选: daily, weekly-sun, weekly-mon, monthly, yearly",
            s
        ))),
    }
}

async fn resolve_account_path(
    db: &SqliteDatabase,
    path: &str,
) -> Result<AccountId, AccountingError> {
    let parts: Vec<&str> = path.split(':').collect();
    let accounts = db
        .account_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

    // Navigate hierarchy
    let mut current_id = None;
    for (i, part) in parts.iter().enumerate() {
        let parent_id = current_id;
        let found = accounts
            .iter()
            .find(|a| a.name == *part && a.parent_id == parent_id && a.closed_at.is_none());

        match found {
            Some(a) => {
                current_id = Some(a.id);
            }
            None => {
                if i == 0 {
                    // Try matching by name regardless of parent for root
                    let found = accounts.iter().find(|a| {
                        a.name == *part && a.parent_id.is_none() && a.closed_at.is_none()
                    });
                    match found {
                        Some(a) => current_id = Some(a.id),
                        None => {
                            return Err(AccountingError::AccountNotFound(format!(
                                "账户路径不存在: {}",
                                path
                            )));
                        }
                    }
                } else {
                    return Err(AccountingError::AccountNotFound(format!(
                        "账户路径不存在: {}",
                        path
                    )));
                }
            }
        }
    }

    current_id.ok_or_else(|| AccountingError::AccountNotFound(format!("账户路径不存在: {}", path)))
}

async fn parse_limits(
    db: &SqliteDatabase,
    limit_strs: &[String],
) -> Result<Vec<(AccountId, Decimal)>, AccountingError> {
    let mut limits = Vec::new();
    for limit_str in limit_strs {
        // Format: "Account:Path:Amount" — last : separates amount
        let parts: Vec<&str> = limit_str.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AccountingError::InvalidDate(format!(
                "限额格式错误: {}，应为 账户路径:金额",
                limit_str
            )));
        }
        let amount = Decimal::from_str(parts[0])
            .map_err(|_| AccountingError::InvalidDate(format!("金额格式错误: {}", parts[0])))?;
        let account_id = resolve_account_path(db, parts[1]).await?;
        limits.push((account_id, amount));
    }
    Ok(limits)
}

async fn create(db: &SqliteDatabase, args: &BudgetCreateArgs) -> Result<(), AccountingError> {
    let period = parse_period(&args.period)?;
    let limits = parse_limits(db, &args.limits).await?;

    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let id = service
        .create_budget(&args.name, period, CommodityId(args.commodity), &limits)
        .await?;

    println!("预算表已创建，ID: {}", id.0);
    Ok(())
}

async fn list(db: &SqliteDatabase) -> Result<(), AccountingError> {
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let budgets = service.list_budgets().await?;

    if budgets.is_empty() {
        println!("暂无预算表");
        return Ok(());
    }

    println!("{:<5} {:<20} {:<20} Commodity", "ID", "Name", "Period");
    for b in &budgets {
        println!(
            "{:<5} {:<20} {:<20} {}",
            b.id.0, b.name, b.period, b.commodity_id.0
        );
    }
    Ok(())
}

async fn show(db: &SqliteDatabase, args: &BudgetShowArgs) -> Result<(), AccountingError> {
    let date = match &args.date {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AccountingError::InvalidDate(format!("日期格式错误: {}", d)))?,
        None => chrono::Local::now().date_naive(),
    };

    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let status = service
        .get_budget_status(accounting::id::BudgetId(args.budget_id), date)
        .await?;

    println!("预算表：{}", status.budget.name);
    println!(
        "周期：{} ~ {} ({})",
        status.period_start, status.period_end, status.budget.period
    );
    println!();

    // Get account names for display
    let accounts = db
        .account_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    let accounts_by_id: std::collections::HashMap<AccountId, accounting::account::Account> =
        accounts.into_iter().map(|a| (a.id, a)).collect();

    println!(
        "{:<30} {:>12} {:>12} {:>12} {:>8}",
        "Account", "Limit", "Actual", "Remaining", "%"
    );
    for item in &status.items {
        let account_name = accounts_by_id
            .get(&item.account_id)
            .map(|a| a.display_path(&accounts_by_id))
            .unwrap_or_else(|| item.account_id.to_string());

        let warning = if item.remaining < Decimal::ZERO {
            " ⚠ 超支"
        } else {
            ""
        };

        println!(
            "{:<30} {:>12} {:>12} {:>12} {:>7.2}%{}",
            account_name,
            item.limit_amount,
            item.actual_amount,
            item.remaining,
            item.percentage,
            warning
        );
    }

    Ok(())
}

async fn update(db: &SqliteDatabase, args: &BudgetUpdateArgs) -> Result<(), AccountingError> {
    let detail = {
        let service = accounting_service::report::budget::BudgetService::new(db.clone());
        service
            .get_budget_detail(accounting::id::BudgetId(args.budget_id))
            .await?
    };

    let name = args.name.as_deref().unwrap_or(&detail.budget.name);
    let period = match &args.period {
        Some(p) => parse_period(p)?,
        None => detail.budget.period,
    };
    let commodity_id = args
        .commodity
        .map(CommodityId)
        .unwrap_or(detail.budget.commodity_id);

    let limits = if args.limits.is_empty() {
        detail
            .limits
            .iter()
            .map(|l| (l.account_id, l.amount))
            .collect::<Vec<_>>()
    } else {
        parse_limits(db, &args.limits).await?
    };

    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    service
        .update_budget(
            accounting::id::BudgetId(args.budget_id),
            name,
            period,
            commodity_id,
            &limits,
        )
        .await?;

    println!("预算表已更新");
    Ok(())
}

async fn delete(db: &SqliteDatabase, args: &BudgetDeleteArgs) -> Result<(), AccountingError> {
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    service
        .delete_budget(accounting::id::BudgetId(args.budget_id))
        .await?;

    println!("预算表已删除");
    Ok(())
}
