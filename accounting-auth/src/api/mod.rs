//! HTTP API 层：axum Router 与认证中间件

pub mod handlers;
pub mod middleware;

use crate::AuthState;
use axum::Router;
use axum::routing::{get, post};

/// 构建认证子系统 Router（内部已绑定 AuthState）。
///
/// 返回泛型 `Router<S>`：标注为 `Router`（即 `Router<()>`）即可独立使用，
/// 也可直接 merge 进任意外层 state 的 Router（axum 0.8 `with_state` 惯例）。
///
/// - 公开：`POST /api/auth/login`、`POST /api/auth/login/totp`（带频控）
/// - 需登录：`logout` / `me` / `totp/setup` / `totp/enable`（套 [`middleware::require_auth`]）
pub fn router<S>(state: AuthState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let public = Router::new()
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/login/totp", post(handlers::login_totp));

    let protected = Router::new()
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/totp/setup", post(handlers::totp_setup))
        .route("/api/auth/totp/enable", post(handlers::totp_enable))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::require_auth,
        ));

    public.merge(protected).with_state(state)
}
