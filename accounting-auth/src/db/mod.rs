//! 数据层：连接池初始化与 users/sessions 表访问

pub mod sessions;
pub mod users;

use crate::error::Result;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

/// 认证数据库句柄（内部为 sqlx 连接池）。
#[derive(Debug, Clone)]
pub struct Db {
    pub(crate) pool: SqlitePool,
}

impl Db {
    /// 打开（必要时创建）指定路径的 SQLite 文件并执行 migration。
    ///
    /// 传入 `":memory:"` 时使用内存数据库（测试用）。
    pub async fn init(db_path: &str) -> Result<Self> {
        let options = if db_path == ":memory:" {
            SqliteConnectOptions::from_str("sqlite::memory:")?
        } else {
            SqliteConnectOptions::from_str(&format!("sqlite://{db_path}"))?.create_if_missing(true)
        };
        // 内存库必须用单连接，否则每个连接是独立的空库
        let pool = SqlitePoolOptions::new()
            .max_connections(if db_path == ":memory:" { 1 } else { 5 })
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_db() -> Db {
        Db::init(":memory:").await.expect("init memory db")
    }

    #[tokio::test]
    async fn user_crud_and_unique_constraint() {
        let db = test_db().await;
        let id = db.create_user("alice", "hash1", "Alice").await.unwrap();
        assert!(id > 0);

        let u = db.find_user_by_name("alice").await.unwrap().unwrap();
        assert_eq!(u.username, "alice");
        assert_eq!(u.display_name, "Alice");
        assert!(!u.totp_enabled);
        assert_eq!(u.totp_secret, None);
        assert_eq!(u.recovery_codes, None);
        assert_eq!(u.last_totp_step, None);

        // 用户名唯一约束
        let err = db.create_user("alice", "hash2", "A2").await.unwrap_err();
        assert!(matches!(err, crate::error::AuthError::DuplicateUsername(_)));

        // 改密
        db.update_password("alice", "hash3").await.unwrap();
        let u = db.find_user_by_name("alice").await.unwrap().unwrap();
        assert_eq!(u.password_hash, "hash3");
        // 不存在的用户改密报错
        let err = db.update_password("nobody", "h").await.unwrap_err();
        assert!(matches!(err, crate::error::AuthError::UserNotFound(_)));

        // TOTP 字段流转
        db.set_totp_secret(u.id, b"01234567890123456789")
            .await
            .unwrap();
        let u = db.find_user_by_id(u.id).await.unwrap().unwrap();
        assert!(u.totp_secret.is_some());
        assert!(!u.totp_enabled);
        db.enable_totp(u.id, "[\"h1\",\"h2\"]").await.unwrap();
        db.update_last_totp_step(u.id, 42).await.unwrap();
        let u = db.find_user_by_id(u.id).await.unwrap().unwrap();
        assert!(u.totp_enabled);
        assert_eq!(u.recovery_codes.as_deref(), Some("[\"h1\",\"h2\"]"));
        assert_eq!(u.last_totp_step, Some(42));
        // 重新 setup 重置状态
        db.set_totp_secret(u.id, b"newsecret").await.unwrap();
        let u = db.find_user_by_id(u.id).await.unwrap().unwrap();
        assert!(!u.totp_enabled);
        assert_eq!(u.recovery_codes, None);
        assert_eq!(u.last_totp_step, None);

        // 列表
        db.create_user("bob", "hashb", "Bob").await.unwrap();
        let users = db.list_users().await.unwrap();
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn session_crud_and_unique_constraint() {
        let db = test_db().await;
        let uid = db.create_user("carol", "hash", "Carol").await.unwrap();

        let sid = db.create_session("thash1", uid, false, 1000).await.unwrap();
        let s = db
            .find_session_by_token_hash("thash1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s.id, sid);
        assert_eq!(s.user_id, uid);
        assert!(!s.pending);
        assert_eq!(s.expires_at, 1000);

        // token_hash 唯一约束
        let err = db
            .create_session("thash1", uid, false, 2000)
            .await
            .unwrap_err();
        assert!(
            matches!(err, crate::error::AuthError::Db(sqlx::Error::Database(_))),
            "expected unique violation, got {err:?}"
        );

        // 滑动续期
        db.touch_session(sid, 3000).await.unwrap();
        let s = db
            .find_session_by_token_hash("thash1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s.expires_at, 3000);

        // pending 流转
        let pid = db.create_session("thash2", uid, true, 500).await.unwrap();
        let attempts = db.increment_totp_attempts(pid).await.unwrap();
        assert_eq!(attempts, 1);
        db.promote_session(pid, 9000).await.unwrap();
        let s = db
            .find_session_by_token_hash("thash2")
            .await
            .unwrap()
            .unwrap();
        assert!(!s.pending);
        assert_eq!(s.totp_attempts, 0);
        assert_eq!(s.expires_at, 9000);

        // 删除
        db.delete_session(sid).await.unwrap();
        assert!(
            db.find_session_by_token_hash("thash1")
                .await
                .unwrap()
                .is_none()
        );
    }
}
