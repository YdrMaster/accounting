//! accounting-api: axum HTTP 服务入口

use accounting::error::AccountingError;
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};

mod dto;
use dto::ErrorResponse;

/// 将 AccountingError 转换为 HTTP 响应。
pub fn account_error(err: AccountingError) -> impl IntoResponse {
    let msg = err.to_string();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: msg }),
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/health", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
