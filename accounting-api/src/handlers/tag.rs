//! 标签 API handler

use crate::dto::TagDto;
use crate::handlers::member::AppState;
use accounting::id::TagId;
use accounting::tag::Tag;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use rust_i18n::t;
use std::sync::Arc;

/// 获取标签列表
async fn list_tags(State(state): State<Arc<AppState>>) -> Result<Json<Vec<TagDto>>, String> {
    let db = state.db();
    let tags = db.tag_list().await.map_err(|e| e.to_string())?;
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

/// 创建标签请求
#[derive(serde::Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 创建标签
async fn create_tag(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<TagDto>, String> {
    let db = state.db();

    // 检查是否已存在
    if let Some(existing) = db
        .tag_get_by_name(&req.name)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(Json(TagDto {
            id: existing.id.0,
            name: existing.name,
            description: existing.description,
            is_system: existing.is_system,
        }));
    }

    let tag = Tag {
        id: TagId(0),
        name: req.name,
        description: req.description,
        is_system: false,
    };
    let id = db.tag_create(&tag).await.map_err(|e| e.to_string())?;

    Ok(Json(TagDto {
        id: id.0,
        name: tag.name,
        description: tag.description,
        is_system: tag.is_system,
    }))
}

/// 删除标签（只能删除非系统标签）
async fn delete_tag(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();

    // 查询标签
    let tags = db.tag_list().await.map_err(|e| e.to_string())?;
    let tag = tags
        .into_iter()
        .find(|t| t.id.0 == id)
        .ok_or(t!("tag_not_found").to_string())?;

    if tag.is_system {
        return Err(t!("cannot_delete_system_tag").to_string());
    }

    db.tag_delete(&tag.name).await.map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 标签路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tags", get(list_tags).post(create_tag))
        .route("/api/tags/{id}", delete(delete_tag))
}
