//! 渠道 API handler

use crate::dto::{ChannelDto, CreateChannelRequest};
use crate::handlers::member::AppState;
use accounting::channel::Channel;
use accounting::id::ChannelId;
use accounting_sql::database::Database;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use std::sync::Arc;

/// 列出所有渠道
async fn list_channels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ChannelDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let channels = db.channel_repo().list(&conn).map_err(|e| e.to_string())?;

    let dtos: Vec<ChannelDto> = channels
        .into_iter()
        .map(|c| ChannelDto {
            id: c.id.0,
            name: c.name,
            description: c.description,
        })
        .collect();

    Ok(Json(dtos))
}

/// 创建渠道
async fn create_channel(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let channel = Channel {
        id: ChannelId(0),
        name: req.name,
        description: req.description,
    };
    let id = db
        .channel_repo()
        .create(&db.connection(), &channel)
        .map_err(|e| e.to_string())?;
    Ok(Json(id.0))
}

/// 删除渠道
async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    // 检查渠道是否被交易引用
    let count: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE channel_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("渠道已被交易引用，无法删除".to_string());
    }
    db.connection()
        .execute("DELETE FROM channels WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 渠道路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/channels", get(list_channels).post(create_channel))
        .route("/api/channels/{id}", delete(delete_channel))
}
