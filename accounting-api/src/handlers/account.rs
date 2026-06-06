//! 账户 API handler

use crate::dto::{AccountDto, CreateAccountRequest};
use crate::handlers::member::AppState;
use accounting::id::AccountId;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use std::sync::Arc;

/// 账户列表（含余额信息，但余额通过单独 API 获取）
async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AccountDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = accounting_service::account_service::AccountService::new(db);
    let accounts = service
        .list(None, None, None)
        .await
        .map_err(|e| e.to_string())?;
    let dtos: Vec<AccountDto> = accounts
        .iter()
        .map(|a| AccountDto {
            id: a.id.0,
            full_name: a.full_name.clone(),
            account_type: format!("{:?}", a.account_type),
            parent_id: a.parent_id.map(|id| id.0),
            closed_at: a.closed_at.map(|d| d.to_string()),
            is_system: a.is_system,
            billing_day: a.billing_day,
            repayment_day: a.repayment_day,
        })
        .collect();
    Ok(Json(dtos))
}

/// 创建账户（级联创建）
async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = accounting_service::account_service::AccountService::new(db);
    let id = service
        .create_cascading(&req.full_name, req.billing_day, req.repayment_day)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(id.0))
}

/// 查询账户余额
async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<(i64, String)>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = accounting_service::account_service::AccountService::new(db);
    let balances = service
        .balance(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    let result: Vec<(i64, String)> = balances
        .into_iter()
        .map(|(cid, amount)| (cid.0, amount.to_string()))
        .collect();
    Ok(Json(result))
}

/// 账户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/:id/balance", get(get_balance))
}
