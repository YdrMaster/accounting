//! 端到端测试：tower oneshot 直接打 Router，覆盖 spec 全部场景

use accounting_auth::service::rate_limit::{DEFAULT_WINDOW, MemoryRateLimiter};
use accounting_auth::service::{password, totp};
use accounting_auth::{AuthState, require_auth};
use axum::Router;
use axum::body::Body;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

/// 构造测试 app：auth router + 一个挂了认证中间件的伪业务路由
async fn test_app(state: AuthState) -> Router {
    let protected = Router::new()
        .route("/api/biz/ping", axum::routing::get(|| async { "pong" }))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));
    Router::new()
        .merge(accounting_auth::router(state))
        .merge(protected)
}

/// 默认频控的 state
async fn default_state() -> AuthState {
    accounting_auth::init(":memory:").await.unwrap()
}

/// 放宽频控的 state（测试"连错 5 次作废 pending"等不受 5 次/分干扰）
async fn relaxed_state() -> AuthState {
    let state = default_state().await;
    AuthState::with_limiter(
        state.db().clone(),
        Arc::new(MemoryRateLimiter::new(1000, DEFAULT_WINDOW)),
    )
}

/// 直接在 DB 建用户
async fn create_user(state: &AuthState, username: &str, pwd: &str) {
    let hash = password::hash_password(pwd).unwrap();
    state
        .db()
        .create_user(username, &hash, &format!("{username}显示名"))
        .await
        .unwrap();
}

async fn post_json(
    app: &Router,
    uri: &str,
    body: serde_json::Value,
    cookie: Option<&str>,
) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    let mut builder = Request::post(uri).header("content-type", "application/json");
    if let Some(c) = cookie {
        builder = builder.header(COOKIE, c);
    }
    let req = builder.body(Body::from(body.to_string())).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, json)
}

async fn get(app: &Router, uri: &str, cookie: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::get(uri);
    if let Some(c) = cookie {
        builder = builder.header(COOKIE, c);
    }
    let req = builder.body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

/// 生成当前窗口内且未被防重放拒绝（> last_step）的动态码对应的时间步
fn fresh_totp_code(secret: &[u8], last_step: Option<i64>) -> u32 {
    let now = chrono::Utc::now().timestamp();
    let mut step = totp::current_step(now);
    if let Some(last) = last_step
        && step <= last as u64
    {
        step = last as u64 + 1;
    }
    totp::totp_at_step(secret, step)
}

/// 从 Set-Cookie 头提取 session cookie（`session=<token>`，供 Cookie 请求头用）
fn session_cookie_from(headers: &axum::http::HeaderMap) -> String {
    let set_cookie = headers
        .get(SET_COOKIE)
        .expect("应有 Set-Cookie")
        .to_str()
        .unwrap();
    assert!(
        set_cookie.contains("HttpOnly"),
        "cookie 应 HttpOnly: {set_cookie}"
    );
    assert!(
        set_cookie.contains("Secure"),
        "cookie 应 Secure: {set_cookie}"
    );
    assert!(
        set_cookie.contains("SameSite=Lax"),
        "cookie 应 SameSite=Lax: {set_cookie}"
    );
    assert!(
        set_cookie.contains("Path=/api"),
        "cookie 应 Path=/api: {set_cookie}"
    );
    set_cookie.split(';').next().unwrap().to_string()
}

async fn login(
    app: &Router,
    username: &str,
    pwd: &str,
) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    post_json(
        app,
        "/api/auth/login",
        serde_json::json!({"username": username, "password": pwd}),
        None,
    )
    .await
}

// ─── 用户名密码登录 ───

