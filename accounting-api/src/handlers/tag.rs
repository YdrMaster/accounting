//! 标签 API handler

use crate::dto::TagDto;
use crate::handlers::member::AppState;
use accounting::id::TagId;
use accounting::tag::Tag;
use accounting_sql::database::Database;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
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
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();

    // 检查是否已存在
    if let Some(existing) = db
        .tag_repo()
        .get_by_name(&conn, &req.name)
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
    let id = db
        .tag_repo()
        .create(&conn, &tag)
        .map_err(|e| e.to_string())?;

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
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();

    // 查询标签
    let tags = db.tag_repo().list(&conn).map_err(|e| e.to_string())?;
    let tag = tags
        .into_iter()
        .find(|t| t.id.0 == id)
        .ok_or("标签不存在")?;

    if tag.is_system {
        return Err("不能删除系统标签".to_string());
    }

    db.tag_repo()
        .delete(&conn, &tag.name)
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 标签路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tags", get(list_tags).post(create_tag))
        .route("/api/tags/:id", delete(delete_tag))
}
