//! 货币 API handler

use crate::dto::CommodityDto;
use crate::handlers::member::AppState;
use accounting_sql::database::Database;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

/// 获取货币列表
async fn list_commodities(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CommodityDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let commodities = db
        .commodity_repo()
        .list(&db.connection())
        .map_err(|e| e.to_string())?;
    let dtos: Vec<CommodityDto> = commodities
        .iter()
        .map(|c| CommodityDto {
            id: c.id.0,
            symbol: c.symbol.clone(),
            name: c.name.clone(),
            precision: c.precision,
        })
        .collect();
    Ok(Json(dtos))
}

/// 货币路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/commodities", get(list_commodities))
}
