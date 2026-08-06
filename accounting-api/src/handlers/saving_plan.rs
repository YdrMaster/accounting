//! 攒钱计划 API handler

use crate::dto::{
    AccountAllocationDto, CreateSavingPlanRequest, SavingPlanDetailDto, SavingPlanDto,
    SavingPlanStatusDto, UpdateSavingPlanRequest, parse_deadline, parse_period_opt,
    period_to_string,
};
use crate::handlers::{Lang, member::AppState};
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId, SavingPlanId};
use accounting_service::report::saving_plan::SavingPlanService;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;

/// API 错误响应
#[derive(Serialize)]
struct ApiError {
    error: String,
}

/// 攒钱计划 API 响应（支持不同 HTTP 状态码）
enum SavingPlanResponse {
    Created(Json<SavingPlanDto>),
    Ok(Json<serde_json::Value>),
    NotFound(String),
    BadRequest(String),
}

impl axum::response::IntoResponse for SavingPlanResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            SavingPlanResponse::Created(json) => (StatusCode::CREATED, json).into_response(),
            SavingPlanResponse::Ok(json) => (StatusCode::OK, json).into_response(),
            SavingPlanResponse::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(ApiError { error: msg })).into_response()
            }
            SavingPlanResponse::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(ApiError { error: msg })).into_response()
            }
        }
    }
}

fn map_error(e: AccountingError) -> SavingPlanResponse {
    let msg = e.to_string();
    // 仅「攒钱计划不存在」映射 404；账户/币种不存在等校验失败均为 400
    if msg.contains("攒钱计划不存在") {
        SavingPlanResponse::NotFound(msg)
    } else {
        SavingPlanResponse::BadRequest(msg)
    }
}

fn plan_to_dto(
    p: &accounting::saving_plan::SavingPlan,
    name: String,
    account_ids: Vec<i64>,
) -> SavingPlanDto {
    SavingPlanDto {
        id: p.id.0,
        name,
        period: period_to_string(p.period),
        deadline: p.deadline.map(|d| d.to_string()),
        commodity_id: p.commodity_id.0,
        target_amount: p.target_amount.to_string(),
        account_ids,
    }
}

/// 攒钱计划状态查询参数
#[derive(serde::Deserialize)]
pub struct SavingPlanStatusQuery {
    pub date: Option<String>,
}

/// 批量解析攒钱计划显示名
async fn plan_names(
    db: &accounting_sql::SqliteDatabase,
    ids: &[SavingPlanId],
    lang: &str,
) -> Result<std::collections::HashMap<SavingPlanId, String>, SavingPlanResponse> {
    db.saving_plan_display_names(ids, lang)
        .await
        .map_err(|e| SavingPlanResponse::BadRequest(e.to_string()))
}

/// 创建/更新请求解析后的公共字段
struct ParsedPlanRequest {
    period: Option<accounting::finance_period::FinancePeriod>,
    deadline: Option<chrono::NaiveDate>,
    target: Decimal,
    account_ids: Vec<AccountId>,
}

/// 解析请求公共字段：period/deadline/target_amount/account_ids
fn parse_request(
    period: Option<&str>,
    deadline: Option<&str>,
    target_amount: &str,
    account_ids: &[i64],
) -> Result<ParsedPlanRequest, String> {
    let period = parse_period_opt(period)?;
    let deadline = parse_deadline(deadline)?;
    let target = Decimal::from_str(target_amount).map_err(|e| format!("无效金额: {}", e))?;
    let ids: Vec<AccountId> = account_ids.iter().map(|&id| AccountId(id)).collect();
    Ok(ParsedPlanRequest {
        period,
        deadline,
        target,
        account_ids: ids,
    })
}

