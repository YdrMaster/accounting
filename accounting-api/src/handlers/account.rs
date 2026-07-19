//! 账户 API handler

use crate::dto::{
    AccountDto, CreateAccountRequest, RenameAccountRequest, SetAccountOwnersRequest,
    SetAccountParentRequest, UpdateAccountRequest,
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

/// 变更账户父节点（整棵子树跟随移动）
async fn set_parent(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<SetAccountParentRequest>,
) -> Result<Json<AccountDto>, String> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    service
        .reparent(AccountId(id), AccountId(req.parent_id), &lang)
        .await
        .map_err(|e| e.to_string())?;

    // 返回更新后的账户（与列表端点的 DTO 口径一致）
    let account = db
        .account_get(AccountId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or(t!("account_not_found", locale = lang.as_str()).to_string())?;
    let names = db
        .account_display_names(&[account.id], &lang)
        .await
        .map_err(|e| e.to_string())?;
    let account_type = db
        .account_find_root_name(account.id, &lang)
        .await
        .ok()
        .and_then(|n| AccountType::from_str(&n).ok())
        .map(|ty| format!("{:?}", ty))
        .unwrap_or_else(|| "Asset".to_string());
    let owner_ids: Vec<i64> = db
        .account_get_owners(account.id)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| m.0)
        .collect();

    Ok(Json(AccountDto {
        id: account.id.0,
        name: names.get(&account.id).cloned().unwrap_or_default(),
        account_type,
        parent_id: account.parent_id.map(|id| id.0),
        closed_at: account.closed_at.map(|d| d.to_string()),
        is_system: account.is_system,
        billing_day: account.billing_day,
        repayment_day: account.repayment_day,
        owner_ids,
    }))
}

/// 账户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/{id}/balance", get(get_balance))
        .route("/api/accounts/{id}/owner", put(set_owner))
        .route("/api/accounts/{id}/rename", put(rename_account))
        .route("/api/accounts/{id}/fields", put(update_account))
        .route("/api/accounts/{id}/parent", put(set_parent))
        .route("/api/accounts/{id}/close", put(close_account))
        .route("/api/accounts/{id}/open", put(reopen_account))
        .route("/api/accounts/{id}", delete(delete_account))
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        Arc::new(AppState { db })
    }

    async fn account_id_by_name(state: &Arc<AppState>, name: &str) -> i64 {
        state
            .db()
            .account_get_by_name(name)
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    async fn create_child(state: &Arc<AppState>, parent_id: i64, name: &str) -> i64 {
        let service = accounting_service::account_service::AccountService::new(state.db().clone());
        let account = Account {
            id: AccountId(0),
            parent_id: Some(AccountId(parent_id)),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        service.create(account, name, "en").await.unwrap().0
    }

    #[tokio::test]
    async fn set_parent_moves_account_and_returns_updated_dto() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;
        let sub = create_child(&state, bank, "Sub").await;
        let broker = create_child(&state, assets, "Broker").await;

        let dto = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(SetAccountParentRequest { parent_id: broker }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(dto.id, bank);
        assert_eq!(dto.parent_id, Some(broker));
        assert_eq!(dto.name, "Bank");
        assert_eq!(dto.account_type, "Asset");
        // 子账户跟随移动
        assert!(
            state
                .db()
                .account_is_descendant_of(AccountId(sub), AccountId(broker))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn set_parent_cross_type_move_updates_account_type() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        let bank = create_child(&state, assets, "Bank").await;
        let food = create_child(&state, expenses, "Food").await;

        let dto = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(SetAccountParentRequest { parent_id: food }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(dto.account_type, "Expense");
    }

    #[tokio::test]
    async fn set_parent_closed_account_keeps_closed_state() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let card = create_child(&state, assets, "Card").await;
        let broker = create_child(&state, assets, "Broker").await;

        close_account(State(state.clone()), Path(card))
            .await
            .unwrap();
        let dto = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(card),
            Json(SetAccountParentRequest { parent_id: broker }),
        )
        .await
        .unwrap()
        .0;
        assert!(dto.closed_at.is_some());
        assert_eq!(dto.parent_id, Some(broker));
    }

    #[tokio::test]
    async fn set_parent_rejects_root_account() {
        let state = setup().await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        // 非系统根账户（repo 层直建）
        let custom_root = state
            .db()
            .account_create_with_name(
                &Account {
                    id: AccountId(0),
                    parent_id: None,
                    closed_at: None,
                    is_system: false,
                    billing_day: None,
                    repayment_day: None,
                },
                "CustomRoot",
                "en",
            )
            .await
            .unwrap();

        let result = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(custom_root.0),
            Json(SetAccountParentRequest {
                parent_id: expenses,
            }),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_parent_rejects_system_account() {
        let state = setup().await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        let cash = account_id_by_name(&state, "Assets:Cash").await;

        let result = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(cash),
            Json(SetAccountParentRequest {
                parent_id: expenses,
            }),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_parent_rejects_missing_target() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;

        let result = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(SetAccountParentRequest { parent_id: 99999 }),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_parent_rejects_self_and_descendant() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;
        let sub = create_child(&state, bank, "Sub").await;

        let to_self = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(SetAccountParentRequest { parent_id: bank }),
        )
        .await;
        assert!(to_self.is_err());

        let to_descendant = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(SetAccountParentRequest { parent_id: sub }),
        )
        .await;
        assert!(to_descendant.is_err());

        // 校验失败后结构不变
        let unchanged = state
            .db()
            .account_get(AccountId(bank))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(unchanged.parent_id, Some(AccountId(assets)));
    }

    #[tokio::test]
    async fn set_parent_rejects_duplicate_name_under_target() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        let _bank = create_child(&state, assets, "Bank").await;
        let other_bank = create_child(&state, expenses, "Bank").await;

        let result = set_parent(
            State(state.clone()),
            Lang("en".to_string()),
            Path(other_bank),
            Json(SetAccountParentRequest { parent_id: assets }),
        )
        .await;
        assert!(result.is_err());
    }
}
