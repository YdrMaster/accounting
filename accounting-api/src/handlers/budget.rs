//! 预算 API handler

use crate::dto::{
    BudgetDetailDto, BudgetDto, BudgetItemStatusDto, BudgetLimitDto, BudgetLimitRequest,
    BudgetStatusDto, CreateBudgetRequest, UpdateBudgetRequest, parse_period, to_period_string,
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
    if msg.contains("不存在") {
        BudgetResponse::NotFound(msg)
    } else {
        BudgetResponse::BadRequest(msg)
    }
}

fn budget_to_dto(b: &accounting::budget::Budget, name: String) -> BudgetDto {
    BudgetDto {
        id: b.id.0,
        name,
        period: to_period_string(b.period).to_string(),
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
    let period = match parse_period(&req.period) {
        Ok(p) => p,
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
                period: req.period,
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
    let period = match parse_period(&req.period) {
        Ok(p) => p,
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
                period: req.period,
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
                period_start: status.period_start.to_string(),
                period_end: status.period_end.to_string(),
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
