//! 报表 API handler

use crate::handlers::{Lang, member::AppState};
use accounting::account::Account;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId};
use accounting_service::report::balance_sheet::BalanceSheetService;
use accounting_service::report::category_breakdown::CategoryBreakdownService;
use accounting_service::report::cash_flow::CashFlowService;
use accounting_service::report::daily_summary::DailySummaryService;
use accounting_service::report::net_worth_trend::NetWorthTrendService;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// 资产负债表响应
#[derive(Serialize)]
struct BalanceSheetResponse {
    assets: Vec<AccountBalanceItem>,
}

/// 账户余额项
#[derive(Serialize)]
struct AccountBalanceItem {
    account: String,
    balances: Vec<BalanceEntry>,
}

/// 余额明细
#[derive(Serialize)]
struct BalanceEntry {
    commodity_id: i64,
    amount: String,
}

/// 资金流量表查询参数
#[derive(serde::Deserialize)]
pub struct CashFlowQuery {
    pub date: Option<String>,
    pub period: Option<String>,
    pub commodity: Option<i64>,
}

/// 资金流量表响应
#[derive(Serialize)]
struct CashFlowResponse {
    period_start: String,
    period_end: String,
    items: Vec<CashFlowItem>,
    total: CashFlowTotal,
}

#[derive(Serialize)]
struct CashFlowItem {
    account: String,
    inflow: String,
    outflow: String,
    net: String,
}

#[derive(Serialize)]
struct CashFlowTotal {
    inflow: String,
    outflow: String,
    net: String,
}

/// 批量加载账户表与按请求语言解析的显示路径
async fn load_account_paths(
    db: &accounting_sql::SqliteDatabase,
    lang: &str,
) -> Result<HashMap<i64, String>, String> {
    let accounts: HashMap<AccountId, Account> = db
        .account_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let ids: Vec<AccountId> = accounts.keys().copied().collect();
    let names = db
        .account_display_names(&ids, lang)
        .await
        .map_err(|e| e.to_string())?;
    Ok(accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts, &names)))
        .collect())
}

/// 获取资产负债表
async fn balance_sheet(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<BalanceSheetResponse>, String> {
    let db = state.db();
    let account_paths = load_account_paths(db, &lang).await?;

    let service = BalanceSheetService::new(db.clone());
    let sheet = service.balance_sheet().await.map_err(|e| e.to_string())?;

    Ok(Json(BalanceSheetResponse {
        assets: sheet
            .assets
            .into_iter()
            .map(|ab| into_item(&account_paths, ab))
            .collect(),
    }))
}

fn into_item(
    account_paths: &HashMap<i64, String>,
    ab: accounting_service::report::balance_sheet::AccountBalance,
) -> AccountBalanceItem {
    AccountBalanceItem {
        account: account_paths
            .get(&ab.account.id.0)
            .cloned()
            .unwrap_or_else(|| ab.account.id.0.to_string()),
        balances: ab.balances.into_iter().map(into_entry).collect(),
    }
}

fn into_entry((cid, amount): (CommodityId, Decimal)) -> BalanceEntry {
    BalanceEntry {
        commodity_id: cid.0,
        amount: amount.to_string(),
    }
}

fn parse_period(s: &str) -> Result<FinancePeriod, String> {
    match s.to_lowercase().as_str() {
        "daily" => Ok(FinancePeriod::Daily),
        "weekly" | "weekly-mon" => Ok(FinancePeriod::WeeklyFromMonday),
        "weekly-sun" => Ok(FinancePeriod::WeeklyFromSunday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(format!("未知周期类型: {}", s)),
    }
}

/// 获取资金流量表
async fn cash_flow(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Query(query): Query<CashFlowQuery>,
) -> Result<Json<CashFlowResponse>, String> {
    let db = state.db();

    let today = chrono::Local::now().date_naive();
    let date = match query.date {
        Some(d) => {
            NaiveDate::parse_from_str(&d, "%Y-%m-%d").map_err(|e| format!("Invalid date: {}", e))?
        }
        None => today,
    };
    let period = match query.period {
        Some(p) => parse_period(&p)?,
        None => FinancePeriod::Monthly,
    };
    let commodity_id = CommodityId(query.commodity.unwrap_or(1));

    let account_paths = load_account_paths(db, &lang).await?;

    let service = CashFlowService::new(db.clone());
    let report = service
        .cash_flow_report(date, period, commodity_id)
        .await
        .map_err(|e| e.to_string())?;

    let mut items: Vec<CashFlowItem> = report
        .items
        .into_iter()
        .map(|item| CashFlowItem {
            account: account_paths
                .get(&item.account.id.0)
                .cloned()
                .unwrap_or_else(|| item.account.id.0.to_string()),
            inflow: item.inflow.to_string(),
            outflow: item.outflow.to_string(),
            net: item.net.to_string(),
        })
        .collect();
    // 按请求语言的账户显示名排序
    items.sort_by(|a, b| a.account.cmp(&b.account));

    Ok(Json(CashFlowResponse {
        period_start: report.period_start.to_string(),
        period_end: report.period_end.to_string(),
        items,
        total: CashFlowTotal {
            inflow: report.total.inflow.to_string(),
            outflow: report.total.outflow.to_string(),
            net: report.total.net.to_string(),
        },
    }))
}

/// 按天收支汇总查询参数
#[derive(serde::Deserialize)]
pub struct DailySummaryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// 按天收支汇总项
#[derive(Serialize)]
struct DailySummaryItem {
    date: String,
    income: String,
    expense: String,
}

/// 获取按天收支汇总（仅返回有交易的日期，按日期升序）
async fn daily_summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DailySummaryQuery>,
) -> Result<Json<Vec<DailySummaryItem>>, (StatusCode, String)> {
    let bad_request = || {
        (
            StatusCode::BAD_REQUEST,
            "参数 from/to 必填，格式为 YYYY-MM-DD".to_string(),
        )
    };
    let (Some(from), Some(to)) = (query.from, query.to) else {
        return Err(bad_request());
    };
    let from = NaiveDate::parse_from_str(&from, "%Y-%m-%d").map_err(|_| bad_request())?;
    let to = NaiveDate::parse_from_str(&to, "%Y-%m-%d").map_err(|_| bad_request())?;

    let db = state.db();
    let service = DailySummaryService::new(db.clone());
    let items = service
        .daily_summary(from, to)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(
        items
            .into_iter()
            .map(|i| DailySummaryItem {
                date: i.date.to_string(),
                income: i.income.to_string(),
                expense: i.expense.to_string(),
            })
            .collect(),
    ))
}

