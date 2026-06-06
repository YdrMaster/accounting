//! axum 路由定义

use crate::handlers;
use axum::Router;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

/// 构建所有 API 路由
pub fn create_app(state: Arc<handlers::member::AppState>, static_dir: &str) -> Router {
    Router::new()
        .merge(handlers::member::router())
        .merge(handlers::account::router())
        .merge(handlers::transaction::router())
        .merge(handlers::tag::router())
        .merge(handlers::report::router())
        .merge(handlers::me::router())
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(
            ServeDir::new(static_dir)
                .fallback(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}
