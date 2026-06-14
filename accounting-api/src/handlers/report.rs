//! 报表 API handler

use crate::handlers::member::AppState;
use accounting::transaction_filter::TransactionFilter;
use accounting_service::report_service::ReportService;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use chrono::NaiveDate;
use rust_i18n::t;
use serde::Serialize;
use std::sync::Arc;

/// 资产负债表响应
#[derive(Serialize)]
struct BalanceSheetResponse {
    assets: Vec<AccountBalanceItem>,
    liabilities: Vec<AccountBalanceItem>,
    equity: Vec<AccountBalanceItem>,
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

/// 损益表响应
#[derive(Serialize)]
struct IncomeStatementResponse {
    income: Vec<AccountBalanceItem>,
    expenses: Vec<AccountBalanceItem>,
}

/// 统计项响应
#[derive(Serialize)]
struct StatItem {
    name: String,
    income: Vec<BalanceEntry>,
    expense: Vec<BalanceEntry>,
}

/// 统计查询参数
#[derive(serde::Deserialize)]
pub struct StatsQuery {
    pub by: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// 获取资产负债表
async fn balance_sheet(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BalanceSheetResponse>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = ReportService::new(db);
    let sheet = service.balance_sheet().await.map_err(|e| e.to_string())?;

    Ok(Json(BalanceSheetResponse {
        assets: sheet.assets.into_iter().map(into_item).collect(),
        liabilities: sheet.liabilities.into_iter().map(into_item).collect(),
        equity: sheet.equity.into_iter().map(into_item).collect(),
    }))
}

/// 获取损益表
async fn income_statement(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IncomeStatementResponse>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = ReportService::new(db);
    let stmt = service
        .income_statement()
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(IncomeStatementResponse {
        income: stmt.income.into_iter().map(into_item).collect(),
        expenses: stmt.expenses.into_iter().map(into_item).collect(),
    }))
}

/// 按维度统计
async fn stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<Vec<StatItem>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;

    let mut filter = TransactionFilter::default();
    if let Some(from) = query.from {
        let date = NaiveDate::parse_from_str(&from, "%Y-%m-%d")
            .map_err(|e| format!("Invalid from date: {}", e))?;
        filter.start_date = Some(date);
    }
    if let Some(to) = query.to {
        let date = NaiveDate::parse_from_str(&to, "%Y-%m-%d")
            .map_err(|e| format!("Invalid to date: {}", e))?;
        filter.end_date = Some(date);
    }

    let service = ReportService::new(db);

    let items = match query.by.as_str() {
        "tag" => service
            .stats_by_tag(&filter)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| StatItem {
                name: s.tag.name,
                income: s.income.into_iter().map(into_entry).collect(),
                expense: s.expense.into_iter().map(into_entry).collect(),
            })
            .collect(),
        "member" => service
            .stats_by_member(&filter)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| StatItem {
                name: s.member.name,
                income: s.income.into_iter().map(into_entry).collect(),
                expense: s.expense.into_iter().map(into_entry).collect(),
            })
            .collect(),
        "channel" => service
            .stats_by_channel(&filter)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| StatItem {
                name: s.channel.name,
                income: s.income.into_iter().map(into_entry).collect(),
                expense: s.expense.into_iter().map(into_entry).collect(),
            })
            .collect(),
        other => return Err(t!("unsupported_stat_dimension", dimension = other).to_string()),
    };

    Ok(Json(items))
}

fn into_item(ab: accounting_service::report_service::AccountBalance) -> AccountBalanceItem {
    AccountBalanceItem {
        account: ab.account.full_name,
        balances: ab.balances.into_iter().map(into_entry).collect(),
    }
}

fn into_entry((cid, amount): (accounting::id::CommodityId, rust_decimal::Decimal)) -> BalanceEntry {
    BalanceEntry {
        commodity_id: cid.0,
        amount: amount.to_string(),
    }
}

/// 报表路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/reports/balance-sheet", get(balance_sheet))
        .route("/api/reports/income-statement", get(income_statement))
        .route("/api/reports/stats", get(stats))
}
