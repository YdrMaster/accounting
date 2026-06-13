//! 账户 API handler

use crate::dto::{
    AccountDto, CreateAccountRequest, RenameAccountRequest, ReorderRequest, SetAccountOwnersRequest,
};
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
    let mut owners: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    let accounts_raw = db.account_repo().list(&conn).map_err(|e| e.to_string())?;
    for account in &accounts_raw {
        if let Ok(list) = db.account_repo().get_owners(&conn, account.id) {
            let ids: Vec<i64> = list.into_iter().map(|m| m.0).collect();
            if !ids.is_empty() {
                owners.insert(account.id.0, ids);
            }
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
            position: a.position,
            owner_ids: owners.get(&a.id.0).cloned().unwrap_or_default(),
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
    let owner_ids: Vec<MemberId> = req.owner_ids.into_iter().map(MemberId).collect();
    let id = service
        .create_cascading(
            &req.full_name,
            req.billing_day,
            req.repayment_day,
            &owner_ids,
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
    Json(req): Json<SetAccountOwnersRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let member_ids: Vec<MemberId> = req.owner_ids.into_iter().map(MemberId).collect();
    db.account_repo()
        .set_owners(&conn, AccountId(id), &member_ids)
        .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 重命名账户
async fn rename_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<RenameAccountRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let accounts = db.account_repo().list(&conn).map_err(|e| e.to_string())?;
    let target = accounts.iter().find(|a| a.id.0 == id).ok_or("账户不存在")?;
    // 同层级检查同名
    let mut siblings = accounts.iter().filter(|a| a.parent_id == target.parent_id);
    if siblings.any(|a| a.id.0 != id && a.full_name == req.full_name) {
        return Err("同名账户已存在".to_string());
    }
    db.account_repo()
        .rename(&conn, AccountId(id), &req.full_name)
        .map_err(|e| e.to_string())?;
    Ok("renamed".to_string())
}

/// 关闭账户
async fn close_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    db.account_repo()
        .close(&db.connection(), AccountId(id))
        .map_err(|e| e.to_string())?;
    Ok("closed".to_string())
}

/// 重开账户
async fn reopen_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    db.account_repo()
        .reopen(&conn, AccountId(id))
        .map_err(|e| e.to_string())?;
    Ok("reopened".to_string())
}

/// 重排账户
async fn reorder_accounts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReorderRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let ids: Vec<AccountId> = req.ids.into_iter().map(AccountId).collect();
    db.account_repo()
        .reorder(&db.connection(), &ids)
        .map_err(|e| e.to_string())?;
    Ok("reordered".to_string())
}

/// 账户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/:id/balance", get(get_balance))
        .route("/api/accounts/:id/owner", put(set_owner))
        .route("/api/accounts/:id/rename", put(rename_account))
        .route("/api/accounts/:id/close", put(close_account))
        .route("/api/accounts/:id/open", put(reopen_account))
        .route("/api/accounts/reorder", put(reorder_accounts))
}
