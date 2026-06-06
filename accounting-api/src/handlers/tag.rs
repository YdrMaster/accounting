//! 标签 API handler

use crate::dto::TagDto;
use crate::handlers::member::AppState;
use accounting_sql::database::Database;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

/// 获取标签列表
async fn list_tags(State(state): State<Arc<AppState>>) -> Result<Json<Vec<TagDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let tags = db
        .tag_repo()
        .list(&db.connection())
        .map_err(|e| e.to_string())?;
    let dtos: Vec<TagDto> = tags
        .iter()
        .map(|t| TagDto {
            id: t.id.0,
            name: t.name.clone(),
            description: t.description.clone(),
            is_system: t.is_system,
        })
        .collect();
    Ok(Json(dtos))
}

/// 标签路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/tags", get(list_tags))
}
