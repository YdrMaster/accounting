//! 成员 API handler

use crate::dto::MemberDto;
use accounting::error::AccountingError;
use accounting::id::MemberId;
use accounting::member::Member;
use accounting_sql::database::Database;
use accounting_sql::impls::sqlite::SqliteDatabase;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use std::sync::Arc;

/// API 共享状态：数据库路径
#[derive(Clone)]
pub struct AppState {
    pub db_path: String,
}

impl AppState {
    /// 根据 db_path 打开数据库
    ///
    /// 启动时已预热，保证数据库始终包含完整种子数据，此处只做打开。
    pub fn db(&self) -> Result<SqliteDatabase, AccountingError> {
        SqliteDatabase::open(&self.db_path)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }
}

/// 获取成员列表
async fn list_members(State(state): State<Arc<AppState>>) -> Result<Json<Vec<MemberDto>>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let members = db
        .member_repo()
        .list(&db.connection())
        .map_err(|e| e.to_string())?;
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
    let db = state.db().map_err(|e| e.to_string())?;
    let member = Member {
        id: MemberId(0),
        name: req.name,
    };
    let id = db
        .member_repo()
        .create(&db.connection(), &member)
        .map_err(|e| e.to_string())?;
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
    let db = state.db().map_err(|e| e.to_string())?;
    db.member_repo()
        .delete(&db.connection(), MemberId(id))
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 成员路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/members", get(list_members).post(create_member))
        .route("/api/members/{id}", delete(delete_member))
}
