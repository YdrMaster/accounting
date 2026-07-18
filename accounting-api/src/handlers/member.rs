//! 成员 API handler

use crate::dto::MemberDto;
use crate::handlers::Lang;
use accounting::id::MemberId;
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
async fn list_members(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<Vec<MemberDto>>, String> {
    let db = state.db();
    let members = db.member_list().await.map_err(|e| e.to_string())?;
    let ids: Vec<MemberId> = members.iter().map(|m| m.id).collect();
    let names = db
        .member_display_names(&ids, &lang)
        .await
        .map_err(|e| e.to_string())?;
    let dtos: Vec<MemberDto> = members
        .iter()
        .map(|m| MemberDto {
            id: m.id.0,
            name: names.get(&m.id).cloned().unwrap_or_default(),
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
    Lang(lang): Lang,
    Json(req): Json<CreateMemberRequest>,
) -> Result<Json<MemberDto>, String> {
    let db = state.db();
    let service = accounting_service::member_service::MemberService::new(db.clone());
    let id = service
        .add(req.name.clone(), &lang)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(MemberDto {
        id: id.0,
        name: req.name,
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

/// 重命名成员（按请求语言改写该语言的显示名）
async fn rename_member(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<RenameMemberRequest>,
) -> Result<Json<MemberDto>, String> {
    let db = state.db();
    db.member_rename(MemberId(id), &req.name, &lang)
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        Arc::new(AppState { db })
    }

    #[tokio::test]
    async fn list_members_resolves_display_names_by_lang() {
        let state = setup().await;
        // 以中文创建成员，名字标注为 zh-CN
        let _ = create_member(
            State(state.clone()),
            Lang("zh-CN".to_string()),
            Json(CreateMemberRequest {
                name: "张三".to_string(),
            }),
        )
        .await
        .unwrap();

        // zh-CN 请求取到中文名
        let zh = list_members(State(state.clone()), Lang("zh-CN".to_string()))
            .await
            .unwrap()
            .0;
        assert!(zh.iter().any(|m| m.name == "张三"));
        // en 请求走回退链，仍能显示中文名
        let en = list_members(State(state), Lang("en".to_string()))
            .await
            .unwrap()
            .0;
        assert!(en.iter().any(|m| m.name == "张三"));
    }

    #[tokio::test]
    async fn rename_member_writes_name_in_request_lang() {
        let state = setup().await;
        let created = create_member(
            State(state.clone()),
            Lang("en".to_string()),
            Json(CreateMemberRequest {
                name: "Alice".to_string(),
            }),
        )
        .await
        .unwrap()
        .0;

        // 中文请求下改名 → 写 zh-CN 显示名，en 显示名不受影响
        let _ = rename_member(
            State(state.clone()),
            Lang("zh-CN".to_string()),
            Path(created.id),
            Json(RenameMemberRequest {
                name: "爱丽丝".to_string(),
            }),
        )
        .await
        .unwrap();

        let zh = list_members(State(state.clone()), Lang("zh-CN".to_string()))
            .await
            .unwrap()
            .0;
        assert!(zh.iter().any(|m| m.id == created.id && m.name == "爱丽丝"));
        let en = list_members(State(state), Lang("en".to_string()))
            .await
            .unwrap()
            .0;
        assert!(en.iter().any(|m| m.id == created.id && m.name == "Alice"));
    }
}
