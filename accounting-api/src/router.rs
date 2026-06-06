//! axum 路由定义

use crate::handlers;
use axum::Router;
use std::sync::Arc;

/// 构建所有 API 路由
pub fn create_app(state: Arc<handlers::member::AppState>) -> Router {
    Router::new()
        .merge(handlers::member::router())
        .merge(handlers::account::router())
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .with_state(state)
}
