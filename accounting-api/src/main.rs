//! accounting-api: axum HTTP 服务入口

mod dto;
mod handlers;
mod router;

use accounting::error::AccountingError;
use axum::{Json, http::StatusCode, response::IntoResponse};
use dto::ErrorResponse;
use std::sync::Arc;

/// 将 AccountingError 转换为 HTTP 响应
pub fn account_error(err: AccountingError) -> impl IntoResponse {
    let msg = err.to_string();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: msg }),
    )
}

#[tokio::main]
async fn main() {
    let state = Arc::new(handlers::member::AppState {
        db_path: "my.db".to_string(),
    });
    let app = router::create_app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