#[tokio::test]
async fn login_success_without_totp() {
    let state = default_state().await;
    create_user(&state, "alice", "pw123").await;
    let app = test_app(state).await;

    let (status, headers, body) = login(&app, "alice", "pw123").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["require_totp"], false);
    assert_eq!(body["totp_enabled"], false);
    assert_eq!(body["display_name"], "alice显示名");
    let cookie = session_cookie_from(&headers);
    assert!(cookie.starts_with("session="));

    // cookie 可用于业务接口与 /me
    let (status, text) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!((status, text.as_str()), (StatusCode::OK, "pong"));
    let (status, body) = get(&app, "/api/auth/me", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(body["username"], "alice");
    assert_eq!(body["totp_enabled"], false);
}

#[tokio::test]
async fn login_failures_are_indistinguishable() {
    let state = default_state().await;
    create_user(&state, "bob", "right-pw").await;
    let app = test_app(state).await;

    // 密码错误
    let (s1, _, b1) = login(&app, "bob", "wrong-pw").await;
    // 用户不存在
    let (s2, _, b2) = login(&app, "nobody", "wrong-pw").await;
    assert_eq!(s1, StatusCode::UNAUTHORIZED);
    assert_eq!(s2, StatusCode::UNAUTHORIZED);
    assert_eq!(b1, serde_json::json!({"error": "用户名或密码错误"}));
    assert_eq!(b1, b2, "用户不存在与密码错误的响应必须完全一致");
}

// ─── TOTP 双因素 ───

