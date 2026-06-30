use crate::cmd::resolver::{resolve_account, resolve_budget, resolve_commodity};
use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::AccountId;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use rust_i18n::t;
use std::str::FromStr;

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
    /// 币种符号
    #[arg(long)]
    pub commodity: String,
    /// 限额映射 (账户路径:金额)，可多次指定
    #[arg(long = "limit")]
    pub limits: Vec<String>,
}

#[derive(Args)]
pub struct BudgetShowArgs {
    /// 预算表名称
    pub name: String,
    /// 查询日期（默认今天）
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Args)]
pub struct BudgetUpdateArgs {
    /// 预算表名称
    pub name: String,
    /// 新名称
    #[arg(long)]
    pub new_name: Option<String>,
    /// 新周期类型
    #[arg(long)]
    pub period: Option<String>,
    /// 新币种符号
    #[arg(long)]
    pub commodity: Option<String>,
    /// 替换限额映射（指定后将替换所有限额）
    #[arg(long = "limit")]
    pub limits: Vec<String>,
}

#[derive(Args)]
pub struct BudgetDeleteArgs {
    /// 预算表名称
    pub name: String,
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
            "{}",
            t!("unknown_period_type", period = s)
        ))),
    }
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
                "{}",
                t!("budget_limit_format_invalid", value = limit_str)
            )));
        }
        let amount = Decimal::from_str(parts[0]).map_err(|_| {
            AccountingError::InvalidDate(format!(
                "{}",
                t!("budget_amount_format_invalid", value = parts[0])
            ))
        })?;
        let account_id = resolve_account(db, parts[1]).await?;
        limits.push((account_id, amount));
    }
    Ok(limits)
}

async fn create(db: &SqliteDatabase, args: &BudgetCreateArgs) -> Result<(), AccountingError> {
    let period = parse_period(&args.period)?;
    let limits = parse_limits(db, &args.limits).await?;
    let commodity_id = resolve_commodity(db, &args.commodity).await?;

    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let id = service
        .create_budget(&args.name, period, commodity_id, &limits)
        .await?;

    println!("{}", t!("budget_created", id = id.0));
    Ok(())
}

async fn list(db: &SqliteDatabase) -> Result<(), AccountingError> {
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let budgets = service.list_budgets().await?;

    if budgets.is_empty() {
        println!("{}", t!("budget_empty"));
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
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| {
            AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = d)))
        })?,
        None => chrono::Local::now().date_naive(),
    };

    let budget_id = resolve_budget(db, &args.name).await?;
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let status = service.get_budget_status(budget_id, date).await?;

    println!("{}", t!("budget_name", name = status.budget.name));
    println!(
        "{}",
        t!(
            "budget_period",
            start = status.period_start,
            end = status.period_end,
            period = status.budget.period
        )
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
            t!("budget_over_spent").to_string()
        } else {
            String::new()
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
    let budget_id = resolve_budget(db, &args.name).await?;
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let detail = service.get_budget_detail(budget_id).await?;

    let name = args.new_name.as_deref().unwrap_or(&detail.budget.name);
    let period = match &args.period {
        Some(p) => parse_period(p)?,
        None => detail.budget.period,
    };
    let commodity_id = match args.commodity {
        Some(ref symbol) => resolve_commodity(db, symbol).await?,
        None => detail.budget.commodity_id,
    };

    let limits = if args.limits.is_empty() {
        detail
            .limits
            .iter()
            .map(|l| (l.account_id, l.amount))
            .collect::<Vec<_>>()
    } else {
        parse_limits(db, &args.limits).await?
    };

    service
        .update_budget(budget_id, name, period, commodity_id, &limits)
        .await?;

    println!("{}", t!("budget_updated"));
    Ok(())
}

async fn delete(db: &SqliteDatabase, args: &BudgetDeleteArgs) -> Result<(), AccountingError> {
    let budget_id = resolve_budget(db, &args.name).await?;
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    service.delete_budget(budget_id).await?;

    println!("{}", t!("budget_deleted"));
    Ok(())
}
