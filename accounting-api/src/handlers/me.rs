//! 当前用户 API handler

use crate::dto::{MeDto, SetMeRequest};
use crate::handlers::{Lang, member::AppState};
use accounting::id::MemberId;
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

/// 从 settings 表读取 current_member_id
async fn get_me(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<MeDto>, String> {
    let db = state.db();

    // 尝试读取已保存的 current_member_id
    let saved_id_str: Option<String> = db
        .get_setting("current_member_id")
        .await
        .map_err(|e| e.to_string())?;

    let member_id = if let Some(s) = saved_id_str {
        match s.parse::<i64>() {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "{}",
                    rust_i18n::t!(
                        "parse_member_id_failed",
                        locale = lang.as_str(),
                        id = s,
                        error = e
                    )
                );
                first_member_id(db, &lang).await?
            }
        }
    } else {
        // 未设置时返回第一个成员
        first_member_id(db, &lang).await?
    };

    let member = db
        .member_get(MemberId(member_id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or(rust_i18n::t!("member_not_found", locale = lang.as_str()).to_string())?;

    let names = db
        .member_display_names(&[member.id], &lang)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(MeDto {
        member_id: member.id.0,
        member_name: names.get(&member.id).cloned().unwrap_or_default(),
    }))
}

/// 返回第一个成员的 ID
async fn first_member_id(db: &accounting_sql::SqliteDatabase, lang: &str) -> Result<i64, String> {
    let members = db.member_list().await.map_err(|e| e.to_string())?;
    let first = members
        .into_iter()
        .next()
        .ok_or(rust_i18n::t!("no_members", locale = lang).to_string())?;
    Ok(first.id.0)
}

/// 将 current_member_id 写入 settings 表
async fn set_me(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetMeRequest>,
) -> Result<String, String> {
    let db = state.db();
    let id = req.member_id.to_string();
    db.set_setting("current_member_id", &id)
        .await
        .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 当前用户路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/me", get(get_me).put(set_me))
}
