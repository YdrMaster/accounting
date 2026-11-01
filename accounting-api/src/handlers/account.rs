//! 账户 API handler

use crate::dto::{
    AccountDto, CreateAccountRequest, RenameAccountRequest, SetAccountOwnersRequest,
    SetAccountParentRequest, UpdateAccountRequest,
};
use crate::handlers::{Lang, member::AppState};
use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, MemberId};
use accounting_sql::error::DbError;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
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
            .map(|ty| format!("{ty:?}"))
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
) -> Result<String, (StatusCode, String)> {
    let db = state.db();
    let bad_request = |msg: String| (StatusCode::BAD_REQUEST, msg);
    let target = db
        .account_get(AccountId(id))
        .await
        .map_err(|e| bad_request(e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            t!("account_not_found", locale = lang.as_str()).to_string(),
        ))?;
    // 同层级检查同名（命中任意语言的名字即视为占用）
    if let Some(dup) = db
        .account_get_by_parent_and_name(target.parent_id, &req.name)
        .await
        .map_err(|e| bad_request(e.to_string()))?
        && dup.id.0 != id
    {
        return Err(bad_request(
            t!("account_name_exists", locale = lang.as_str()).to_string(),
        ));
    }
    db.account_rename(AccountId(id), &req.name, &lang)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            // 系统根账户改名保护：按错误变体映射 400 + 本地化词条（不依赖本地化字面量）
            if matches!(e, DbError::SystemRootRenameProtected(_)) {
                bad_request(
                    t!("cannot_rename_system_root_account", locale = lang.as_str()).to_string(),
                )
            } else {
                bad_request(msg)
            }
        })?;
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

