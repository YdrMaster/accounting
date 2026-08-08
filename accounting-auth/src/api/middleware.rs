//! 认证中间件：cookie → 哈希查 session → 非 pending 放行 → 滑动续期 → 注入 CurrentUser

use crate::AuthState;
use crate::service::session;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;

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

/// 401 响应体形状与主项目 ErrorResponse 一致：`{"error": "..."}`。
pub(crate) fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "error": message })),
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
        return unauthorized("未登录或会话已过期");
    };
    match authenticate(&state, &token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(resp) => resp,
    }
}

/// 校验 token 对应的会话；有效则滑动续期并返回用户身份。
async fn authenticate(
    state: &AuthState,
    token: &str,
) -> std::result::Result<CurrentUser, Response> {
    let token_hash = session::hash_token(token);
    let db = state.db();
    let fail = || unauthorized("未登录或会话已过期");

    let sess = db
        .find_session_by_token_hash(&token_hash)
        .await
        .map_err(internal)?
        .ok_or_else(fail)?;
    let now = chrono::Utc::now().timestamp();
    if session::is_expired(sess.expires_at, now) {
        return Err(fail());
    }
    // pending（等待 TOTP）会话不得通过业务认证
    if sess.pending {
        return Err(fail());
    }
    let user = db
        .find_user_by_id(sess.user_id)
        .await
        .map_err(internal)?
        .ok_or_else(fail)?;
    // 滑动续期
    db.touch_session(sess.id, now + session::SESSION_TTL_SECS)
        .await
        .map_err(internal)?;

    Ok(CurrentUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        totp_enabled: user.totp_enabled,
        session_token_hash: token_hash,
    })
}

/// 500：不向前端泄漏内部细节。
pub(crate) fn internal(err: impl std::fmt::Display) -> Response {
    tracing::error!("auth 内部错误: {err}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(serde_json::json!({ "error": "服务器内部错误" })),
    )
        .into_response()
}
