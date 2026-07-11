//! 渠道 API handler

use crate::dto::{ChannelDto, CreateChannelRequest, UpdateChannelRequest};
use crate::handlers::member::AppState;
use accounting::channel::Channel;
use accounting::id::{AccountId, ChannelId};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use rust_i18n::t;
use std::sync::Arc;

/// 列出所有渠道
async fn list_channels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ChannelDto>>, String> {
    let db = state.db();
    let channels = db.channel_list().await.map_err(|e| e.to_string())?;

    let dtos: Vec<ChannelDto> = channels
        .into_iter()
        .map(|c| ChannelDto {
            id: c.id.0,
            name: c.name,
            description: c.description,
            account_id: c.account_id.map(|id| id.0),
            is_system: c.is_system,
        })
        .collect();

    Ok(Json(dtos))
}

/// 创建渠道
async fn create_channel(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();
    let channel = Channel {
        id: ChannelId(0),
        name: req.name,
        description: req.description,
        account_id: req.account_id.map(AccountId),
        is_system: false,
    };
    let id = db
        .channel_create(&channel)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(id.0))
}

/// 更新渠道（修改 name、description、account_id）
async fn update_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<String, String> {
    let db = state.db();
    let existing = db
        .channel_get(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("channel_not_found").to_string())?;

    let new_name = match req.name {
        Some(Some(ref v)) => v.as_str(),
        _ => existing.name.as_str(),
    };
    let new_description = match req.description {
        Some(ref v) => v.as_deref(),
        None => existing.description.as_deref(),
    };
    let new_account_id = match req.account_id {
        Some(v) => v.map(AccountId),
        None => existing.account_id,
    };

    db.channel_update(ChannelId(id), new_name, new_description, new_account_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 删除渠道
async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    // 检查渠道是否被 channel_paths 引用
    let count = db
        .channel_count_transactions_by_id(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(t!("channel_in_use").to_string());
    }
    db.channel_force_delete_by_id(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 渠道路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/channels", get(list_channels).post(create_channel))
        .route(
            "/api/channels/{id}",
            put(update_channel).delete(delete_channel),
        )
}
