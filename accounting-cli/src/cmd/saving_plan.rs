use crate::cmd::budget::{parse_deadline, parse_period, period_label};
use crate::cmd::resolver::{
    account_display_maps, resolve_account, resolve_commodity, resolve_saving_plan,
};
use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting::id::{AccountId, SavingPlanId};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use rust_i18n::t;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum SavingPlanCmd {
    /// 创建攒钱计划
    Create(SavingPlanCreateArgs),
    /// 列出所有攒钱计划
    List,
    /// 显示攒钱计划状态
    Show(SavingPlanShowArgs),
    /// 更新攒钱计划
    Update(SavingPlanUpdateArgs),
    /// 删除攒钱计划
    Delete(SavingPlanDeleteArgs),
}

#[derive(Args)]
pub struct SavingPlanCreateArgs {
    /// 攒钱计划名称
    #[arg(long)]
    pub name: String,
    /// 周期类型 (daily | weekly-sun | weekly-mon | monthly | yearly | once)，缺省为一次性计划
    #[arg(long)]
    pub period: Option<String>,
    /// 截止日期（YYYY-MM-DD，可选）
    #[arg(long)]
    pub deadline: Option<String>,
    /// 币种符号
    #[arg(long)]
    pub commodity: String,
    /// 目标金额
    #[arg(long)]
    pub target: String,
    /// 账户路径（必须位于 Assets 根账户子树内），可多次指定
    #[arg(long = "account")]
    pub accounts: Vec<String>,
}

#[derive(Args)]
pub struct SavingPlanShowArgs {
    /// 攒钱计划名称
    pub name: String,
    /// 查询日期（默认今天）
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Args)]
pub struct SavingPlanUpdateArgs {
    /// 攒钱计划名称
    pub name: String,
    /// 新名称
    #[arg(long = "name")]
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
    /// 新目标金额
    #[arg(long)]
    pub target: Option<String>,
    /// 替换账户集合（指定后将替换所有关联账户），可多次指定
    #[arg(long = "account")]
    pub accounts: Vec<String>,
}

#[derive(Args)]
pub struct SavingPlanDeleteArgs {
    /// 攒钱计划名称
    pub name: String,
}

impl SavingPlanCmd {
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

fn parse_target(s: &str) -> Result<Decimal, AccountingError> {
    Decimal::from_str(s).map_err(|_| {
        AccountingError::InvalidDate(format!("{}", t!("invalid_amount_format", value = s)))
    })
}

async fn parse_accounts(
    db: &SqliteDatabase,
    paths: &[String],
) -> Result<Vec<AccountId>, AccountingError> {
    let mut account_ids = Vec::new();
    for path in paths {
        account_ids.push(resolve_account(db, path).await?);
    }
    Ok(account_ids)
}

/// 批量解析攒钱计划显示名（回退链内置）
async fn saving_plan_name_map(
    db: &SqliteDatabase,
    ids: &[SavingPlanId],
    lang: &str,
) -> Result<std::collections::HashMap<SavingPlanId, String>, AccountingError> {
    db.saving_plan_display_names(ids, lang)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))
}

async fn create(
    db: &SqliteDatabase,
    args: &SavingPlanCreateArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let period = args
        .period
        .as_deref()
        .map(parse_period)
        .transpose()?
        .flatten();
    let deadline = args.deadline.as_deref().map(parse_deadline).transpose()?;
    let target = parse_target(&args.target)?;
    let commodity_id = resolve_commodity(db, &args.commodity).await?;
    let account_ids = parse_accounts(db, &args.accounts).await?;

    let service = accounting_service::report::saving_plan::SavingPlanService::new(db.clone());
    let id = service
        .create_saving_plan(
            &args.name,
            period,
            deadline,
            commodity_id,
            target,
            &account_ids,
            lang,
        )
        .await?;

    println!("{}", t!("saving_plan_created", id = id.0));
    Ok(())
}

async fn list(db: &SqliteDatabase, lang: &str) -> Result<(), AccountingError> {
    let service = accounting_service::report::saving_plan::SavingPlanService::new(db.clone());
    let plans = service.list_saving_plans().await?;

    if plans.is_empty() {
        println!("{}", t!("saving_plan_empty"));
        return Ok(());
    }

    let ids: Vec<SavingPlanId> = plans.iter().map(|p| p.id).collect();
    let names = saving_plan_name_map(db, &ids, lang).await?;
    let symbols = commodity_symbol_map(db).await?;
    // 满足率：一次批量全局分配计算（与 show 口径一致），避免逐计划调用 status
    let today = chrono::Local::now().date_naive();
    let statuses = service.list_saving_plan_statuses(today).await?;
    let satisfactions: std::collections::HashMap<SavingPlanId, String> = statuses
        .iter()
        .map(|s| (s.plan.id, s.satisfaction.normalize().to_string()))
        .collect();

    println!(
        "{:<5} {:<20} {:<20} {:<12} {:>12} {:<10} {}",
        t!("saving_plan_col_id"),
        t!("saving_plan_col_name"),
        t!("saving_plan_col_period"),
        t!("saving_plan_col_deadline"),
        t!("saving_plan_col_target"),
        t!("saving_plan_col_commodity"),
        t!("saving_plan_col_satisfaction"),
    );
    for p in &plans {
        println!(
            "{:<5} {:<20} {:<20} {:<12} {:>12} {:<10} {}",
            p.id.0,
            names.get(&p.id).cloned().unwrap_or_default(),
            p.period.map(period_label).unwrap_or_default(),
            p.deadline.map(|d| d.to_string()).unwrap_or_default(),
            p.target_amount,
            symbols.get(&p.commodity_id).cloned().unwrap_or_default(),
            satisfactions.get(&p.id).cloned().unwrap_or_default()
        );
    }
    Ok(())
}