/// 删除账户（级联删除整棵干净子树）
async fn delete_account(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<String, (StatusCode, String)> {
    use accounting_service::account_service::DeleteAccountError;
    use accounting_sql::repo::account::AccountReferenceSource;

    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    if let Err(e) = service.delete(AccountId(id)).await {
        let status = match &e {
            DeleteAccountError::NotFound(_) => StatusCode::NOT_FOUND,
            DeleteAccountError::SystemAccount(_) => StatusCode::BAD_REQUEST,
            DeleteAccountError::Blocked { .. } => StatusCode::CONFLICT,
            DeleteAccountError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let msg = match e {
            DeleteAccountError::NotFound(_) => {
                t!("account_not_found", locale = lang.as_str()).to_string()
            }
            DeleteAccountError::SystemAccount(_) => {
                t!("cannot_delete_system_account", locale = lang.as_str()).to_string()
            }
            DeleteAccountError::Blocked { account_id, source } => {
                match db
                    .account_display_name(account_id, &lang)
                    .await
                    .ok()
                    .flatten()
                {
                    Some(name) => match source {
                        AccountReferenceSource::Posting => t!(
                            "account_subtree_blocked_by_posting",
                            locale = lang.as_str(),
                            name = name
                        )
                        .to_string(),
                        AccountReferenceSource::Mapping => t!(
                            "account_subtree_blocked_by_mapping",
                            locale = lang.as_str(),
                            name = name
                        )
                        .to_string(),
                        AccountReferenceSource::BudgetLimit => t!(
                            "account_subtree_blocked_by_budget",
                            locale = lang.as_str(),
                            name = name
                        )
                        .to_string(),
                        AccountReferenceSource::SavingPlan => t!(
                            "account_subtree_blocked_by_saving_plan",
                            locale = lang.as_str(),
                            name = name
                        )
                        .to_string(),
                        AccountReferenceSource::Channel => t!(
                            "account_subtree_blocked_by_channel",
                            locale = lang.as_str(),
                            name = name
                        )
                        .to_string(),
                    },
                    None => t!(
                        "account_delete_blocked_fallback",
                        locale = lang.as_str(),
                        id = account_id.0
                    )
                    .to_string(),
                }
            }
            DeleteAccountError::Database(_) => {
                t!("err_database", locale = lang.as_str()).to_string()
            }
        };
        return Err((status, msg));
    }
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
) -> Result<Json<AccountDto>, (StatusCode, String)> {
    let db = state.db();
    let service = accounting_service::account_service::AccountService::new(db.clone());
    service
        .reparent(AccountId(id), AccountId(req.parent_id), &lang)
        .await
        .map_err(|e| {
            let status = match &e {
                AccountingError::AccountAlreadyExists(_) => StatusCode::CONFLICT,
                AccountingError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            };
            (status, e.to_string())
        })?;

    // 返回更新后的账户（与列表端点的 DTO 口径一致）
    let server_error = |msg: String| (StatusCode::INTERNAL_SERVER_ERROR, msg);
    let account = db
        .account_get(AccountId(id))
        .await
        .map_err(|e| server_error(e.to_string()))?
        .ok_or_else(|| server_error(t!("account_not_found", locale = lang.as_str()).to_string()))?;
    let names = db
        .account_display_names(&[account.id], &lang)
        .await
        .map_err(|e| server_error(e.to_string()))?;
    let account_type = db
        .account_find_root_name(account.id, &lang)
        .await
        .ok()
        .and_then(|n| AccountType::from_str(&n).ok())
        .map(|ty| format!("{ty:?}"))
        .unwrap_or_else(|| "Asset".to_string());
    let owner_ids: Vec<i64> = db
        .account_get_owners(account.id)
        .await
        .map_err(|e| server_error(e.to_string()))?
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

    #[tokio::test]
    async fn rename_system_root_account_returns_400_en() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;

        let err = rename_account(
            State(state.clone()),
            Lang("en".to_string()),
            Path(assets),
            Json(RenameAccountRequest {
                name: "MyAssets".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(
            err.1.contains("System root account cannot be renamed"),
            "unexpected error: {}",
            err.1
        );

        // 根账户名保持不变
        let names = state
            .db()
            .account_display_names(&[AccountId(assets)], "en")
            .await
            .unwrap();
        assert_eq!(names.get(&AccountId(assets)).unwrap(), "Assets");
    }

    #[tokio::test]
    async fn rename_system_root_account_returns_400_zh() {
        let state = setup().await;
        let expenses = account_id_by_name(&state, "Expenses").await;

        let err = rename_account(
            State(state.clone()),
            Lang("zh-CN".to_string()),
            Path(expenses),
            Json(RenameAccountRequest {
                name: "花销".to_string(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(
            err.1.contains("系统根账户不可改名"),
            "unexpected error: {}",
            err.1
        );

        // 根账户名保持不变
        let names = state
            .db()
            .account_display_names(&[AccountId(expenses)], "zh-CN")
            .await
            .unwrap();
        assert_eq!(names.get(&AccountId(expenses)).unwrap(), "支出");
    }

    #[tokio::test]
    async fn rename_non_root_account_ok() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;

        rename_account(
            State(state.clone()),
            Lang("en".to_string()),
            Path(bank),
            Json(RenameAccountRequest {
                name: "NewBank".to_string(),
            }),
        )
        .await
        .unwrap();

        let names = state
            .db()
            .account_display_names(&[AccountId(bank)], "en")
            .await
            .unwrap();
        assert_eq!(names.get(&AccountId(bank)).unwrap(), "NewBank");
    }

    async fn set_parent_http(
        state: Arc<AppState>,
        id: i64,
        parent_id: i64,
    ) -> axum::http::Response<axum::body::Body> {
        let app = router().with_state(state);
        let req = axum::http::Request::builder()
            .method(axum::http::Method::PUT)
            .uri(format!("/api/accounts/{id}/parent?lang=en"))
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::json!({ "parent_id": parent_id }).to_string(),
            ))
            .unwrap();
        tower::ServiceExt::oneshot(app, req).await.unwrap()
    }

    #[tokio::test]
    async fn set_parent_http_conflict_names_duplicate_account() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        let bank = create_child(&state, assets, "Bank").await;
        // 目标父账户下已有同名子账户
        let _bank_dup = create_child(&state, expenses, "Bank").await;

        let resp = set_parent_http(state, bank, expenses).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            text.contains("Account already exists: Bank"),
            "conflict body should name the duplicate, got: {text}"
        );
    }

    #[tokio::test]
    async fn set_parent_http_rejects_root_with_bad_request() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let expenses = account_id_by_name(&state, "Expenses").await;

        let resp = set_parent_http(state, assets, expenses).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn set_parent_http_success_returns_updated_dto() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let expenses = account_id_by_name(&state, "Expenses").await;
        let bank = create_child(&state, assets, "Bank").await;

        let resp = set_parent_http(state, bank, expenses).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["parent_id"], serde_json::json!(expenses));
    }

    async fn delete_http(
        state: Arc<AppState>,
        id: i64,
        lang: &str,
    ) -> axum::http::Response<axum::body::Body> {
        let app = router().with_state(state);
        let req = axum::http::Request::builder()
            .method(axum::http::Method::DELETE)
            .uri(format!("/api/accounts/{id}?lang={lang}"))
            .body(axum::body::Body::empty())
            .unwrap();
        tower::ServiceExt::oneshot(app, req).await.unwrap()
    }

    async fn body_text(resp: axum::http::Response<axum::body::Body>) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// 在 `account_id` 上插一条分录引用（走公开 repo 门面，测试造数）。
    async fn insert_posting(state: &Arc<AppState>, account_id: i64) {
        let db = state.db();
        let member = db.member_get_or_create_by_name("甲", "en").await.unwrap();
        let cny = db.commodity_get_by_symbol("CNY").await.unwrap().unwrap().id;
        let tx_id = db
            .transaction_insert(
                &accounting::transaction::Transaction {
                    id: accounting::id::TransactionId(0),
                    date_time: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    description: "t".to_string(),
                    kind: accounting::transaction::TransactionKind::Normal,
                    member_id: member,
                },
                &[],
            )
            .await
            .unwrap();
        db.posting_insert(&accounting::posting::Posting {
            id: accounting::id::PostingId(0),
            transaction_id: tx_id,
            account_id: AccountId(account_id),
            commodity_id: cny,
            amount: rust_decimal::Decimal::ONE,
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: rust_decimal::Decimal::ZERO,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn delete_http_rejects_system_and_root_account() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let cash = account_id_by_name(&state, "Assets:Cash").await;

        // 根账户（is_system）删除被拒：非 2xx + 400 + 文案
        let resp = delete_http(state.clone(), assets, "en").await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let text = body_text(resp).await;
        assert!(
            text.contains("Cannot delete system account"),
            "unexpected body: {text}"
        );

        // 系统子账户同样被拒
        let resp = delete_http(state, cash, "en").await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_http_missing_account_returns_404() {
        let state = setup().await;
        let resp = delete_http(state, 999_999, "en").await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let text = body_text(resp).await;
        assert!(
            text.contains("Account not found"),
            "unexpected body: {text}"
        );
    }

    #[tokio::test]
    async fn delete_http_blocked_by_own_posting_returns_409_names_account() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;
        insert_posting(&state, bank).await;

        let resp = delete_http(state, bank, "en").await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let text = body_text(resp).await;
        assert!(
            text.contains("Bank") && text.contains("postings"),
            "conflict body should name the account and source, got: {text}"
        );
    }

    #[tokio::test]
    async fn delete_http_blocked_by_deep_descendant_names_descendant() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;
        let sub = create_child(&state, bank, "Sub").await;
        // 引用挂在深层后代 Sub 上，删除 Bank 被拒且指名 Sub
        insert_posting(&state, sub).await;

        let resp = delete_http(state.clone(), bank, "en").await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let text = body_text(resp).await;
        assert!(
            text.contains("Sub") && text.contains("postings"),
            "conflict body should name the descendant, got: {text}"
        );
        // 挡路后两账户都原样保留
        assert!(
            state
                .db()
                .account_get(AccountId(bank))
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            state
                .db()
                .account_get(AccountId(sub))
                .await
                .unwrap()
                .is_some()
        );
    }

    /// 在 `account_id` 上插一条 `kind` 类型引用（budget/saving_plan/mapping/channel，测试造数）。
    async fn insert_reference(state: &Arc<AppState>, kind: &str, account_id: i64) {
        let db = state.db();
        match kind {
            "budget" => {
                let cny = db.commodity_get_by_symbol("CNY").await.unwrap().unwrap().id;
                db.budget_create(
                    "预算",
                    None,
                    None,
                    cny,
                    &[(AccountId(account_id), rust_decimal::Decimal::ONE)],
                    "en",
                )
                .await
                .unwrap();
            }
            "saving_plan" => {
                let cny = db.commodity_get_by_symbol("CNY").await.unwrap().unwrap().id;
                db.saving_plan_create(
                    "攒钱",
                    None,
                    None,
                    cny,
                    rust_decimal::Decimal::ONE,
                    &[AccountId(account_id)],
                    "en",
                )
                .await
                .unwrap();
            }
            "mapping" => {
                let member = db.member_get_or_create_by_name("甲", "en").await.unwrap();
                let channel = db.channel_get_by_name("Alipay").await.unwrap().unwrap().id;
                db.account_mapping_upsert(&accounting::account_mapping::AccountMapping {
                    member_id: member,
                    channel_id: channel,
                    category: "收支:餐饮".to_string(),
                    account_id: AccountId(account_id),
                })
                .await
                .unwrap();
            }
            "channel" => {
                db.channel_create(&accounting::channel::Channel {
                    id: accounting::id::ChannelId(0),
                    description: None,
                    account_id: Some(AccountId(account_id)),
                    is_system: false,
                })
                .await
                .unwrap();
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn delete_http_blocked_by_budget_saving_plan_mapping_and_channel() {
        for (kind, word) in [
            ("budget", "budget limit"),
            ("saving_plan", "saving plan"),
            ("mapping", "account mappings"),
            ("channel", "channel"),
        ] {
            let state = setup().await;
            let assets = account_id_by_name(&state, "Assets").await;
            let bank = create_child(&state, assets, "Bank").await;
            let sub = create_child(&state, bank, "Sub").await;
            // 无分录时其余四类引用同样拦截（挂深层后代 Sub 上）
            insert_reference(&state, kind, sub).await;

            let resp = delete_http(state, bank, "en").await;
            assert_eq!(resp.status(), StatusCode::CONFLICT, "kind={kind}");
            let text = body_text(resp).await;
            assert!(
                text.contains("Sub") && text.contains(word),
                "kind={kind} body should name descendant and source, got: {text}"
            );
        }
    }

    #[tokio::test]
    async fn delete_http_blocked_reports_zh_message() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "银行").await;
        insert_posting(&state, bank).await;

        let resp = delete_http(state, bank, "zh-CN").await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let text = body_text(resp).await;
        assert!(
            text.contains("银行") && text.contains("关联分录"),
            "zh conflict body should name the account, got: {text}"
        );
    }

    #[tokio::test]
    async fn delete_http_cascades_clean_subtree() {
        let state = setup().await;
        let assets = account_id_by_name(&state, "Assets").await;
        let bank = create_child(&state, assets, "Bank").await;
        let sub = create_child(&state, bank, "Sub").await;
        let leaf = create_child(&state, sub, "Leaf").await;

        let resp = delete_http(state.clone(), bank, "en").await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 整棵子树消失，账户列表不再包含任何后代
        for id in [bank, sub, leaf] {
            assert!(
                state
                    .db()
                    .account_get(AccountId(id))
                    .await
                    .unwrap()
                    .is_none()
            );
        }
        let dtos = list_accounts(State(state), Lang("en".to_string()))
            .await
            .unwrap()
            .0;
        assert!(
            dtos.iter()
                .all(|d| d.id != bank && d.id != sub && d.id != leaf),
            "account list should not contain deleted subtree"
        );
    }
}
