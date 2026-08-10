//! 认证 HTTP handlers

use crate::AuthState;
use crate::api::middleware::{CurrentUser, SESSION_COOKIE, internal, unauthorized};
use crate::service::{password, recovery, session, totp};
use axum::Extension;
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use time::Duration;

/// 登录失败统一文案（防用户名枚举，见归档 `add-user-auth` design.md 安全决策）
const MSG_BAD_CREDENTIALS: &str = "用户名或密码错误";
/// TOTP 第二步失败文案
const MSG_BAD_TOTP: &str = "验证码错误";
/// 频控文案
const MSG_RATE_LIMITED: &str = "尝试过于频繁，请稍后再试";

// ─── 请求/响应 DTO ───

/// 登录请求。
#[derive(Deserialize)]
pub struct LoginRequest {
    /// 用户名。
    pub username: String,
    /// 密码。
    pub password: String,
}

/// 登录响应。
#[derive(Serialize)]
pub struct LoginResponse {
    /// 是否需要 TOTP 第二步。
    pub require_totp: bool,
    /// 需要 TOTP 时的 pending token（5 分钟有效）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_token: Option<String>,
    /// 显示名。
    pub display_name: String,
    /// 是否已开启 TOTP。
    pub totp_enabled: bool,
}

/// TOTP 第二步请求。
#[derive(Deserialize)]
pub struct LoginTotpRequest {
    /// 密码登录返回的 pending token。
    pub pending_token: String,
    /// 6 位动态码或一次性恢复码。
    pub code: String,
}

/// TOTP 绑定 setup 响应。
#[derive(Serialize)]
pub struct TotpSetupResponse {
    /// otpauth:// URI。
    pub otpauth_uri: String,
}

/// TOTP enable 请求。
#[derive(Deserialize)]
pub struct TotpEnableRequest {
    /// Authenticator 中当前有效动态码。
    pub code: String,
}

/// TOTP enable 响应：8 个恢复码明文（仅此一次返回）。
#[derive(Serialize)]
pub struct TotpEnableResponse {
    /// 一次性恢复码明文。
    pub recovery_codes: Vec<String>,
}

/// 当前用户信息响应。
#[derive(Serialize)]
pub struct MeResponse {
    /// 用户名。
    pub username: String,
    /// 显示名。
    pub display_name: String,
    /// 是否已开启 TOTP。
    pub totp_enabled: bool,
}

// ─── handlers ───

