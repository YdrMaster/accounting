//! 预算 API handler

use crate::dto::{
    BudgetDetailDto, BudgetDto, BudgetItemStatusDto, BudgetLimitDto, BudgetLimitRequest,
    BudgetStatusDto, CreateBudgetRequest, UpdateBudgetRequest, parse_deadline, parse_period_opt,
    period_to_string,
};
use crate::handlers::{Lang, member::AppState};
use accounting::error::AccountingError;
use accounting::id::{AccountId, BudgetId, CommodityId};
use accounting_service::report::budget::BudgetService;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use rust_decimal::Decimal;
use serde::Serialize;
use std::sync::Arc;

/// API 错误响应
#[derive(Serialize)]
struct ApiError {
    error: String,
}

/// 预算 API 响应（支持不同 HTTP 状态码）
enum BudgetResponse {
    Created(Json<BudgetDto>),
    Ok(Json<serde_json::Value>),
    NotFound(String),
    BadRequest(String),
}

impl axum::response::IntoResponse for BudgetResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            BudgetResponse::Created(json) => (StatusCode::CREATED, json).into_response(),
            BudgetResponse::Ok(json) => (StatusCode::OK, json).into_response(),
            BudgetResponse::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(ApiError { error: msg })).into_response()
            }
            BudgetResponse::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(ApiError { error: msg })).into_response()
            }
        }
    }
}

fn map_error(e: AccountingError) -> BudgetResponse {
    let msg = e.to_string();
    // 仅「预算表不存在」映射 404；账户/币种不存在等校验失败均为 400
    if msg.contains("预算表不存在") {
        BudgetResponse::NotFound(msg)
    } else {
        BudgetResponse::BadRequest(msg)
    }
}

fn budget_to_dto(b: &accounting::budget::Budget, name: String) -> BudgetDto {
    BudgetDto {
        id: b.id.0,
        name,
        period: period_to_string(b.period),
        deadline: b.deadline.map(|d| d.to_string()),
        commodity_id: b.commodity_id.0,
    }
}

fn parse_limits(limits: &[BudgetLimitRequest]) -> Result<Vec<(AccountId, Decimal)>, String> {
    limits
        .iter()
        .map(|l| {
            let amount = Decimal::from_str(&l.amount).map_err(|e| format!("无效金额: {}", e))?;
            Ok((AccountId(l.account_id), amount))
        })
        .collect()
}

use std::str::FromStr;

/// 预算列表查询参数
#[derive(serde::Deserialize)]
pub struct BudgetStatusQuery {
    pub date: Option<String>,
}

/// 批量解析预算显示名
async fn budget_names(
    db: &accounting_sql::SqliteDatabase,
    ids: &[BudgetId],
    lang: &str,
) -> Result<std::collections::HashMap<BudgetId, String>, BudgetResponse> {
    db.budget_display_names(ids, lang)
        .await
        .map_err(|e| BudgetResponse::BadRequest(e.to_string()))
}

