//! 货币 API handler

use crate::dto::CommodityDto;
use crate::handlers::{Lang, member::AppState};
use accounting::id::CommodityId;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

/// 获取货币列表
async fn list_commodities(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<Vec<CommodityDto>>, String> {
    let db = state.db();
    let commodities = db.commodity_list().await.map_err(|e| e.to_string())?;
    let ids: Vec<CommodityId> = commodities.iter().map(|c| c.id).collect();
    let names = db
        .commodity_display_names(&ids, &lang)
        .await
        .map_err(|e| e.to_string())?;
    let dtos: Vec<CommodityDto> = commodities
        .iter()
        .map(|c| CommodityDto {
            id: c.id.0,
            symbol: c.symbol.clone(),
            name: names.get(&c.id).cloned().unwrap_or_default(),
            precision: c.precision,
        })
        .collect();
    Ok(Json(dtos))
}

/// 货币路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/commodities", get(list_commodities))
}
