//! 报表 API handler

use crate::handlers::member::AppState;
use accounting::account::Account;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId};
use accounting_service::report::balance_sheet::BalanceSheetService;
use accounting_service::report::cash_flow::CashFlowService;
use axum::{
    Json, Router,
    extract::{Query, State},
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

/// 获取资产负债表
async fn balance_sheet(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BalanceSheetResponse>, String> {
    let db = state.db();
    let account_paths: HashMap<i64, String> = {
        let accounts: HashMap<AccountId, Account> = db
            .account_list()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|a| (a.id, a))
            .collect();
        accounts
            .values()
            .map(|a| (a.id.0, a.display_path(&accounts)))
            .collect()
    };

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
            .unwrap_or_else(|| ab.account.name.clone()),
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
        "weekly-sun" => Ok(FinancePeriod::WeeklyFromSunday),
        "weekly-mon" => Ok(FinancePeriod::WeeklyFromMonday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(format!("未知周期类型: {}", s)),
    }
}

/// 获取资金流量表
async fn cash_flow(
    State(state): State<Arc<AppState>>,
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

    let account_paths: HashMap<i64, String> = {
        let accounts: HashMap<AccountId, Account> = db
            .account_list()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|a| (a.id, a))
            .collect();
        accounts
            .values()
            .map(|a| (a.id.0, a.display_path(&accounts)))
            .collect()
    };

    let service = CashFlowService::new(db.clone());
    let report = service
        .cash_flow_report(date, period, commodity_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(CashFlowResponse {
        period_start: report.period_start.to_string(),
        period_end: report.period_end.to_string(),
        items: report
            .items
            .into_iter()
            .map(|item| CashFlowItem {
                account: account_paths
                    .get(&item.account.id.0)
                    .cloned()
                    .unwrap_or_else(|| item.account.name.clone()),
                inflow: item.inflow.to_string(),
                outflow: item.outflow.to_string(),
                net: item.net.to_string(),
            })
            .collect(),
        total: CashFlowTotal {
            inflow: report.total.inflow.to_string(),
            outflow: report.total.outflow.to_string(),
            net: report.total.net.to_string(),
        },
    }))
}

/// 报表路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/reports/balance-sheet", get(balance_sheet))
        .route("/api/reports/cash-flow", get(cash_flow))
}
