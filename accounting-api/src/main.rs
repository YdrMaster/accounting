//! accounting-api: axum HTTP 服务入口

rust_i18n::i18n!("locales", fallback = "en");

mod dto;
mod handlers;
mod router;

use accounting::error::AccountingError;
use axum::{Json, http::StatusCode, response::IntoResponse};
use clap::Parser;
use dto::ErrorResponse;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::sync::Arc;

/// 将 AccountingError 转换为 HTTP 响应
pub fn account_error(err: AccountingError) -> impl IntoResponse {
    let msg = err.to_string();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: msg }),
    )
}

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
    /// 语言（如 zh-CN、en），默认 zh-CN
    #[arg(long)]
    lang: Option<String>,
}

/// 检测 PATH 中是否包含 npm
fn find_npm_debug() {
    if let Ok(path) = std::env::var("PATH") {
        for segment in path.split(';') {
            for name in &["npm.cmd", "npm", "npm.ps1"] {
                let candidate = Path::new(segment).join(name);
                if candidate.exists() {
                    println!("{}", rust_i18n::t!("npm_found", path = candidate.display()));
                    return;
                }
            }
        }
        println!("{}", rust_i18n::t!("npm_not_found"));
    }
}

/// 在 Windows 上通过 cmd /c 调用 npm，其他平台直接调用
fn npm_command(args: &[&str], cwd: &Path) -> io::Result<ExitStatus> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd_args = vec!["/c", "npm"];
        cmd_args.extend(args);
        Command::new("cmd")
            .args(&cmd_args)
            .current_dir(cwd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("npm").args(args).current_dir(cwd).status()
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 自动编译前端：检测 accounting-web 目录是否存在，dist 是否过期
    let dist_path = Path::new(&args.static_dir);
    let web_dir = Path::new("accounting-web");
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
                fn walk_dir(dir: &Path, newest: &mut std::time::SystemTime) {
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
                println!("{}", rust_i18n::t!("npm_installing"));
                find_npm_debug();
                match npm_command(&["install"], web_dir) {
                    Ok(s) if s.success() => println!("{}", rust_i18n::t!("npm_install_done")),
                    Ok(s) => {
                        eprintln!("{}", rust_i18n::t!("npm_install_failed", exit = s));
                        skip_build = true;
                    }
                    Err(e) => {
                        eprintln!("{}", rust_i18n::t!("npm_install_exec_failed", error = e));
                        skip_build = true;
                    }
                }
            }
            if !skip_build {
                println!("{}", rust_i18n::t!("frontend_building"));
                match npm_command(&["run", "build"], web_dir) {
                    Ok(s) if s.success() => println!("{}", rust_i18n::t!("frontend_build_done")),
                    Ok(s) => eprintln!("{}", rust_i18n::t!("frontend_build_failed", exit = s)),
                    Err(e) => eprintln!("{}", rust_i18n::t!("npm_exec_failed", error = e)),
                }
            } else {
                eprintln!("{}", rust_i18n::t!("frontend_build_skipped"));
            }
        }
    }

    let db = accounting_sql::SqliteDatabase::open(&args.db)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        });
    let lang = db
        .initialize(args.lang.as_deref())
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        });
    rust_i18n::set_locale(&lang);

    let state = Arc::new(handlers::member::AppState { db });
    let app = router::create_app(state, &args.static_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on http://localhost:{}", args.port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
