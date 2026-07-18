//! 账户 API handler

use crate::dto::{
    AccountDto, CreateAccountRequest, RenameAccountRequest, SetAccountOwnersRequest,
    UpdateAccountRequest,
};
use crate::handlers::{Lang, member::AppState};
use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::id::{AccountId, MemberId};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, put},
};
use rust_i18n::t;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// 账户列表
async fn list_accounts(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<Vec<AccountDto>>, String> {
    let db = state.db();

    // 先查询所有者
    let mut owners: HashMap<i64, Vec<i64>> = HashMap::new();
    let accounts_raw = db.account_list().await.map_err(|e| e.to_string())?;
    for account in &accounts_raw {
        if let Ok(list) = db.account_get_owners(account.id).await {
            let ids: Vec<i64> = list.into_iter().map(|m| m.0).collect();
            if !ids.is_empty() {
                owners.insert(account.id.0, ids);
            }
        }
    }

    let ids: Vec<AccountId> = accounts_raw.iter().map(|a| a.id).collect();
    let names = db
        .account_display_names(&ids, &lang)
        .await
        .map_err(|e| e.to_string())?;
    let accounts_by_id: HashMap<AccountId, Account> =
        accounts_raw.iter().map(|a| (a.id, a.clone())).collect();

    let mut dtos = Vec::new();
    for a in &accounts_raw {
        // 沿父链在内存中找到根账户，类型名双语均可被 AccountType::from_str 接受
        let root_name = root_display_name(a, &accounts_by_id, &names);
        let account_type = root_name
            .and_then(|n| AccountType::from_str(&n).ok())
            .map(|ty| format!("{:?}", ty))
            .unwrap_or_else(|| "Asset".to_string());
        dtos.push(AccountDto {
            id: a.id.0,
            name: names.get(&a.id).cloned().unwrap_or_default(),
            account_type,
            parent_id: a.parent_id.map(|id| id.0),
            closed_at: a.closed_at.map(|d| d.to_string()),
            is_system: a.is_system,
            billing_day: a.billing_day,
            repayment_day: a.repayment_day,
            owner_ids: owners.get(&a.id.0).cloned().unwrap_or_default(),
        });
    }
    Ok(Json(dtos))
}

/// 沿父链找根账户的显示名
fn root_display_name(
    account: &Account,
    accounts_by_id: &HashMap<AccountId, Account>,
    names: &HashMap<AccountId, String>,
) -> Option<String> {
    let mut current = account;
    while let Some(pid) = current.parent_id {
        current = accounts_by_id.get(&pid)?;
    }
    names.get(&current.id).cloned()
}

/// 创建账户
async fn create_account(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    let owner_ids: Vec<MemberId> = req.owner_ids.into_iter().map(MemberId).collect();

    let account = Account {
        id: AccountId(0),
        parent_id: req.parent_id.map(AccountId),
        closed_at: None,
        is_system: false,
        billing_day: req.billing_day,
        repayment_day: req.repayment_day,
    };

    let id = service
        .create(account, &req.name, &lang)
        .await
        .map_err(|e| e.to_string())?;

    if !owner_ids.is_empty() {
        db.account_set_owners(id, &owner_ids)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(Json(id.0))
}

/// 查询账户余额
async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<(i64, String)>>, String> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
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
    let db = state.db();
    let member_ids: Vec<MemberId> = req.owner_ids.into_iter().map(MemberId).collect();
    db.account_set_owners(AccountId(id), &member_ids)
        .await
        .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 重命名账户（按请求语言改写该语言的显示名）
async fn rename_account(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<RenameAccountRequest>,
) -> Result<String, String> {
    let db = state.db();
    let target = db
        .account_get(AccountId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or(t!("account_not_found", locale = lang.as_str()).to_string())?;
    // 同层级检查同名（命中任意语言的名字即视为占用）
    if let Some(dup) = db
        .account_get_by_parent_and_name(target.parent_id, &req.name)
        .await
        .map_err(|e| e.to_string())?
        && dup.id.0 != id
    {
        return Err(t!("account_name_exists", locale = lang.as_str()).to_string());
    }
    db.account_rename(AccountId(id), &req.name, &lang)
        .await
        .map_err(|e| e.to_string())?;
    Ok("renamed".to_string())
}

/// 关闭账户
async fn close_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    service
        .close(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("closed".to_string())
}

/// 重开账户
async fn reopen_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    service
        .reopen(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("reopened".to_string())
}

/// 删除账户
async fn delete_account(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    let target = db
        .account_get(AccountId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or(t!("account_not_found", locale = lang.as_str()).to_string())?;

    if target.is_system {
        return Err(t!("cannot_delete_system_account", locale = lang.as_str()).to_string());
    }

    let children = db
        .account_list_children(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    if !children.is_empty() {
        return Err(t!("delete_children_first", locale = lang.as_str()).to_string());
    }

    let has_postings = db
        .posting_has_postings(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    if has_postings {
        return Err(t!("account_has_postings", locale = lang.as_str()).to_string());
    }

    // 检查是否被账户映射引用
    let mapping_count = db
        .account_mapping_count_by_account(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    if mapping_count > 0 {
        return Err(t!("account_referenced_by_mapping", locale = lang.as_str()).to_string());
    }

    db.account_delete(AccountId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 更新账户字段
async fn update_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<String, String> {
    let db = state.db();
    db.account_update_fields(AccountId(id), req.billing_day, req.repayment_day)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 账户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/{id}/balance", get(get_balance))
        .route("/api/accounts/{id}/owner", put(set_owner))
        .route("/api/accounts/{id}/rename", put(rename_account))
        .route("/api/accounts/{id}/fields", put(update_account))
        .route("/api/accounts/{id}/close", put(close_account))
        .route("/api/accounts/{id}/open", put(reopen_account))
        .route("/api/accounts/{id}", delete(delete_account))
}
