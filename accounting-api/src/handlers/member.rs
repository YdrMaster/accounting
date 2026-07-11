//! 成员 API handler

use crate::dto::MemberDto;
use accounting::id::MemberId;
use accounting::member::Member;
use accounting_sql::SqliteDatabase;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use std::sync::Arc;

/// API 共享状态：数据库连接池
#[derive(Clone)]
pub struct AppState {
    pub db: SqliteDatabase,
}

impl AppState {
    /// 返回共享的数据库实例
    pub fn db(&self) -> &SqliteDatabase {
        &self.db
    }
}

/// 获取成员列表
async fn list_members(State(state): State<Arc<AppState>>) -> Result<Json<Vec<MemberDto>>, String> {
    let db = state.db();
    let members = db.member_list().await.map_err(|e| e.to_string())?;
    let dtos: Vec<MemberDto> = members
        .iter()
        .map(|m| MemberDto {
            id: m.id.0,
            name: m.name.clone(),
        })
        .collect();
    Ok(Json(dtos))
}

/// 创建成员请求体
#[derive(serde::Deserialize)]
pub struct CreateMemberRequest {
    pub name: String,
}

/// 创建成员
async fn create_member(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMemberRequest>,
) -> Result<Json<MemberDto>, String> {
    let db = state.db();
    let member = Member {
        id: MemberId(0),
        name: req.name,
    };
    let id = db.member_create(&member).await.map_err(|e| e.to_string())?;
    Ok(Json(MemberDto {
        id: id.0,
        name: member.name,
    }))
}

/// 删除成员
async fn delete_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    db.member_delete(MemberId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 重命名成员请求体
#[derive(serde::Deserialize)]
pub struct RenameMemberRequest {
    pub name: String,
}

/// 重命名成员
async fn rename_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<RenameMemberRequest>,
) -> Result<Json<MemberDto>, String> {
    let db = state.db();
    db.member_rename(MemberId(id), &req.name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(MemberDto { id, name: req.name }))
}

/// 成员路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/members", get(list_members).post(create_member))
        .route(
            "/api/members/{id}",
            delete(delete_member).put(rename_member),
        )
}