/// `POST /api/auth/login`：用户名 + 密码。
pub async fn login(
    State(state): State<AuthState>,
    client_ip: Option<Extension<ConnectInfo<SocketAddr>>>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Response {
    let key = rate_key(client_ip.as_ref(), &req.username);
    if let Some(resp) = check_rate(&state, &key) {
        return resp;
    }

    let db = state.db();
    let user = match db.find_user_by_name(&req.username).await {
        Ok(u) => u,
        Err(e) => return internal(e),
    };
    let Some(user) = user else {
        // 用户不存在：垫一次 argon2 校验保持耗时一致，返回与密码错误相同的 401
        password::dummy_verify(&req.password);
        return unauthorized(MSG_BAD_CREDENTIALS);
    };
    if !password::verify_password(&user.password_hash, &req.password) {
        return unauthorized(MSG_BAD_CREDENTIALS);
    }

    let now = chrono::Utc::now().timestamp();
    let token = session::generate_token();
    let token_hash = session::hash_token(&token);

    if user.totp_enabled {
        // 开了 TOTP：建 pending session（5 分钟），不直接发 cookie
        let expires = now + session::PENDING_TTL_SECS;
        if let Err(e) = db.create_session(&token_hash, user.id, true, expires).await {
            return internal(e);
        }
        return Json(LoginResponse {
            require_totp: true,
            pending_token: Some(token),
            display_name: user.display_name,
            totp_enabled: true,
        })
        .into_response();
    }

    let expires = now + session::SESSION_TTL_SECS;
    if let Err(e) = db
        .create_session(&token_hash, user.id, false, expires)
        .await
    {
        return internal(e);
    }
    (
        jar.add(session_cookie(&token)),
        Json(LoginResponse {
            require_totp: false,
            pending_token: None,
            display_name: user.display_name,
            totp_enabled: false,
        }),
    )
        .into_response()
}

/// `POST /api/auth/login/totp`：TOTP 第二步（动态码或恢复码）。
pub async fn login_totp(
    State(state): State<AuthState>,
    client_ip: Option<Extension<ConnectInfo<SocketAddr>>>,
    jar: CookieJar,
    Json(req): Json<LoginTotpRequest>,
) -> Response {
    // pending_token 本身就是该次登录的标识，与 IP 组成频控键
    let key = rate_key(client_ip.as_ref(), &req.pending_token);
    if let Some(resp) = check_rate(&state, &key) {
        return resp;
    }

    let db = state.db();
    let token_hash = session::hash_token(&req.pending_token);
    let sess = match db.find_session_by_token_hash(&token_hash).await {
        Ok(s) => s,
        Err(e) => return internal(e),
    };
    let now = chrono::Utc::now().timestamp();
    let Some(sess) = sess.filter(|s| s.pending && !session::is_expired(s.expires_at, now)) else {
        return unauthorized(MSG_BAD_CREDENTIALS);
    };
    let user = match db.find_user_by_id(sess.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return unauthorized(MSG_BAD_CREDENTIALS),
        Err(e) => return internal(e),
    };

    // 先尝试动态码，失败再尝试恢复码
    let ok = if let Some(secret) = user.totp_secret.as_deref() {
        match totp::verify_code(secret, &req.code, now, user.last_totp_step) {
            Some(step) => {
                if let Err(e) = db.update_last_totp_step(user.id, step as i64).await {
                    return internal(e);
                }
                true
            }
            None => {
                // 恢复码：格式上先排除 6 位纯数字（避免无谓的 JSON 解析）
                if req.code.len() == totp::CODE_DIGITS as usize
                    && req.code.chars().all(|c| c.is_ascii_digit())
                {
                    false
                } else {
                    match user
                        .recovery_codes
                        .as_deref()
                        .and_then(|json| recovery::consume_code(json, &req.code))
                    {
                        Some(new_json) => {
                            if let Err(e) = db.update_recovery_codes(user.id, &new_json).await {
                                return internal(e);
                            }
                            true
                        }
                        None => false,
                    }
                }
            }
        }
    } else {
        false
    };

    if !ok {
        // 连错 5 次作废 pending session
        match db.increment_totp_attempts(sess.id).await {
            Ok(n) if n >= 5 => {
                if let Err(e) = db.delete_session(sess.id).await {
                    return internal(e);
                }
            }
            Ok(_) => {}
            Err(e) => return internal(e),
        }
        return unauthorized(MSG_BAD_TOTP);
    }

    // 转正：pending → 正式 session，7 天滑动过期
    if let Err(e) = db
        .promote_session(sess.id, now + session::SESSION_TTL_SECS)
        .await
    {
        return internal(e);
    }
    (
        jar.add(session_cookie(&req.pending_token)),
        Json(LoginResponse {
            require_totp: false,
            pending_token: None,
            display_name: user.display_name,
            totp_enabled: true,
        }),
    )
        .into_response()
}

/// `POST /api/auth/logout`：删除服务端 session 并清 cookie。
pub async fn logout(
    State(state): State<AuthState>,
    jar: CookieJar,
    Extension(user): Extension<CurrentUser>,
) -> Response {
    let db = state.db();
    if let Ok(Some(sess)) = db
        .find_session_by_token_hash(&user.session_token_hash)
        .await
        && let Err(e) = db.delete_session(sess.id).await
    {
        return internal(e);
    }
    (jar.remove(expired_cookie()), StatusCode::NO_CONTENT).into_response()
}

/// `GET /api/auth/me`：当前登录用户信息。
pub async fn me(Extension(user): Extension<CurrentUser>) -> Response {
    Json(MeResponse {
        username: user.username,
        display_name: user.display_name,
        totp_enabled: user.totp_enabled,
    })
    .into_response()
}

/// `POST /api/auth/totp/setup`：重新生成 TOTP 密钥，返回 otpauth:// URI。
/// 注意：setup 会重置 totp_enabled 与旧恢复码，必须重新 enable 才生效。
pub async fn totp_setup(
    State(state): State<AuthState>,
    Extension(user): Extension<CurrentUser>,
) -> Response {
    let db = state.db();
    let secret = totp::generate_secret();
    if let Err(e) = db.set_totp_secret(user.id, &secret).await {
        return internal(e);
    }
    Json(TotpSetupResponse {
        otpauth_uri: totp::otpauth_uri(&secret, &user.username),
    })
    .into_response()
}

/// `POST /api/auth/totp/enable`：验证动态码后开启 TOTP，返回 8 个恢复码明文（仅此一次）。
pub async fn totp_enable(
    State(state): State<AuthState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<TotpEnableRequest>,
) -> Response {
    let db = state.db();
    let fresh = match db.find_user_by_id(user.id).await {
        Ok(Some(u)) => u,
        Ok(None) => return unauthorized("未登录或会话已过期"),
        Err(e) => return internal(e),
    };
    let Some(secret) = fresh.totp_secret.as_deref() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "请先调用 /api/auth/totp/setup" })),
        )
            .into_response();
    };
    let now = chrono::Utc::now().timestamp();
    let Some(step) = totp::verify_code(secret, &req.code, now, None) else {
        return unauthorized(MSG_BAD_TOTP);
    };

    let codes = recovery::generate_codes();
    let hashes: Vec<String> = codes.iter().map(|c| recovery::hash_code(c)).collect();
    let json = recovery::hashes_to_json(&hashes);
    if let Err(e) = db.enable_totp(user.id, &json).await {
        return internal(e);
    }
    if let Err(e) = db.update_last_totp_step(user.id, step as i64).await {
        return internal(e);
    }
    Json(TotpEnableResponse {
        recovery_codes: codes,
    })
    .into_response()
}

// ─── 工具函数 ───

/// 构造 session cookie：Secure; HttpOnly; SameSite=Lax; Path=/api；7 天。
fn session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/api")
        .max_age(session_max_age())
        .build()
}

/// 清除 cookie 用的过期 cookie。
fn expired_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/api")
        .max_age(Duration::ZERO)
        .build()
}

/// session cookie 的 Max-Age（7 天，与 DB 滑动过期一致）。
fn session_max_age() -> Duration {
    Duration::days(session::SESSION_TTL_SECS / 3600 / 24)
}

/// 频控键：IP（无 ConnectInfo 时退化为 "-"；测试环境）+ 标识符。
fn rate_key(connect_info: Option<&Extension<ConnectInfo<SocketAddr>>>, identifier: &str) -> String {
    let ip = connect_info
        .map(|c| c.0.0.ip().to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("{ip}|{identifier}")
}

/// 频控检查：放行返回 None，超限返回 429 + Retry-After 响应。
fn check_rate(state: &AuthState, key: &str) -> Option<Response> {
    match state.limiter().check(key) {
        Ok(()) => None,
        Err(retry_after) => Some(
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_after.to_string())],
                Json(serde_json::json!({ "error": MSG_RATE_LIMITED })),
            )
                .into_response(),
        ),
    }
}