/// 图表周期解析：仅支持 weekly / monthly / yearly（weekly 按周一起始）
fn parse_chart_period(s: &str) -> Result<FinancePeriod, String> {
    match s.to_lowercase().as_str() {
        "weekly" => Ok(FinancePeriod::WeeklyFromMonday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(format!("不支持的周期类型: {}", s)),
    }
}

fn chart_period_name(period: FinancePeriod) -> &'static str {
    match period {
        FinancePeriod::WeeklyFromMonday => "weekly",
        FinancePeriod::Monthly => "monthly",
        FinancePeriod::Yearly => "yearly",
        _ => "monthly",
    }
}

/// 资产趋势查询参数
#[derive(serde::Deserialize)]
pub struct NetWorthTrendQuery {
    pub period: Option<String>,
    pub commodity: Option<i64>,
}

/// 资产趋势响应
#[derive(Serialize)]
struct NetWorthTrendResponse {
    period: String,
    points: Vec<NetWorthTrendPointDto>,
}

#[derive(Serialize)]
struct NetWorthTrendPointDto {
    date: String,
    assets: String,
    liabilities: String,
}

/// 获取资产趋势（全量历史，按周/月/年分桶）
async fn net_worth_trend(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NetWorthTrendQuery>,
) -> Result<Json<NetWorthTrendResponse>, (StatusCode, String)> {
    let bad_request = |msg: String| (StatusCode::BAD_REQUEST, msg);

    let period = match query.period {
        Some(p) => parse_chart_period(&p).map_err(bad_request)?,
        None => FinancePeriod::Monthly,
    };
    let commodity_id = CommodityId(query.commodity.unwrap_or(1));

    let db = state.db();
    let service = NetWorthTrendService::new(db.clone());
    let points = service
        .net_worth_trend(period, commodity_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(NetWorthTrendResponse {
        period: chart_period_name(period).to_string(),
        points: points
            .into_iter()
            .map(|p| NetWorthTrendPointDto {
                date: p.date.to_string(),
                assets: p.assets.to_string(),
                liabilities: p.liabilities.to_string(),
            })
            .collect(),
    }))
}

/// 收支分类明细查询参数
#[derive(serde::Deserialize)]
pub struct CategoryBreakdownQuery {
    pub date: Option<String>,
    pub period: Option<String>,
    pub commodity: Option<i64>,
}

/// 收支分类明细响应
#[derive(Serialize)]
struct CategoryBreakdownResponse {
    period_start: String,
    period_end: String,
    income: Vec<CategoryAmountItem>,
    expense: Vec<CategoryAmountItem>,
}

#[derive(Serialize)]
struct CategoryAmountItem {
    account: String,
    amount: String,
}

/// 获取收支分类明细（Income / Expenses 各层级汇总）
async fn category_breakdown(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Query(query): Query<CategoryBreakdownQuery>,
) -> Result<Json<CategoryBreakdownResponse>, (StatusCode, String)> {
    let bad_request = |msg: String| (StatusCode::BAD_REQUEST, msg);

    let today = chrono::Local::now().date_naive();
    let date = match query.date {
        Some(d) => NaiveDate::parse_from_str(&d, "%Y-%m-%d")
            .map_err(|e| bad_request(format!("Invalid date: {}", e)))?,
        None => today,
    };
    let period = match query.period {
        Some(p) => parse_chart_period(&p).map_err(bad_request)?,
        None => FinancePeriod::Monthly,
    };
    let commodity_id = CommodityId(query.commodity.unwrap_or(1));

    let db = state.db();
    let account_paths = load_account_paths(db, &lang)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let service = CategoryBreakdownService::new(db.clone());
    let report = service
        .category_breakdown(date, period, commodity_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let to_items = |items: Vec<accounting_service::report::category_breakdown::CategoryBreakdownItem>| {
        items
            .into_iter()
            .map(|item| CategoryAmountItem {
                account: account_paths
                    .get(&item.account.id.0)
                    .cloned()
                    .unwrap_or_else(|| item.account.id.0.to_string()),
                amount: item.amount.to_string(),
            })
            .collect()
    };

    Ok(Json(CategoryBreakdownResponse {
        period_start: report.period_start.to_string(),
        period_end: report.period_end.to_string(),
        income: to_items(report.income),
        expense: to_items(report.expense),
    }))
}

/// 报表路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/reports/balance-sheet", get(balance_sheet))
        .route("/api/reports/cash-flow", get(cash_flow))
        .route("/api/reports/daily-summary", get(daily_summary))
        .route("/api/reports/net-worth-trend", get(net_worth_trend))
        .route("/api/reports/category-breakdown", get(category_breakdown))
}