/// 列出所有预算表
async fn list_budgets(State(state): State<Arc<AppState>>, Lang(lang): Lang) -> BudgetResponse {
    let service = BudgetService::new(state.db.clone());
    match service.list_budgets().await {
        Ok(budgets) => {
            let ids: Vec<BudgetId> = budgets.iter().map(|b| b.id).collect();
            let names = match budget_names(&state.db, &ids, &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let dtos: Vec<BudgetDto> = budgets
                .iter()
                .map(|b| budget_to_dto(b, names.get(&b.id).cloned().unwrap_or_default()))
                .collect();
            BudgetResponse::Ok(Json(serde_json::to_value(dtos).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 创建预算表
async fn create_budget(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateBudgetRequest>,
) -> BudgetResponse {
    let period = match parse_period_opt(req.period.as_deref()) {
        Ok(p) => p,
        Err(e) => return BudgetResponse::BadRequest(e),
    };
    let deadline = match parse_deadline(req.deadline.as_deref()) {
        Ok(d) => d,
        Err(e) => return BudgetResponse::BadRequest(e),
    };
    let limits = match parse_limits(&req.limits) {
        Ok(l) => l,
        Err(e) => return BudgetResponse::BadRequest(e),
    };

    let service = BudgetService::new(state.db.clone());
    match service
        .create_budget(
            &req.name,
            period,
            deadline,
            CommodityId(req.commodity_id),
            &limits,
            &lang,
        )
        .await
    {
        Ok(id) => {
            let dto = BudgetDto {
                id: id.0,
                name: req.name,
                period: period_to_string(period),
                deadline: deadline.map(|d| d.to_string()),
                commodity_id: req.commodity_id,
            };
            BudgetResponse::Created(Json(dto))
        }
        Err(e) => map_error(e),
    }
}

/// 获取预算表详情
async fn get_budget_detail(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> BudgetResponse {
    let service = BudgetService::new(state.db.clone());
    match service.get_budget_detail(BudgetId(id)).await {
        Ok(detail) => {
            let names = match budget_names(&state.db, &[detail.budget.id], &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let dto = BudgetDetailDto {
                budget: budget_to_dto(
                    &detail.budget,
                    names.get(&detail.budget.id).cloned().unwrap_or_default(),
                ),
                limits: detail
                    .limits
                    .iter()
                    .map(|l| BudgetLimitDto {
                        account_id: l.account_id.0,
                        amount: l.amount.to_string(),
                    })
                    .collect(),
            };
            BudgetResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 更新预算表
async fn update_budget(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBudgetRequest>,
) -> BudgetResponse {
    let period = match parse_period_opt(req.period.as_deref()) {
        Ok(p) => p,
        Err(e) => return BudgetResponse::BadRequest(e),
    };
    let deadline = match parse_deadline(req.deadline.as_deref()) {
        Ok(d) => d,
        Err(e) => return BudgetResponse::BadRequest(e),
    };
    let limits = match parse_limits(&req.limits) {
        Ok(l) => l,
        Err(e) => return BudgetResponse::BadRequest(e),
    };

    let service = BudgetService::new(state.db.clone());
    match service
        .update_budget(
            BudgetId(id),
            &req.name,
            period,
            deadline,
            CommodityId(req.commodity_id),
            &limits,
            &lang,
        )
        .await
    {
        Ok(()) => {
            let dto = BudgetDto {
                id,
                name: req.name,
                period: period_to_string(period),
                deadline: deadline.map(|d| d.to_string()),
                commodity_id: req.commodity_id,
            };
            BudgetResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 删除预算表
async fn delete_budget(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> BudgetResponse {
    let service = BudgetService::new(state.db.clone());
    // 先确认存在，保持与详情/更新一致的 404 语义
    match service.get_budget_detail(BudgetId(id)).await {
        Ok(_) => {}
        Err(e) => return map_error(e),
    }
    match service.delete_budget(BudgetId(id)).await {
        Ok(()) => BudgetResponse::Ok(Json(serde_json::json!({"deleted": true}))),
        Err(e) => map_error(e),
    }
}

/// 查询预算执行情况
async fn get_budget_status(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Query(query): Query<BudgetStatusQuery>,
) -> BudgetResponse {
    let today = chrono::Local::now().date_naive();
    let date = match query.date {
        Some(ref d) => match chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return BudgetResponse::BadRequest(format!("无效日期: {}", e)),
        },
        None => today,
    };

    let service = BudgetService::new(state.db.clone());
    match service.get_budget_status(BudgetId(id), date).await {
        Ok(status) => {
            let names = match budget_names(&state.db, &[status.budget.id], &lang).await {
                Ok(n) => n,
                Err(r) => return r,
            };
            let dto = BudgetStatusDto {
                budget: budget_to_dto(
                    &status.budget,
                    names.get(&status.budget.id).cloned().unwrap_or_default(),
                ),
                expired: status.expired,
                period_start: status.period_start.map(|d| d.to_string()),
                period_end: status.period_end.map(|d| d.to_string()),
                items: status
                    .items
                    .iter()
                    .map(|item| BudgetItemStatusDto {
                        account_id: item.account_id.0,
                        limit_amount: item.limit_amount.to_string(),
                        actual_amount: item.actual_amount.to_string(),
                        remaining: item.remaining.to_string(),
                        percentage: item.percentage.to_string(),
                    })
                    .collect(),
            };
            BudgetResponse::Ok(Json(serde_json::to_value(dto).unwrap()))
        }
        Err(e) => map_error(e),
    }
}

/// 预算路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/budgets", get(list_budgets).post(create_budget))
        .route(
            "/api/budgets/{id}",
            get(get_budget_detail)
                .put(update_budget)
                .delete(delete_budget),
        )
        .route("/api/budgets/{id}/status", get(get_budget_status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;
    use axum::response::IntoResponse;

    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        db.account_create_with_name(
            &accounting::account::Account {
                id: AccountId(0),
                parent_id: Some(expenses_id),
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            },
            "Food",
            "en",
        )
        .await
        .unwrap();
        Arc::new(AppState { db })
    }

    async fn food_id(state: &Arc<AppState>) -> i64 {
        state
            .db()
            .account_get_by_name("Expenses:Food")
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    async fn respond(resp: BudgetResponse) -> (StatusCode, serde_json::Value) {
        let r = resp.into_response();
        let status = r.status();
        let bytes = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    /// 旧客户端 body（不含 deadline/period 键）仍可反序列化并创建一次性预算
    #[tokio::test]
    async fn create_budget_legacy_body_without_period_and_deadline() {
        let state = setup().await;
        let food = food_id(&state).await;
        let body = format!(
            r#"{{"name":"月度生活","commodity_id":1,"limits":[{{"account_id":{},"amount":"2000"}}]}}"#,
            food
        );
        let req: CreateBudgetRequest = serde_json::from_str(&body).unwrap();
        assert_eq!(req.period, None);
        assert_eq!(req.deadline, None);

        let (status, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["name"], "月度生活");
        assert_eq!(json["period"], serde_json::Value::Null);
        assert_eq!(json["deadline"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn create_budget_monthly_period_serializes_as_string() {
        let state = setup().await;
        let food = food_id(&state).await;
        let body = format!(
            r#"{{"name":"月度生活","period":"monthly","commodity_id":1,"limits":[{{"account_id":{},"amount":"2000"}}]}}"#,
            food
        );
        let req: CreateBudgetRequest = serde_json::from_str(&body).unwrap();
        let (status, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["period"], "monthly");
        assert_eq!(json["deadline"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn create_budget_one_off_with_deadline() {
        let state = setup().await;
        let food = food_id(&state).await;
        let body = format!(
            r#"{{"name":"促销预算","deadline":"2026-09-30","commodity_id":1,"limits":[{{"account_id":{},"amount":"500"}}]}}"#,
            food
        );
        let req: CreateBudgetRequest = serde_json::from_str(&body).unwrap();
        let (status, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(json["period"], serde_json::Value::Null);
        assert_eq!(json["deadline"], "2026-09-30");
    }

    #[tokio::test]
    async fn create_budget_invalid_deadline_rejected() {
        let state = setup().await;
        let food = food_id(&state).await;
        let req = CreateBudgetRequest {
            name: "坏日期".to_string(),
            period: None,
            deadline: Some("not-a-date".to_string()),
            commodity_id: 1,
            limits: vec![BudgetLimitRequest {
                account_id: food,
                amount: "100".to_string(),
            }],
        };
        let (status, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].as_str().unwrap().contains("无效日期"));
    }

    #[tokio::test]
    async fn create_budget_account_not_found_returns_400() {
        let state = setup().await;
        let req = CreateBudgetRequest {
            name: "月度生活".to_string(),
            period: Some("monthly".to_string()),
            deadline: None,
            commodity_id: 1,
            limits: vec![BudgetLimitRequest {
                account_id: 99999,
                amount: "100".to_string(),
            }],
        };
        let (status, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(json["error"].is_string());
    }

    #[tokio::test]
    async fn get_budget_detail_not_found_returns_404() {
        let state = setup().await;
        let (status, json) = respond(
            get_budget_detail(State(state.clone()), Lang("en".to_string()), Path(999)).await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("不存在"));
    }

    #[tokio::test]
    async fn delete_budget_not_found() {
        let state = setup().await;
        let (status, json) = respond(delete_budget(State(state.clone()), Path(999)).await).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(json["error"].as_str().unwrap().contains("预算表不存在"));
    }

    /// 创建一个月度预算并返回其 ID
    async fn create_monthly_budget(state: &Arc<AppState>) -> i64 {
        let food = food_id(state).await;
        let req = CreateBudgetRequest {
            name: "月度生活".to_string(),
            period: Some("monthly".to_string()),
            deadline: None,
            commodity_id: 1,
            limits: vec![BudgetLimitRequest {
                account_id: food,
                amount: "2000".to_string(),
            }],
        };
        let (_, json) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        json["id"].as_i64().unwrap()
    }

    #[tokio::test]
    async fn status_monthly_budget_period_fields_are_strings() {
        let state = setup().await;
        let id = create_monthly_budget(&state).await;

        let (status, json) = respond(
            get_budget_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(BudgetStatusQuery {
                    date: Some("2026-06-15".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["expired"], false);
        assert_eq!(json["period_start"], "2026-06-01");
        assert_eq!(json["period_end"], "2026-06-30");
        assert_eq!(json["budget"]["period"], "monthly");
    }

    #[tokio::test]
    async fn status_one_off_budget_period_fields_are_null() {
        let state = setup().await;
        let food = food_id(&state).await;
        let req = CreateBudgetRequest {
            name: "促销预算".to_string(),
            period: None,
            deadline: Some("2026-09-30".to_string()),
            commodity_id: 1,
            limits: vec![BudgetLimitRequest {
                account_id: food,
                amount: "500".to_string(),
            }],
        };
        let (_, created) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_budget_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(BudgetStatusQuery {
                    date: Some("2026-06-15".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["expired"], false);
        assert_eq!(json["period_start"], serde_json::Value::Null);
        assert_eq!(json["period_end"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn status_expired_budget_returns_200_with_expired_true() {
        let state = setup().await;
        let food = food_id(&state).await;
        let req = CreateBudgetRequest {
            name: "促销预算".to_string(),
            period: None,
            deadline: Some("2026-09-30".to_string()),
            commodity_id: 1,
            limits: vec![BudgetLimitRequest {
                account_id: food,
                amount: "500".to_string(),
            }],
        };
        let (_, created) =
            respond(create_budget(State(state.clone()), Lang("en".to_string()), Json(req)).await)
                .await;
        let id = created["id"].as_i64().unwrap();

        let (status, json) = respond(
            get_budget_status(
                State(state.clone()),
                Lang("en".to_string()),
                Path(id),
                Query(BudgetStatusQuery {
                    date: Some("2026-10-15".to_string()),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["expired"], true);
    }
}
