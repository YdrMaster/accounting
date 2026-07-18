//! 标签 API handler

use crate::dto::TagDto;
use crate::handlers::{Lang, member::AppState};
use accounting::id::TagId;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use rust_i18n::t;
use std::sync::Arc;

/// 获取标签列表
async fn list_tags(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<Vec<TagDto>>, String> {
    let db = state.db();
    let tags = db.tag_list().await.map_err(|e| e.to_string())?;
    let ids: Vec<TagId> = tags.iter().map(|t| t.id).collect();
    let names = db
        .tag_display_names(&ids, &lang)
        .await
        .map_err(|e| e.to_string())?;
    let dtos: Vec<TagDto> = tags
        .iter()
        .map(|t| TagDto {
            id: t.id.0,
            name: names.get(&t.id).cloned().unwrap_or_default(),
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
    Lang(lang): Lang,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<TagDto>, String> {
    let db = state.db();

    // 命中任意语言的名字即返回既有标签
    if let Some(existing) = db
        .tag_get_by_name(&req.name)
        .await
        .map_err(|e| e.to_string())?
    {
        let names = db
            .tag_display_names(&[existing.id], &lang)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(Json(TagDto {
            id: existing.id.0,
            name: names.get(&existing.id).cloned().unwrap_or_default(),
            description: existing.description,
            is_system: existing.is_system,
        }));
    }

    let service = accounting_service::tag_service::TagService::new(db.clone());
    let id = service
        .add(req.name.clone(), req.description.clone(), &lang)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(TagDto {
        id: id.0,
        name: req.name,
        description: req.description,
        is_system: false,
    }))
}

/// 更新标签请求
#[derive(serde::Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

/// 更新标签（改名 = 改写请求语言下的显示名）
async fn update_tag(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<TagDto>, String> {
    let db = state.db();

    let existing = db
        .tag_get_by_id(TagId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("tag_not_found", locale = lang.as_str()).to_string())?;

    if existing.is_system {
        return Err(t!("cannot_modify_system_tag", locale = lang.as_str()).to_string());
    }

    let names = db
        .tag_display_names(&[existing.id], &lang)
        .await
        .map_err(|e| e.to_string())?;
    let current_name = names.get(&existing.id).cloned().unwrap_or_default();

    let new_name = match req.name {
        Some(Some(ref v)) => v.clone(),
        _ => current_name,
    };
    let new_description = match req.description {
        Some(ref v) => v.as_deref(),
        None => existing.description.as_deref(),
    };

    db.tag_update(TagId(id), &new_name, new_description, &lang)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(TagDto {
        id: existing.id.0,
        name: new_name,
        description: new_description.map(|s| s.to_string()),
        is_system: existing.is_system,
    }))
}

/// 删除标签（只能删除非系统标签）
async fn delete_tag(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();

    let tag = db
        .tag_get_by_id(TagId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("tag_not_found", locale = lang.as_str()).to_string())?;

    if tag.is_system {
        return Err(t!("cannot_delete_system_tag", locale = lang.as_str()).to_string());
    }

    let names = db
        .tag_display_names(&[tag.id], &lang)
        .await
        .map_err(|e| e.to_string())?;
    let name = names
        .get(&tag.id)
        .cloned()
        .ok_or_else(|| t!("tag_not_found", locale = lang.as_str()).to_string())?;

    db.tag_delete(&name).await.map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 标签路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tags", get(list_tags).post(create_tag))
        .route("/api/tags/{id}", put(update_tag).delete(delete_tag))
}