/// 列出所有攒钱计划
async fn list_saving_plans(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> SavingPlanResponse {
    let service = SavingPlanService::new(state.db.clone());
    match service.list_saving_plans().await {
        Ok(plans) => {
            let ids: Vec<SavingPlanId> = plans.iter().map(|p| p.id).collect();
            let names = match plan_names(&state.db, &ids, &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let mut dtos = Vec::new();
            for p in &plans {
                let accounts = match state.db.saving_plan_get_accounts(p.id).await {
                    Ok(a) => a,
                    Err(e) => return SavingPlanResponse::BadRequest(e.to_string()),
                };
                dtos.push(plan_to_dto(
                    p,
                    names.get(&p.id).cloned().unwrap_or_default(),
                    accounts.iter().map(|a| a.account_id.0).collect(),
                ));
            }
            SavingPlanResponse::Ok(Json(serde_json::to_value(dtos).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 创建攒钱计划
async fn create_saving_plan(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateSavingPlanRequest>,
) -> SavingPlanResponse {
    let parsed = match parse_request(
        req.period.as_deref(),
        req.deadline.as_deref(),
        &req.target_amount,
        &req.account_ids,
    ) {
        Ok(v) => v,
        Err(e) => return SavingPlanResponse::BadRequest(e),
    };
    let (period, deadline, target, account_ids) = (
        parsed.period,
        parsed.deadline,
        parsed.target,
        parsed.account_ids,
    );

    let service = SavingPlanService::new(state.db.clone());
    match service
        .create_saving_plan(
            &req.name,
            period,
            deadline,
            CommodityId(req.commodity_id),
            target,
            &account_ids,
            &lang,
        )
        .await
    {
        Ok(id) => {
            let dto = SavingPlanDto {
                id: id.0,
                name: req.name,
                period: period_to_string(period),
                deadline: deadline.map(|d| d.to_string()),
                commodity_id: req.commodity_id,
                target_amount: target.to_string(),
                account_ids: account_ids.iter().map(|a| a.0).collect(),
            };
            SavingPlanResponse::Created(Json(dto))
        }
        Err(e) => map_error(e),
    }
}

/// 获取攒钱计划详情
async fn get_saving_plan_detail(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> SavingPlanResponse {
    let service = SavingPlanService::new(state.db.clone());
    match service.get_saving_plan_detail(SavingPlanId(id)).await {
        Ok(detail) => {
            let names = match plan_names(&state.db, &[detail.plan.id], &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let account_ids: Vec<i64> = detail.account_ids.iter().map(|a| a.0).collect();
            let dto = SavingPlanDetailDto {
                plan: plan_to_dto(
                    &detail.plan,
                    names.get(&detail.plan.id).cloned().unwrap_or_default(),
                    account_ids.clone(),
                ),
                account_ids,
            };
            SavingPlanResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 更新攒钱计划
async fn update_saving_plan(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSavingPlanRequest>,
) -> SavingPlanResponse {
    let parsed = match parse_request(
        req.period.as_deref(),
        req.deadline.as_deref(),
        &req.target_amount,
        &req.account_ids,
    ) {
        Ok(v) => v,
        Err(e) => return SavingPlanResponse::BadRequest(e),
    };
    let (period, deadline, target, account_ids) = (
        parsed.period,
        parsed.deadline,
        parsed.target,
        parsed.account_ids,
    );

    let service = SavingPlanService::new(state.db.clone());
    match service
        .update_saving_plan(
            SavingPlanId(id),
            &req.name,
            period,
            deadline,
            CommodityId(req.commodity_id),
            target,
            &account_ids,
            &lang,
        )
        .await
    {
        Ok(()) => {
            let dto = SavingPlanDto {
                id,
                name: req.name,
                period: period_to_string(period),
                deadline: deadline.map(|d| d.to_string()),
                commodity_id: req.commodity_id,
                target_amount: target.to_string(),
                account_ids: account_ids.iter().map(|a| a.0).collect(),
            };
            SavingPlanResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 删除攒钱计划
async fn delete_saving_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> SavingPlanResponse {
    let service = SavingPlanService::new(state.db.clone());
    // 先确认存在，保持与详情/更新一致的 404 语义
    match service.get_saving_plan_detail(SavingPlanId(id)).await {
        Ok(_) => {}
        Err(e) => return map_error(e),
    }
    match service.delete_saving_plan(SavingPlanId(id)).await {
        Ok(()) => SavingPlanResponse::Ok(Json(serde_json::json!({"deleted": true}))),
        Err(e) => map_error(e),
    }
}

/// 攒钱计划状态转 DTO（单条与批量端点共用，保证序列化口径一致）
fn status_to_dto(
    status: &accounting_service::report::saving_plan::SavingPlanStatus,
    name: String,
    account_ids: Vec<i64>,
) -> SavingPlanStatusDto {
    SavingPlanStatusDto {
        plan: plan_to_dto(&status.plan, name, account_ids),
        expired: status.expired,
        period_start: status.period_start.map(|d| d.to_string()),
        period_end: status.period_end.map(|d| d.to_string()),
        target_amount: status.target_amount.to_string(),
        current_balance: status.current_balance.to_string(),
        gap: status.gap.to_string(),
        met: status.met,
        allocated: status.allocated.to_string(),
        // satisfaction 为计算比率，除法/乘法会引入尾随零（如 75.00），归一化去掉
        satisfaction: status.satisfaction.normalize().to_string(),
        accounts: status
            .accounts
            .iter()
            .map(|a| AccountAllocationDto {
                account_id: a.account_id.0,
                balance: a.balance.to_string(),
                occupied_by_earlier: a.occupied_by_earlier.to_string(),
                allocated: a.allocated.to_string(),
            })
            .collect(),
    }
}

/// 解析状态查询的 date 参数（缺省当天；格式无效返回错误信息）
fn parse_status_date(query: &SavingPlanStatusQuery) -> Result<chrono::NaiveDate, String> {
    match query.date {
        Some(ref d) => {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| format!("无效日期: {}", e))
        }
        None => Ok(chrono::Local::now().date_naive()),
    }
}

/// 查询攒钱计划状态
async fn get_saving_plan_status(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Query(query): Query<SavingPlanStatusQuery>,
) -> SavingPlanResponse {
    let date = match parse_status_date(&query) {
        Ok(d) => d,
        Err(e) => return SavingPlanResponse::BadRequest(e),
    };

    let service = SavingPlanService::new(state.db.clone());
    match service.get_saving_plan_status(SavingPlanId(id), date).await {
        Ok(status) => {
            let names = match plan_names(&state.db, &[status.plan.id], &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let account_ids = match state.db.saving_plan_get_accounts(status.plan.id).await {
                Ok(a) => a.iter().map(|x| x.account_id.0).collect(),
                Err(e) => return SavingPlanResponse::BadRequest(e.to_string()),
            };
            let dto = status_to_dto(
                &status,
                names.get(&status.plan.id).cloned().unwrap_or_default(),
                account_ids,
            );
            SavingPlanResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 查询全部攒钱计划状态
///
/// 参与全局分配的计划按（检查点, plan_id）升序在前，过期/永久计划在后。
async fn list_saving_plan_statuses(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Query(query): Query<SavingPlanStatusQuery>,
) -> SavingPlanResponse {
    let date = match parse_status_date(&query) {
        Ok(d) => d,
        Err(e) => return SavingPlanResponse::BadRequest(e),
    };

    let service = SavingPlanService::new(state.db.clone());
    match service.list_saving_plan_statuses(date).await {
        Ok(statuses) => {
            let ids: Vec<SavingPlanId> = statuses.iter().map(|s| s.plan.id).collect();
            let names = match plan_names(&state.db, &ids, &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let mut dtos = Vec::with_capacity(statuses.len());
            for status in &statuses {
                let account_ids = match state.db.saving_plan_get_accounts(status.plan.id).await {
                    Ok(a) => a.iter().map(|x| x.account_id.0).collect(),
                    Err(e) => return SavingPlanResponse::BadRequest(e.to_string()),
                };
                dtos.push(status_to_dto(
                    status,
                    names.get(&status.plan.id).cloned().unwrap_or_default(),
                    account_ids,
                ));
            }
            SavingPlanResponse::Ok(Json(serde_json::to_value(dtos).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 攒钱计划路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/saving-plans",
            get(list_saving_plans).post(create_saving_plan),
        )
        // 静态段优先于 /{id}，不受注册顺序影响（axum 0.8）
        .route("/api/saving-plans/statuses", get(list_saving_plan_statuses))
        .route(
            "/api/saving-plans/{id}",
            get(get_saving_plan_detail)
                .put(update_saving_plan)
                .delete(delete_saving_plan),
        )
        .route("/api/saving-plans/{id}/status", get(get_saving_plan_status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting_sql::SqliteDatabase;
    use axum::response::IntoResponse;
    use chrono::NaiveDateTime;

    /// 建库并在 Assets 下建 Alipay/WeChat/Bank，Expenses 下建 Food
    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let bare = |parent| Account {
            id: AccountId(0),
            parent_id: Some(parent),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        db.account_create_with_name(&bare(assets_id), "Alipay", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(assets_id), "WeChat", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(assets_id), "Bank", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(expenses_id), "Food", "en")
            .await
            .unwrap();
        Arc::new(AppState { db })
    }

    async fn account_id(state: &Arc<AppState>, name: &str) -> i64 {
        state
            .db()
            .account_get_by_name(name)
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    async fn respond(resp: SavingPlanResponse) -> (StatusCode, serde_json::Value) {
        let r = resp.into_response();
        let status = r.status();
        let bytes = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    /// 插入一笔单分录交易（仅用于构造余额）
    async fn add_posting(state: &Arc<AppState>, date: &str, account_id: i64, amount: &str) {
        let db = state.db();
        let member_id = db.member_get_or_create_by_name("Test", "en").await.unwrap();
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDateTime::parse_from_str(
                &format!("{} 00:00:00", date),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            description: "test".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let tx_id = db.transaction_insert(&tx, &[]).await.unwrap();
        let posting = Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id: AccountId(account_id),
            commodity_id: CommodityId(1),
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        db.posting_insert(&posting).await.unwrap();
    }

    /// 按 spec 示例 body 创建一次性攒钱计划，返回 (status, json)
    async fn create_travel_fund(state: &Arc<AppState>) -> (StatusCode, serde_json::Value) {
        let alipay = account_id(state, "Assets:Alipay").await;
        let wechat = account_id(state, "Assets:WeChat").await;
        let body = format!(
            r#"{{"name":"旅行基金","period":null,"deadline":"2026-09-30","commodity_id":1,"target_amount":"5000","account_ids":[{},{}]}}"#,
            alipay, wechat
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        respond(create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await)
            .await
    }

    /// 快捷创建一次性攒钱计划（deadline 决定全局分配检查点顺序），返回创建响应 json
    async fn create_plan(
        state: &Arc<AppState>,
        name: &str,
        deadline: &str,
        target: &str,
        account_ids: &[i64],
    ) -> serde_json::Value {
        let ids = account_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let body = format!(
            r#"{{"name":"{}","period":null,"deadline":"{}","commodity_id":1,"target_amount":"{}","account_ids":[{}]}}"#,
            name, deadline, target, ids
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        json
    }

    /// 查询状态并断言 200，返回响应 json
    async fn status_json(state: &Arc<AppState>, id: i64, date: &str) -> serde_json::Value {
        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some(date.to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        json
    }

    // === 列表 ===

    #[tokio::test]
    async fn list_saving_plans_empty_returns_empty_array() {
        let state = setup().await;
        let (status, json) =
            respond(list_saving_plans(State(state.clone()), Lang("en".to_string())).await).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json, serde_json::json!([]));
    }

    #[tokio::test]
    async fn list_saving_plans_returns_all() {
        let state = setup().await;
        create_travel_fund(&state).await;
        let bank = account_id(&state, "Assets:Bank").await;
        let body = format!(
            r#"{{"name":"房租备用金","period":"monthly","deadline":null,"commodity_id":1,"target_amount":"6000","account_ids":[{}]}}"#,
            bank
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (created_status, _) = respond(
            create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(created_status, StatusCode::CREATED);

        let (status, json) =
            respond(list_saving_plans(State(state.clone()), Lang("zh".to_string())).await).await;
        assert_eq!(status, StatusCode::OK);
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // 循环计划 DTO：period 为字符串、deadline 为 null
        let monthly = arr.iter().find(|p| p["period"] == "monthly").unwrap();
        assert_eq!(monthly["deadline"], serde_json::Value::Null);
        assert_eq!(monthly["target_amount"], "6000");
        assert_eq!(monthly["account_ids"].as_array().unwrap().len(), 1);
    }

    // === 创建 ===

    #[tokio::test]
    async fn create_saving_plan_success_one_off() {
        let state = setup().await;
        let (status, json) = create_travel_fund(&state).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["name"], "旅行基金");
        assert_eq!(json["period"], serde_json::Value::Null);
        assert_eq!(json["deadline"], "2026-09-30");
        assert_eq!(json["commodity_id"], 1);
        assert_eq!(json["target_amount"], "5000");
        assert_eq!(json["account_ids"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn create_saving_plan_empty_name_rejected() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let body = format!(
            r#"{{"name":"","period":null,"deadline":null,"commodity_id":1,"target_amount":"100","account_ids":[{}]}}"#,
            alipay
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("不能为空"));
    }

    #[tokio::test]
    async fn create_saving_plan_empty_accounts_rejected() {
        let state = setup().await;
        let body = r#"{"name":"测试","period":null,"deadline":null,"commodity_id":1,"target_amount":"100","account_ids":[]}"#;
        let req: CreateSavingPlanRequest = serde_json::from_str(body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("账户集合不能为空"));
    }

    #[tokio::test]
    async fn create_saving_plan_expense_account_rejected() {
        let state = setup().await;
        let food = account_id(&state, "Expenses:Food").await;
        let body = format!(
            r#"{{"name":"测试","period":null,"deadline":null,"commodity_id":1,"target_amount":"100","account_ids":[{}]}}"#,
            food
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].is_string());
    }

    #[tokio::test]
    async fn create_saving_plan_commodity_not_found_rejected() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let body = format!(
            r#"{{"name":"测试","period":null,"deadline":null,"commodity_id":9999,"target_amount":"100","account_ids":[{}]}}"#,
            alipay
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].is_string());
    }

    #[tokio::test]
    async fn create_saving_plan_invalid_period_rejected() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let body = format!(
            r#"{{"name":"测试","period":"fortnightly","deadline":null,"commodity_id":1,"target_amount":"100","account_ids":[{}]}}"#,
            alipay
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("无效周期类型"));
    }

    #[tokio::test]
    async fn create_saving_plan_invalid_deadline_rejected() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let body = format!(
            r#"{{"name":"测试","period":null,"deadline":"not-a-date","commodity_id":1,"target_amount":"100","account_ids":[{}]}}"#,
            alipay
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("en".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("无效日期"));
    }

    // === 详情 ===

    #[tokio::test]
    async fn get_saving_plan_detail_returns_plan_and_accounts() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let wechat = account_id(&state, "Assets:WeChat").await;
        let bank = account_id(&state, "Assets:Bank").await;
        let body = format!(
            r#"{{"name":"三账户计划","period":null,"deadline":null,"commodity_id":1,"target_amount":"1000","account_ids":[{},{},{}]}}"#,
            alipay, wechat, bank
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (_, created) = respond(
            create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await,
        )
        .await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_saving_plan_detail(State(state.clone()), Lang("zh".to_string()), Path(id)).await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["plan"]["id"], id);
        assert_eq!(json["plan"]["name"], "三账户计划");
        assert_eq!(json["account_ids"].as_array().unwrap().len(), 3);
        assert_eq!(json["plan"]["account_ids"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn get_saving_plan_detail_not_found() {
        let state = setup().await;
        let (status, json) = respond(
            get_saving_plan_detail(State(state.clone()), Lang("en".to_string()), Path(999)).await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("不存在"));
    }

    // === 更新 ===

    #[tokio::test]
    async fn update_saving_plan_success() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let bank = account_id(&state, "Assets:Bank").await;

        let body = format!(
            r#"{{"name":"欧洲旅行基金","period":null,"deadline":"2026-12-31","commodity_id":1,"target_amount":"8000","account_ids":[{}]}}"#,
            bank
        );
        let req: UpdateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            update_saving_plan(
                State(state.clone()),
                Lang("zh".to_string()),
                Path(id),
                Json(req),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["name"], "欧洲旅行基金");
        assert_eq!(json["target_amount"], "8000");
        assert_eq!(json["deadline"], "2026-12-31");
        assert_eq!(json["account_ids"].as_array().unwrap().len(), 1);

        // 详情中账户集合确已替换
        let (_, detail) = respond(
            get_saving_plan_detail(State(state.clone()), Lang("zh".to_string()), Path(id)).await,
        )
        .await;
        assert_eq!(detail["account_ids"].as_array().unwrap().len(), 1);
        assert_eq!(detail["plan"]["name"], "欧洲旅行基金");
    }

    #[tokio::test]
    async fn update_saving_plan_not_found() {
        let state = setup().await;
        let bank = account_id(&state, "Assets:Bank").await;
        let body = format!(
            r#"{{"name":"x","period":null,"deadline":null,"commodity_id":1,"target_amount":"100","account_ids":[{}]}}"#,
            bank
        );
        let req: UpdateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            update_saving_plan(
                State(state.clone()),
                Lang("en".to_string()),
                Path(999),
                Json(req),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("不存在"));
    }

    #[tokio::test]
    async fn update_saving_plan_duplicate_accounts_rejected() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let alipay = account_id(&state, "Assets:Alipay").await;
        let body = format!(
            r#"{{"name":"旅行基金","period":null,"deadline":null,"commodity_id":1,"target_amount":"5000","account_ids":[{},{}]}}"#,
            alipay, alipay
        );
        let req: UpdateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            update_saving_plan(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Json(req),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].is_string());
    }

    #[tokio::test]
    async fn update_saving_plan_expense_account_rejected() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let food = account_id(&state, "Expenses:Food").await;
        let body = format!(
            r#"{{"name":"旅行基金","period":null,"deadline":null,"commodity_id":1,"target_amount":"5000","account_ids":[{}]}}"#,
            food
        );
        let req: UpdateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, _) = respond(
            update_saving_plan(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Json(req),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // === 删除 ===

    #[tokio::test]
    async fn delete_saving_plan_success() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();

        let (status, _) = respond(delete_saving_plan(State(state.clone()), Path(id)).await).await;
        assert_eq!(status, StatusCode::OK);

        // 删除后详情返回 404，列表为空
        let (get_status, _) = respond(
            get_saving_plan_detail(State(state.clone()), Lang("en".to_string()), Path(id)).await,
        )
        .await;
        assert_eq!(get_status, StatusCode::NOT_FOUND);
        let (_, list) =
            respond(list_saving_plans(State(state.clone()), Lang("en".to_string())).await).await;
        assert_eq!(list, serde_json::json!([]));
    }

    #[tokio::test]
    async fn delete_saving_plan_not_found() {
        let state = setup().await;
        let (status, json) =
            respond(delete_saving_plan(State(state.clone()), Path(999)).await).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("不存在"));
    }

    // === 状态 ===

    #[tokio::test]
    async fn status_unmet_returns_gap() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let alipay = account_id(&state, "Assets:Alipay").await;
        add_posting(&state, "2026-06-01", alipay, "3200").await;

        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-06-26".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["expired"], false);
        assert_eq!(json["target_amount"], "5000");
        assert_eq!(json["current_balance"], "3200");
        assert_eq!(json["gap"], "1800");
        assert_eq!(json["met"], false);
        assert!(json["plan"].is_object());
    }

    #[tokio::test]
    async fn status_met_returns_negative_gap() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let alipay = account_id(&state, "Assets:Alipay").await;
        add_posting(&state, "2026-06-01", alipay, "5300").await;

        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-06-26".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["current_balance"], "5300");
        assert_eq!(json["gap"], "-300");
        assert_eq!(json["met"], true);
    }

    #[tokio::test]
    async fn status_respects_query_date() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let alipay = account_id(&state, "Assets:Alipay").await;
        add_posting(&state, "2026-06-01", alipay, "4000").await;
        add_posting(&state, "2026-09-15", alipay, "2000").await;

        // 查询日 2026-09-30：两笔都计入
        let (_, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-09-30".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(json["current_balance"], "6000");
        assert_eq!(json["met"], true);

        // 查询日 2026-06-26：只计入第一笔
        let (_, json2) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-06-26".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(json2["current_balance"], "4000");
        assert_eq!(json2["met"], false);
    }

    #[tokio::test]
    async fn status_expired_plan_returns_200_with_expired_true() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();
        let alipay = account_id(&state, "Assets:Alipay").await;
        add_posting(&state, "2026-06-01", alipay, "5200").await;

        // deadline=2026-09-30，查询 2026-10-15：已失效但其余字段正常返回
        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-10-15".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["expired"], true);
        assert_eq!(json["current_balance"], "5200");
        assert_eq!(json["gap"], "-200");
        assert_eq!(json["met"], true);
        // 过期计划的分配字段也完整返回（无竞争退化口径：不视为被占用）
        assert_eq!(json["allocated"], "5000");
        assert_eq!(json["satisfaction"], "100");
        let accounts = json["accounts"].as_array().unwrap();
        assert_eq!(accounts.len(), 2);
        let alipay_entry = accounts.iter().find(|x| x["account_id"] == alipay).unwrap();
        assert_eq!(alipay_entry["balance"], "5200");
        assert_eq!(alipay_entry["occupied_by_earlier"], "0");
        assert_eq!(alipay_entry["allocated"], "5000");
    }

    #[tokio::test]
    async fn status_one_off_plan_period_fields_are_null() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-06-26".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["period_start"], serde_json::Value::Null);
        assert_eq!(json["period_end"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn status_monthly_plan_period_fields_are_strings() {
        let state = setup().await;
        let bank = account_id(&state, "Assets:Bank").await;
        let body = format!(
            r#"{{"name":"房租备用金","period":"monthly","deadline":null,"commodity_id":1,"target_amount":"6000","account_ids":[{}]}}"#,
            bank
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (_, created) = respond(
            create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await,
        )
        .await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("2026-06-26".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["period_start"], "2026-06-01");
        assert_eq!(json["period_end"], "2026-06-30");
    }

    #[tokio::test]
    async fn status_not_found() {
        let state = setup().await;
        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(999),
                Query(SavingPlanStatusQuery { date: None }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("不存在"));
    }

    // === 状态：全局分配字段 ===

    #[tokio::test]
    async fn status_account_allocation_serialization() {
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        // 更早检查点的计划先占用 Alipay 2000
        create_plan(&state, "更早计划", "2026-08-31", "2000", &[alipay]).await;
        let later = create_plan(&state, "本计划", "2026-09-30", "1000", &[alipay]).await;
        let id = later["id"].as_i64().unwrap();
        add_posting(&state, "2026-06-01", alipay, "3000").await;

        // spec「账户明细序列化」：A 余额 3000、被更早计划占用 2000、本计划分配 1000
        let json = status_json(&state, id, "2026-06-26").await;
        assert_eq!(json["allocated"], "1000");
        assert_eq!(json["satisfaction"], "100");
        let accounts = json["accounts"].as_array().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0]["account_id"], alipay);
        assert_eq!(accounts[0]["balance"], "3000");
        assert_eq!(accounts[0]["occupied_by_earlier"], "2000");
        assert_eq!(accounts[0]["allocated"], "1000");
    }

    #[tokio::test]
    async fn status_shared_account_satisfaction_reflects_occupation() {
        let state = setup().await;
        let a = account_id(&state, "Assets:Alipay").await;
        let b = account_id(&state, "Assets:WeChat").await;
        let e = account_id(&state, "Assets:Bank").await;
        // spec「共享账户的计划满足率反映占用」：
        // 计划1 {A,B} 目标 3000 检查点早于 计划2 {A,E} 目标 2000；A 3000、B 1000、E 500
        let p1 = create_plan(&state, "计划1", "2026-08-31", "3000", &[a, b]).await;
        let p2 = create_plan(&state, "计划2", "2026-09-30", "2000", &[a, e]).await;
        add_posting(&state, "2026-06-01", a, "3000").await;
        add_posting(&state, "2026-06-01", b, "1000").await;
        add_posting(&state, "2026-06-01", e, "500").await;

        let j1 = status_json(&state, p1["id"].as_i64().unwrap(), "2026-06-26").await;
        assert_eq!(j1["allocated"], "3000");
        assert_eq!(j1["satisfaction"], "100");

        let j2 = status_json(&state, p2["id"].as_i64().unwrap(), "2026-06-26").await;
        assert_eq!(j2["allocated"], "1500");
        assert_eq!(j2["satisfaction"], "75");
        // 共享账户 A：被计划1占用 2000，本计划仅分到剩余 1000
        let accounts = j2["accounts"].as_array().unwrap();
        assert_eq!(accounts.len(), 2);
        let a_entry = accounts.iter().find(|x| x["account_id"] == a).unwrap();
        assert_eq!(a_entry["balance"], "3000");
        assert_eq!(a_entry["occupied_by_earlier"], "2000");
        assert_eq!(a_entry["allocated"], "1000");
        let e_entry = accounts.iter().find(|x| x["account_id"] == e).unwrap();
        assert_eq!(e_entry["balance"], "500");
        assert_eq!(e_entry["occupied_by_earlier"], "0");
        assert_eq!(e_entry["allocated"], "500");
    }

    #[tokio::test]
    async fn status_invalid_date_rejected() {
        let state = setup().await;
        let (_, created) = create_travel_fund(&state).await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_saving_plan_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(SavingPlanStatusQuery {
                    date: Some("invalid".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("无效日期"));
    }

    // === 批量状态 ===

    /// 查询批量状态，返回 (status, json)
    async fn statuses_json(
        state: &Arc<AppState>,
        date: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        respond(
            list_saving_plan_statuses(
                State(state.clone()),
                Lang("en".to_string()),
                Query(SavingPlanStatusQuery {
                    date: date.map(|s| s.to_string()),
                }),
            )
            .await,
        )
        .await
    }

    /// 创建永久计划（period/deadline 皆空，不参与全局分配）
    async fn create_permanent_plan(
        state: &Arc<AppState>,
        name: &str,
        target: &str,
        account_ids: &[i64],
    ) -> serde_json::Value {
        let ids = account_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let body = format!(
            r#"{{"name":"{}","period":null,"deadline":null,"commodity_id":1,"target_amount":"{}","account_ids":[{}]}}"#,
            name, target, ids
        );
        let req: CreateSavingPlanRequest = serde_json::from_str(&body).unwrap();
        let (status, json) = respond(
            create_saving_plan(State(state.clone()), Lang("zh".to_string()), Json(req)).await,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        json
    }

    #[tokio::test]
    async fn statuses_returns_all_plans() {
        // spec「批量返回全部计划状态」：3 个计划 → 200 + 3 个 DTO，含 allocated/satisfaction/accounts
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        create_travel_fund(&state).await;
        create_plan(&state, "换新手机", "2026-12-31", "8000", &[alipay]).await;
        create_permanent_plan(&state, "应急金", "10000", &[alipay]).await;

        let (status, json) = statuses_json(&state, Some("2026-06-26")).await;
        assert_eq!(status, StatusCode::OK);
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        for dto in arr {
            assert!(dto["plan"].is_object());
            assert!(dto["allocated"].is_string());
            assert!(dto["satisfaction"].is_string());
            assert!(dto["accounts"].is_array());
        }
    }

    #[tokio::test]
    async fn statuses_sorted_by_checkpoint() {
        // spec「按检查点升序排列」：计划1 检查点 2026-09-30，计划2 检查点 2026-07-31 → 计划2 在前；
        // 不参与分配的永久计划排在最后
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let p1 = create_plan(&state, "计划1", "2026-09-30", "1000", &[alipay]).await;
        let permanent = create_permanent_plan(&state, "永久计划", "1000", &[alipay]).await;
        let p2 = create_plan(&state, "计划2", "2026-07-31", "1000", &[alipay]).await;

        let (status, json) = statuses_json(&state, Some("2026-06-26")).await;
        assert_eq!(status, StatusCode::OK);
        let ids: Vec<i64> = json
            .as_array()
            .unwrap()
            .iter()
            .map(|dto| dto["plan"]["id"].as_i64().unwrap())
            .collect();
        assert_eq!(
            ids,
            vec![
                p2["id"].as_i64().unwrap(),
                p1["id"].as_i64().unwrap(),
                permanent["id"].as_i64().unwrap()
            ]
        );
    }

    #[tokio::test]
    async fn statuses_matches_single() {
        // spec「批量与单条口径一致」：同一计划的 allocated、satisfaction 完全一致
        let state = setup().await;
        let alipay = account_id(&state, "Assets:Alipay").await;
        let wechat = account_id(&state, "Assets:WeChat").await;
        let p1 = create_plan(&state, "计划1", "2026-08-31", "3000", &[alipay, wechat]).await;
        let p2 = create_plan(&state, "计划2", "2026-09-30", "2000", &[alipay]).await;
        add_posting(&state, "2026-06-01", alipay, "3000").await;
        add_posting(&state, "2026-06-01", wechat, "1000").await;

        let (status, json) = statuses_json(&state, Some("2026-06-26")).await;
        assert_eq!(status, StatusCode::OK);
        for dto in json.as_array().unwrap() {
            let id = dto["plan"]["id"].as_i64().unwrap();
            assert!(id == p1["id"].as_i64().unwrap() || id == p2["id"].as_i64().unwrap());
            let single = status_json(&state, id, "2026-06-26").await;
            assert_eq!(dto["allocated"], single["allocated"]);
            assert_eq!(dto["satisfaction"], single["satisfaction"]);
        }
    }

    #[tokio::test]
    async fn statuses_empty_returns_empty_array() {
        // spec「无计划时返回空数组」
        let state = setup().await;
        let (status, json) = statuses_json(&state, Some("2026-06-26")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json, serde_json::json!([]));
    }

    #[tokio::test]
    async fn statuses_invalid_date_returns_400() {
        // spec「日期格式无效」
        let state = setup().await;
        let (status, json) = statuses_json(&state, Some("invalid")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("无效日期"));
    }

    #[tokio::test]
    async fn router_statuses_route_takes_priority_over_id() {
        // 静态段 /api/saving-plans/statuses 优先于 /api/saving-plans/{id}：
        // 空库走批量端点应返回 200 []（若被 {id} 捕获则 "statuses" 解析 i64 失败）
        let state = setup().await;
        let app = router().with_state(state);
        let req = axum::http::Request::builder()
            .uri("/api/saving-plans/statuses")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json, serde_json::json!([]));
    }
}
