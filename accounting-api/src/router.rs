//! axum 路由定义

use crate::handlers;
use axum::Router;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

/// 构建所有 API 路由
pub fn create_app(
    state: Arc<handlers::member::AppState>,
    static_dir: &str,
    auth_state: accounting_auth::AuthState,
) -> Router {
    // 业务 API：全部要求有效（非 pending）session，未认证 401
    let business = Router::new()
        .merge(handlers::member::router())
        .merge(handlers::account::router())
        .merge(handlers::budget::router())
        .merge(handlers::saving_plan::router())
        .merge(handlers::transaction::router())
        .merge(handlers::channel::router())
        .merge(handlers::commodity::router())
        .merge(handlers::tag::router())
        .merge(handlers::mapping::router())
        .merge(handlers::report::router())
        .route_layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            accounting_auth::require_auth,
        ));

    Router::new()
        .merge(business)
        // 认证接口（/api/auth/*）自身管理登录态，不套业务认证
        .merge(accounting_auth::router(auth_state))
        // 健康检查与静态文件不拦截
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(
            ServeDir::new(static_dir)
                .fallback(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}
