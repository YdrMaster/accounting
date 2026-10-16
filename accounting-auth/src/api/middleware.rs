//! 认证中间件：cookie → 哈希查 session → 非 pending 放行 → 滑动续期 → 注入 CurrentUser

use crate::AuthState;
use crate::service::session;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use rust_i18n::t;

/// session cookie 名
pub const SESSION_COOKIE: &str = "session";

/// 认证通过后注入 request extension 的当前用户身份。
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// 用户 ID。
    pub id: i64,
    /// 用户名。
    pub username: String,
    /// 显示名。
    pub display_name: String,
    /// 是否已开启 TOTP。
    pub totp_enabled: bool,
    /// 当前会话的 token 哈希（登出时删除服务端记录用）。
    pub session_token_hash: String,
}

/// 401 响应体：`{"error": "<本地化文案>", "code": "<稳定码>"}`。
/// 客户端按 `code` 分支，不解析本地化文案。文案由进程 locale 经 `t!` 渲染。
pub(crate) fn unauthorized(code: &str) -> Response {
    let message = match code {
        "bad_credentials" => t!("bad_credentials").to_string(),
        "bad_totp" => t!("bad_totp").to_string(),
        "unauthenticated" => t!("unauthenticated").to_string(),
        other => other.to_string(),
    };
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "error": message, "code": code })),
    )
        .into_response()
}

/// 认证中间件（供 `middleware::from_fn_with_state` 使用）。
///
/// 读 session cookie → SHA-256 哈希查 session → 过期/pending 拒绝 →
/// 滑动续期 7 天 → 注入 [`CurrentUser`] extension 放行。
pub async fn require_auth(
    State(state): State<AuthState>,
    jar: CookieJar,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let Some(token) = jar.get(SESSION_COOKIE).map(|c| c.value().to_string()) else {
        return unauthorized("unauthenticated");
    };
    match authenticate(&state, &token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(resp) => *resp,
    }
}

/// 校验 token 对应的会话；有效则滑动续期并返回用户身份。
///
/// `Err` 装箱避免 `result_large_err`：`Response` 体积超阈值，此处仅经
/// `require_auth` 解引用转发，不需要内联布局。
async fn authenticate(
    state: &AuthState,
    token: &str,
) -> std::result::Result<CurrentUser, Box<Response>> {
    let token_hash = session::hash_token(token);
    let db = state.db();
    let fail = || Box::new(unauthorized("unauthenticated"));

    let row = db
        .find_session_with_user_by_token_hash(&token_hash)
        .await
        .map_err(|e| Box::new(internal(e)))?
        .ok_or_else(fail)?;
    let now = chrono::Utc::now().timestamp();
    if session::is_expired(row.expires_at, now) {
        return Err(fail());
    }
    // pending（等待 TOTP）会话不得通过业务认证
    if row.pending {
        return Err(fail());
    }
    // 滑动续期节流：剩余不足半程才续期，写放大从每请求 1 次降为约每半程 1 次
    if row.expires_at - now < session::SESSION_TTL_SECS / 2 {
        db.touch_session(row.session_id, now + session::SESSION_TTL_SECS)
            .await
            .map_err(|e| Box::new(internal(e)))?;
    }

    Ok(CurrentUser {
        id: row.user_id,
        username: row.username,
        display_name: row.display_name,
        totp_enabled: row.totp_enabled,
        session_token_hash: token_hash,
    })
}

/// 500：不向前端泄漏内部细节。
pub(crate) fn internal(err: impl std::fmt::Display) -> Response {
    tracing::error!("auth 内部错误: {err}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(serde_json::json!({ "error": t!("internal_error").to_string() })),
    )
        .into_response()
}
