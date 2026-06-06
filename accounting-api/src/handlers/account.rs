//! 账户 API handler

use crate::dto::{AccountDto, CreateAccountRequest, SetAccountOwnerRequest};
use crate::handlers::member::AppState;
use accounting::id::{AccountId, MemberId};
use accounting_sql::database::Database;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use std::sync::Arc;

/// 账户列表
async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AccountDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();

    // 先查询所有者
    let mut owners: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let accounts_raw = db.account_repo().list(&conn).map_err(|e| e.to_string())?;
    for account in &accounts_raw {
        if let Ok(Some(owner)) = db.account_repo().get_owner(&conn, account.id) {
            owners.insert(account.id.0, owner.0);
        }
    }

    let dtos: Vec<AccountDto> = accounts_raw
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
            owner_id: owners.get(&a.id.0).copied(),
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
        .create_cascading(
            &req.full_name,
            req.billing_day,
            req.repayment_day,
            req.owner_id.map(MemberId),
        )
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

/// 设置账户所有者
async fn set_owner(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<SetAccountOwnerRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    db.account_repo()
        .set_owner(&conn, AccountId(id), MemberId(req.owner_id))
        .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 账户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/:id/balance", get(get_balance))
        .route("/api/accounts/:id/owner", put(set_owner))
}