/// 币种 ID → 符号映射（list 输出用，budget list 复用）
pub(crate) async fn commodity_symbol_map(
    db: &SqliteDatabase,
) -> Result<std::collections::HashMap<accounting::id::CommodityId, String>, AccountingError> {
    Ok(db
        .commodity_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
        .into_iter()
        .map(|c| (c.id, c.symbol))
        .collect())
}

async fn show(
    db: &SqliteDatabase,
    args: &SavingPlanShowArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let date = match &args.date {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| {
            AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = d)))
        })?,
        None => chrono::Local::now().date_naive(),
    };

    let plan_id = resolve_saving_plan(db, &args.name).await?;
    let service = accounting_service::report::saving_plan::SavingPlanService::new(db.clone());
    let status = service.get_saving_plan_status(plan_id, date).await?;

    let names = saving_plan_name_map(db, &[plan_id], lang).await?;
    let plan_name = names.get(&plan_id).cloned().unwrap_or_default();

    println!("{}", t!("saving_plan_name", name = plan_name));
    if status.expired {
        println!("{}", t!("saving_plan_expired"));
    }
    println!(
        "{}",
        t!("saving_plan_target", amount = status.target_amount)
    );
    println!(
        "{}",
        t!("saving_plan_balance", amount = status.current_balance)
    );
    println!("{}", t!("saving_plan_gap", amount = status.gap));
    if status.met {
        println!("{}", t!("saving_plan_met"));
    } else {
        println!("{}", t!("saving_plan_not_met", gap = status.gap));
    }
    // 满足率（service 返回未归一化 Decimal，如 "75.00"，此处归一化）
    println!(
        "{}",
        t!(
            "saving_plan_satisfaction",
            rate = status.satisfaction.normalize()
        )
    );
    // 每账户分配明细：余额 / 被更早计划占用 / 本计划分配
    if !status.accounts.is_empty() {
        let (accounts_by_id, account_names) = account_display_maps(db, lang).await?;
        println!("{}", t!("saving_plan_allocation_header"));
        for alloc in &status.accounts {
            let account_label = accounts_by_id
                .get(&alloc.account_id)
                .map(|a| a.display_path(&accounts_by_id, &account_names))
                .unwrap_or_else(|| alloc.account_id.to_string());
            println!(
                "{}",
                t!(
                    "saving_plan_allocation_row",
                    account = account_label,
                    balance = alloc.balance,
                    occupied = alloc.occupied_by_earlier,
                    allocated = alloc.allocated
                )
            );
        }
    }
    // 一次性计划不显示周期区间
    if status.plan.period.is_some() {
        println!(
            "{}",
            t!(
                "saving_plan_period",
                start = status
                    .period_start
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                end = status.period_end.map(|d| d.to_string()).unwrap_or_default(),
                period = status.plan.period.map(period_label).unwrap_or_default()
            )
        );
    }

    Ok(())
}

async fn update(
    db: &SqliteDatabase,
    args: &SavingPlanUpdateArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    let plan_id = resolve_saving_plan(db, &args.name).await?;
    let service = accounting_service::report::saving_plan::SavingPlanService::new(db.clone());
    let detail = service.get_saving_plan_detail(plan_id).await?;

    // 未指定的项沿用旧值
    let name = match args.new_name {
        Some(ref n) => n.clone(),
        None => saving_plan_name_map(db, &[plan_id], lang)
            .await?
            .get(&plan_id)
            .cloned()
            .ok_or_else(|| {
                AccountingError::InvalidTransaction(format!(
                    "{}",
                    t!("saving_plan_not_found", name = args.name)
                ))
            })?,
    };
    // 三态：未提供沿用旧值；`once` 置为一次性（清除循环）；其余设为新周期
    let period = match &args.period {
        Some(p) => parse_period(p)?,
        None => detail.plan.period,
    };
    let deadline = match &args.deadline {
        Some(d) if d.eq_ignore_ascii_case("none") => None,
        Some(d) => Some(parse_deadline(d)?),
        None => detail.plan.deadline,
    };
    let commodity_id = match args.commodity {
        Some(ref symbol) => resolve_commodity(db, symbol).await?,
        None => detail.plan.commodity_id,
    };
    let target = match &args.target {
        Some(t) => parse_target(t)?,
        None => detail.plan.target_amount,
    };
    // 提供 --account 时整体替换账户集合
    let account_ids = if args.accounts.is_empty() {
        detail.account_ids
    } else {
        parse_accounts(db, &args.accounts).await?
    };

    service
        .update_saving_plan(
            plan_id,
            &name,
            period,
            deadline,
            commodity_id,
            target,
            &account_ids,
            lang,
        )
        .await?;

    println!("{}", t!("saving_plan_updated"));
    Ok(())
}

async fn delete(db: &SqliteDatabase, args: &SavingPlanDeleteArgs) -> Result<(), AccountingError> {
    let plan_id = resolve_saving_plan(db, &args.name).await?;
    let service = accounting_service::report::saving_plan::SavingPlanService::new(db.clone());
    service.delete_saving_plan(plan_id).await?;

    println!("{}", t!("saving_plan_deleted"));
    Ok(())
}
