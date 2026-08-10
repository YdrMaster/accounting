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
    /// 周期类型 (daily | weekly-sun | weekly-mon | monthly | yearly | once)，缺省为一次性预算
    #[arg(long)]
    pub period: Option<String>,
    /// 截止日期（YYYY-MM-DD，可选）
    #[arg(long)]
    pub deadline: Option<String>,
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
    #[arg(long = "name", alias = "new-name")]
    pub new_name: Option<String>,
    /// 新周期类型（once 表示置为一次性）
    #[arg(long)]
    pub period: Option<String>,
    /// 新截止日期（YYYY-MM-DD；`none` 表示清除）
    #[arg(long)]
    pub deadline: Option<String>,
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
        lang: &str,
    ) -> Result<(), AccountingError> {
        match self {
            Self::Create(args) => self::create(db, args, lang).await,
            Self::List => self::list(db, lang).await,
            Self::Show(args) => self::show(db, args, lang).await,
            Self::Update(args) => self::update(db, args, lang).await,
            Self::Delete(args) => self::delete(db, args).await,
        }
    }
}

/// 解析周期类型；`once`（大小写不敏感）表示一次性，返回 None
pub(crate) fn parse_period(s: &str) -> Result<Option<FinancePeriod>, AccountingError> {
    match s.to_lowercase().as_str() {
        "once" => Ok(None),
        "daily" => Ok(Some(FinancePeriod::Daily)),
        "weekly-sun" => Ok(Some(FinancePeriod::WeeklyFromSunday)),
        "weekly-mon" => Ok(Some(FinancePeriod::WeeklyFromMonday)),
        "monthly" => Ok(Some(FinancePeriod::Monthly)),
        "yearly" => Ok(Some(FinancePeriod::Yearly)),
        _ => Err(AccountingError::InvalidDate(format!(
            "{}",
            t!("unknown_period_type", period = s)
        ))),
    }
}

pub(crate) fn parse_deadline(s: &str) -> Result<chrono::NaiveDate, AccountingError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = s)))
    })
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

async fn create(
    db: &SqliteDatabase,
    args: &BudgetCreateArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let period = args
        .period
        .as_deref()
        .map(parse_period)
        .transpose()?
        .flatten();
    let deadline = args.deadline.as_deref().map(parse_deadline).transpose()?;
    let limits = parse_limits(db, &args.limits).await?;
    let commodity_id = resolve_commodity(db, &args.commodity).await?;

    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let id = service
        .create_budget(&args.name, period, deadline, commodity_id, &limits, lang)
        .await?;

    println!("{}", t!("budget_created", id = id.0));
    Ok(())
}

/// 批量解析预算显示名（回退链内置）
async fn budget_name_map(
    db: &SqliteDatabase,
    ids: &[accounting::id::BudgetId],
    lang: &str,
) -> Result<std::collections::HashMap<accounting::id::BudgetId, String>, AccountingError> {
    db.budget_display_names(ids, lang)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))
}

async fn list(db: &SqliteDatabase, lang: &str) -> Result<(), AccountingError> {
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let budgets = service.list_budgets().await?;

    if budgets.is_empty() {
        println!("{}", t!("budget_empty"));
        return Ok(());
    }

    let ids: Vec<accounting::id::BudgetId> = budgets.iter().map(|b| b.id).collect();
    let names = budget_name_map(db, &ids, lang).await?;
    let symbols = super::saving_plan::commodity_symbol_map(db).await?;

    println!("{:<5} {:<20} {:<20} Commodity", "ID", "Name", "Period");
    for b in &budgets {
        println!(
            "{:<5} {:<20} {:<20} {}",
            b.id.0,
            names.get(&b.id).cloned().unwrap_or_default(),
            b.period.map(|p| p.to_string()).unwrap_or_default(),
            symbols.get(&b.commodity_id).cloned().unwrap_or_default()
        );
    }
    Ok(())
}

async fn show(
    db: &SqliteDatabase,
    args: &BudgetShowArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let date = match &args.date {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| {
            AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = d)))
        })?,
        None => chrono::Local::now().date_naive(),
    };

    let budget_id = resolve_budget(db, &args.name).await?;
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let status = service.get_budget_status(budget_id, date).await?;

    let names = budget_name_map(db, &[budget_id], lang).await?;
    let budget_name = names.get(&budget_id).cloned().unwrap_or_default();

    println!("{}", t!("budget_name", name = budget_name));
    if status.expired {
        println!("{}", t!("budget_expired"));
    }
    // 一次性预算不显示周期区间
    if status.budget.period.is_some() {
        println!(
            "{}",
            t!(
                "budget_period",
                start = status
                    .period_start
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                end = status.period_end.map(|d| d.to_string()).unwrap_or_default(),
                period = status
                    .budget
                    .period
                    .map(|p| p.to_string())
                    .unwrap_or_default()
            )
        );
    }
    println!();

    // Get account names for display
    let (accounts_by_id, account_names) =
        crate::cmd::resolver::account_display_maps(db, lang).await?;

    println!(
        "{:<30} {:>12} {:>12} {:>12} {:>8}",
        "Account", "Limit", "Actual", "Remaining", "%"
    );
    for item in &status.items {
        let account_name = accounts_by_id
            .get(&item.account_id)
            .map(|a| a.display_path(&accounts_by_id, &account_names))
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

async fn update(
    db: &SqliteDatabase,
    args: &BudgetUpdateArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let budget_id = resolve_budget(db, &args.name).await?;
    let service = accounting_service::report::budget::BudgetService::new(db.clone());
    let detail = service.get_budget_detail(budget_id).await?;

    // 未指定新名称时沿用当前显示名（回退链取 lang 最优名字）
    let name = match args.new_name {
        Some(ref n) => n.clone(),
        None => budget_name_map(db, &[budget_id], lang)
            .await?
            .get(&budget_id)
            .cloned()
            .ok_or_else(|| {
                AccountingError::InvalidTransaction(format!(
                    "{}",
                    t!("budget_not_found", name = args.name)
                ))
            })?,
    };
    // 三态：未提供沿用旧值；`once` 置为一次性（清除循环）；其余设为新周期
    let period = match &args.period {
        Some(p) => parse_period(p)?,
        None => detail.budget.period,
    };
    let deadline = match &args.deadline {
        Some(d) if d.eq_ignore_ascii_case("none") => None,
        Some(d) => Some(parse_deadline(d)?),
        None => detail.budget.deadline,
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
        .update_budget(
            budget_id,
            &name,
            period,
            deadline,
            commodity_id,
            &limits,
            lang,
        )
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