/// 走完 setup + enable，返回 (cookie, secret, 恢复码)
async fn setup_and_enable_totp(
    app: &Router,
    state: &AuthState,
    username: &str,
) -> (String, Vec<u8>, Vec<String>) {
    let (status, headers, _) = login(app, username, "pw123").await;
    assert_eq!(status, StatusCode::OK);
    let cookie = session_cookie_from(&headers);

    let (status, _, body) = post_json(
        app,
        "/api/auth/totp/setup",
        serde_json::json!({}),
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let uri = body["otpauth_uri"].as_str().unwrap();
    assert!(uri.starts_with("otpauth://totp/Accounting:"), "uri: {uri}");

    let user = state
        .db()
        .find_user_by_name(username)
        .await
        .unwrap()
        .unwrap();
    let secret = user.totp_secret.clone().unwrap();
    let now = chrono::Utc::now().timestamp();
    let code = format!(
        "{:06}",
        totp::totp_at_step(&secret, totp::current_step(now))
    );

    let (status, _, body) = post_json(
        app,
        "/api/auth/totp/enable",
        serde_json::json!({"code": code}),
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let codes: Vec<String> = serde_json::from_value(body["recovery_codes"].clone()).unwrap();
    assert_eq!(codes.len(), 8);
    (cookie, secret, codes)
}

#[tokio::test]
async fn totp_setup_enable_flow() {
    let state = default_state().await;
    create_user(&state, "carol", "pw123").await;
    let app = test_app(state.clone()).await;

    setup_and_enable_totp(&app, &state, "carol").await;
    let user = state
        .db()
        .find_user_by_name("carol")
        .await
        .unwrap()
        .unwrap();
    assert!(user.totp_enabled);
    assert!(user.recovery_codes.is_some());
}

#[tokio::test]
async fn two_step_login_success() {
    let state = default_state().await;
    create_user(&state, "dave", "pw123").await;
    let app = test_app(state.clone()).await;
    let (_, secret, _) = setup_and_enable_totp(&app, &state, "dave").await;

    // 第一步：密码 → require_totp + pending_token，且无 Set-Cookie
    let (status, headers, body) = login(&app, "dave", "pw123").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["require_totp"], true);
    assert!(
        headers.get(SET_COOKIE).is_none(),
        "pending 阶段不应发 cookie"
    );
    let pending_token = body["pending_token"].as_str().unwrap().to_string();

    // 第二步：动态码 → 200 + cookie
    // （enable 刚用过当前时间步的码，防重放会拒绝同一步，故取 last_totp_step 之后的步）
    let user = state.db().find_user_by_name("dave").await.unwrap().unwrap();
    let code = format!("{:06}", fresh_totp_code(&secret, user.last_totp_step));
    let (status, headers, body) = post_json(
        &app,
        "/api/auth/login/totp",
        serde_json::json!({"pending_token": pending_token, "code": code}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["require_totp"], false);
    let cookie = session_cookie_from(&headers);

    let (status, text) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!((status, text.as_str()), (StatusCode::OK, "pong"));
}

#[tokio::test]
async fn wrong_totp_code_and_pending_invalidated_after_5_failures() {
    let state = relaxed_state().await; // 放宽频控，专注"连错 5 次作废"
    create_user(&state, "erin", "pw123").await;
    let app = test_app(state.clone()).await;
    let (_, secret, _) = setup_and_enable_totp(&app, &state, "erin").await;

    let (_, _, body) = login(&app, "erin", "pw123").await;
    let pending_token = body["pending_token"].as_str().unwrap().to_string();

    // 构造一个保证错误的码：当前码 + 1（mod 1e6，且避开 ±1 窗口与恢复码格式）
    let now = chrono::Utc::now().timestamp();
    let current = totp::totp_at_step(&secret, totp::current_step(now));
    let mut bad = (current + 1) % 1_000_000;
    for s in [
        totp::current_step(now) - 1,
        totp::current_step(now),
        totp::current_step(now) + 1,
    ] {
        if bad == totp::totp_at_step(&secret, s) {
            bad = (bad + 1) % 1_000_000;
        }
    }
    let bad_code = format!("{bad:06}");

    // 连错 5 次
    for i in 0..5 {
        let (status, _, _) = post_json(
            &app,
            "/api/auth/login/totp",
            serde_json::json!({"pending_token": pending_token, "code": bad_code}),
            None,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "第 {} 次错误码应 401",
            i + 1
        );
    }

    // pending 已作废：即使提交正确码也 401
    let now = chrono::Utc::now().timestamp();
    let code = format!(
        "{:06}",
        totp::totp_at_step(&secret, totp::current_step(now))
    );
    let (status, _, _) = post_json(
        &app,
        "/api/auth/login/totp",
        serde_json::json!({"pending_token": pending_token, "code": code}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn recovery_code_is_one_time() {
    let state = relaxed_state().await;
    create_user(&state, "frank", "pw123").await;
    let app = test_app(state.clone()).await;
    let (_, _, codes) = setup_and_enable_totp(&app, &state, "frank").await;

    // 用恢复码完成两步登录
    let (_, _, body) = login(&app, "frank", "pw123").await;
    let pending_token = body["pending_token"].as_str().unwrap().to_string();
    let (status, headers, _) = post_json(
        &app,
        "/api/auth/login/totp",
        serde_json::json!({"pending_token": pending_token, "code": codes[0]}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    session_cookie_from(&headers);

    // 再次提交同一恢复码 → 401
    let (_, _, body) = login(&app, "frank", "pw123").await;
    let pending_token2 = body["pending_token"].as_str().unwrap().to_string();
    let (status, _, _) = post_json(
        &app,
        "/api/auth/login/totp",
        serde_json::json!({"pending_token": pending_token2, "code": codes[0]}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 其余恢复码仍可用
    let (_, _, body) = login(&app, "frank", "pw123").await;
    let pending_token3 = body["pending_token"].as_str().unwrap().to_string();
    let (status, _, _) = post_json(
        &app,
        "/api/auth/login/totp",
        serde_json::json!({"pending_token": pending_token3, "code": codes[1]}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

// ─── Session 管理 / 业务保护 ───

#[tokio::test]
async fn unauthenticated_business_api_returns_401() {
    let state = default_state().await;
    let app = test_app(state).await;

    let (status, _) = get(&app, "/api/biz/ping", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // 伪造 cookie 也 401
    let (status, _) = get(&app, "/api/biz/ping", Some("session=deadbeef")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn pending_session_cannot_access_business_api() {
    let state = default_state().await;
    create_user(&state, "grace", "pw123").await;
    let app = test_app(state.clone()).await;
    setup_and_enable_totp(&app, &state, "grace").await;

    let (_, _, body) = login(&app, "grace", "pw123").await;
    let pending_token = body["pending_token"].as_str().unwrap().to_string();

    // pending token 作为 cookie 访问业务接口 → 401
    let cookie = format!("session={pending_token}");
    let (status, _) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // /me 同样拒绝
    let (status, _) = get(&app, "/api/auth/me", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_session_access_renews_expiry() {
    let state = default_state().await;
    create_user(&state, "heidi", "pw123").await;
    let app = test_app(state.clone()).await;

    // 手工建一个 1 小时后过期的正式 session
    let token = accounting_auth::service::session::generate_token();
    let hash = accounting_auth::service::session::hash_token(&token);
    let now = chrono::Utc::now().timestamp();
    let user = state
        .db()
        .find_user_by_name("heidi")
        .await
        .unwrap()
        .unwrap();
    state
        .db()
        .create_session(&hash, user.id, false, now + 3600)
        .await
        .unwrap();

    let cookie = format!("session={token}");
    let (status, _) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);

    // 滑动续期：过期时间被推迟到约 7 天后
    let sess = state
        .db()
        .find_session_by_token_hash(&hash)
        .await
        .unwrap()
        .unwrap();
    let expected = now + accounting_auth::service::session::SESSION_TTL_SECS;
    assert!(
        (sess.expires_at - expected).abs() < 60,
        "续期后 expires_at={} 应≈{expected}",
        sess.expires_at
    );
}

#[tokio::test]
async fn logout_invalidates_session_immediately() {
    let state = default_state().await;
    create_user(&state, "ivan", "pw123").await;
    let app = test_app(state).await;

    let (status, headers, _) = login(&app, "ivan", "pw123").await;
    assert_eq!(status, StatusCode::OK);
    let cookie = session_cookie_from(&headers);

    // 登出：204 + 清 cookie
    let (status, headers, _) = post_json(
        &app,
        "/api/auth/logout",
        serde_json::json!({}),
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let cleared = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cleared.contains("session="), "应下发清除 cookie: {cleared}");

    // 同一 cookie 再访问 → 401（服务端记录已删，不依赖清 cookie）
    let (status, _) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_session_returns_401() {
    let state = default_state().await;
    create_user(&state, "judy", "pw123").await;
    let app = test_app(state.clone()).await;

    let token = accounting_auth::service::session::generate_token();
    let hash = accounting_auth::service::session::hash_token(&token);
    let now = chrono::Utc::now().timestamp();
    let user = state.db().find_user_by_name("judy").await.unwrap().unwrap();
    // 直接插入已过期的 session（模拟 7 天未活动）
    state
        .db()
        .create_session(&hash, user.id, false, now - 1)
        .await
        .unwrap();

    let cookie = format!("session={token}");
    let (status, _) = get(&app, "/api/biz/ping", Some(&cookie)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ─── 频控 ───

#[tokio::test]
async fn rate_limit_triggers_429_with_retry_after() {
    let state = default_state().await;
    create_user(&state, "victim", "pw123").await;
    let app = test_app(state).await;

    // 同 IP（无 ConnectInfo → "-"）+ 同用户名：前 5 次放行（401），第 6 次 429
    for i in 0..5 {
        let (status, _, _) = login(&app, "victim", "bad-pw").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "第 {} 次", i + 1);
    }
    let (status, headers, body) = login(&app, "victim", "bad-pw").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(
        headers.get("retry-after").is_some(),
        "429 必须带 Retry-After"
    );
    assert_eq!(body["error"], "尝试过于频繁，请稍后再试");

    // 换用户名不受该键的频控影响
    let (status, _, _) = login(&app, "someone-else", "bad-pw").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
