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

    // 自动编译前端：检测 accounting-web 目录是否存在，dist 是否过期
    let dist_path = std::path::Path::new(&args.static_dir);
    let web_dir = std::path::Path::new("accounting-web");
    if web_dir.exists() && web_dir.join("package.json").exists() {
        let needs_build = if !dist_path.exists() {
            true
        } else if let (Ok(d), Ok(p)) = (
            std::fs::metadata(dist_path),
            std::fs::metadata(web_dir.join("package.json")),
        ) {
            let dist_time = d.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let pkg_time = p.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if pkg_time > dist_time {
                true
            } else {
                // 递归检查 src 目录下所有源文件
                let src_dir = web_dir.join("src");
                let mut newest_src = std::time::SystemTime::UNIX_EPOCH;
                fn walk_dir(dir: &std::path::Path, newest: &mut std::time::SystemTime) {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                walk_dir(&path, newest);
                            } else if let Ok(m) = entry.metadata() {
                                let t = m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                if t > *newest {
                                    *newest = t;
                                }
                            }
                        }
                    }
                }
                walk_dir(&src_dir, &mut newest_src);
                newest_src > dist_time
            }
        } else {
            true
        };
        if needs_build {
            // 检查依赖是否已安装
            let deps_installed = web_dir
                .join("node_modules")
                .join(".bin")
                .join("vue-tsc")
                .exists();
            let mut skip_build = false;
            if !deps_installed {
                println!("前端依赖未安装，正在 npm install...");
                let install = std::process::Command::new("npm")
                    .args(["install"])
                    .current_dir(web_dir)
                    .status();
                match install {
                    Ok(s) if s.success() => println!("依赖安装完成"),
                    Ok(s) => {
                        eprintln!("依赖安装失败 (exit: {})", s);
                        skip_build = true;
                    }
                    Err(e) => {
                        eprintln!("无法执行 npm install: {}", e);
                        skip_build = true;
                    }
                }
            }
            if !skip_build {
                println!("前端需要编译，正在自动构建...");
                let status = std::process::Command::new("npm")
                    .args(["run", "build"])
                    .current_dir(web_dir)
                    .status();
                match status {
                    Ok(s) if s.success() => println!("前端编译完成"),
                    Ok(s) => eprintln!("前端编译失败 (exit: {})", s),
                    Err(e) => eprintln!("无法执行 npm: {} (请确保已安装 Node.js)", e),
                }
            } else {
                eprintln!("跳过前端编译，将使用已有 dist 或报 404");
            }
        }
    }

    let state = Arc::new(handlers::member::AppState { db_path: args.db });
    let app = router::create_app(state, &args.static_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
