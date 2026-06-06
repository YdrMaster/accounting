//! 当前用户 API handler

use crate::dto::{MeDto, SetMeRequest};
use crate::handlers::member::AppState;
use accounting_sql::database::Database;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

/// 从 settings 表读取 current_member_id
async fn get_me(State(state): State<Arc<AppState>>) -> Result<Json<MeDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();

    // 尝试读取已保存的 current_member_id
    let saved_id_str: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'current_member_id'",
            [],
            |row| row.get(0),
        )
        .ok();

    let member_id = if let Some(s) = saved_id_str {
        match s.parse::<i64>() {
            Ok(id) => id,
            Err(e) => {
                eprintln!("无法解析 current_member_id '{}': {}", s, e);
                let members = db.member_repo().list(&conn).map_err(|e| e.to_string())?;
                let first = members.into_iter().next().ok_or("没有成员")?;
                first.id.0
            }
        }
    } else {
        // 未设置时返回第一个成员
        let members = db.member_repo().list(&conn).map_err(|e| e.to_string())?;
        let first = members.into_iter().next().ok_or("没有成员")?;
        first.id.0
    };

    let member = db
        .member_repo()
        .get(&conn, accounting::id::MemberId(member_id))
        .map_err(|e| e.to_string())?
        .ok_or("成员不存在")?;

    Ok(Json(MeDto {
        member_id: member.id.0,
        member_name: member.name,
    }))
}

/// 将 current_member_id 写入 settings 表
async fn set_me(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetMeRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('current_member_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [req.member_id.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 当前用户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/me", get(get_me).put(set_me))
}
