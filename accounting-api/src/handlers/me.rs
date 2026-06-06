//! 当前用户 API handler

use crate::dto::{MeDto, SetMeRequest};
use crate::handlers::member::AppState;
use accounting_sql::database::Database;
use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use std::sync::Arc;

/// 获取当前用户（返回第一个成员作为默认）
async fn get_me(State(state): State<Arc<AppState>>) -> Result<Json<MeDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let members = db
        .member_repo()
        .list(&db.connection())
        .map_err(|e| e.to_string())?;
    let first = members.into_iter().next().ok_or("没有成员")?;
    Ok(Json(MeDto {
        member_id: first.id.0,
        member_name: first.name,
    }))
}

/// 设置当前用户（暂时只返回 ok，等待 settings 表持久化）
async fn set_me(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<SetMeRequest>,
) -> Result<String, String> {
    Ok("ok".to_string())
}

/// 当前用户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/me", get(get_me).put(set_me))
}
