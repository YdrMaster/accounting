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

use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "accounting-api")]
struct Args {
    /// 数据库文件路径
    #[arg(long, default_value = "my.db")]
    db: String,
    /// 监听端口
    #[arg(long, default_value = "3000")]
    port: u16,
    /// 前端静态文件目录
    #[arg(long, default_value = "accounting-web/dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state = Arc::new(handlers::member::AppState { db_path: args.db });
    let app = router::create_app(state, &args.static_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
