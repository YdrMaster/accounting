//! accounting-auth：独立认证子系统（单 crate 垂直切片，决策见
//! `openspec/changes/archive/2026-08-08-add-user-auth/design.md` D1）
//!
//! 内部按 `db` / `service` / `api` 划分，对外只暴露：
//! - [`AuthState`] / [`init`]：初始化独立 auth.db（sqlx + SQLite）
//! - [`router`]：认证 HTTP API（`/api/auth/*`）
//! - [`require_auth`]：业务路由认证中间件（注入 [`CurrentUser`]）
//!
//! 部署注意：session cookie 带 `Secure`，必须经 HTTPS 访问
//! （决策见上述归档 `design.md` 的 Risks / Trade-offs）。

pub mod api;
pub mod db;
pub mod error;
pub mod service;

pub use api::middleware::{CurrentUser, require_auth};
pub use api::router;
pub use error::{AuthError, Result};

use service::rate_limit::{MemoryRateLimiter, RateLimiter};
use std::sync::Arc;

/// 认证子系统共享状态。
#[derive(Clone)]
pub struct AuthState {
    db: db::Db,
    limiter: Arc<dyn RateLimiter>,
}

impl AuthState {
    /// 认证数据库句柄。
    pub fn db(&self) -> &db::Db {
        &self.db
    }

    /// 频控器。
    pub fn limiter(&self) -> &Arc<dyn RateLimiter> {
        &self.limiter
    }

    /// 用自定义频控器构造（测试或可替换实现用）。
    pub fn with_limiter(db: db::Db, limiter: Arc<dyn RateLimiter>) -> Self {
        Self { db, limiter }
    }
}

/// 初始化认证子系统：打开（必要时创建）`db_path` 的 SQLite 文件并执行 migration。
///
/// `db_path` 为文件路径（如 `"auth.db"`）；测试可传 `":memory:"` 用内存库。
pub async fn init(db_path: &str) -> Result<AuthState> {
    let db = db::Db::init(db_path).await?;
    Ok(AuthState {
        db,
        limiter: Arc::new(MemoryRateLimiter::default()),
    })
}
