//! accounting-api: axum HTTP 服务入口

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/health", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
