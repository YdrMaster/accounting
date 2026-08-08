//! users 表 CRUD

use super::Db;
use crate::error::{AuthError, Result};

/// 用户记录。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    /// 用户 ID。
    pub id: i64,
    /// 用户名（唯一）。
    pub username: String,
    /// argon2id 密码哈希。
    pub password_hash: String,
    /// 显示名。
    pub display_name: String,
    /// TOTP 密钥原始字节（未设置为 None）。
    pub totp_secret: Option<Vec<u8>>,
    /// 是否已开启 TOTP。
    pub totp_enabled: bool,
    /// 恢复码哈希 JSON 数组。
    pub recovery_codes: Option<String>,
    /// 上次成功验证的 TOTP 时间步（防重放）。
    pub last_totp_step: Option<i64>,
    /// 创建时间（Unix 秒）。
    pub created_at: i64,
}

impl Db {
    /// 创建用户。用户名重复时返回 [`AuthError::DuplicateUsername`]。
    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<i64> {
        let now = now_unix();
        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, display_name, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .bind(now)
        .execute(&self.pool)
        .await;
        match result {
            Ok(r) => Ok(r.last_insert_rowid()),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                Err(AuthError::DuplicateUsername(username.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// 按用户名查找用户。
    pub async fn find_user_by_name(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// 按 ID 查找用户。
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// 列出全部用户（按 ID 升序）。
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    /// 更新用户密码哈希。用户不存在时返回 [`AuthError::UserNotFound`]。
    pub async fn update_password(&self, username: &str, password_hash: &str) -> Result<()> {
        let r = sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
            .bind(password_hash)
            .bind(username)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(AuthError::UserNotFound(username.to_string()));
        }
        Ok(())
    }

    /// 写入新的 TOTP 密钥；同时重置 totp_enabled、恢复码与防重放时间步，
    /// 保证旧 secret/恢复码立即失效，必须重新 enable 才生效。
    pub async fn set_totp_secret(&self, user_id: i64, secret: &[u8]) -> Result<()> {
        sqlx::query(
            "UPDATE users
             SET totp_secret = ?, totp_enabled = 0, recovery_codes = NULL, last_totp_step = NULL
             WHERE id = ?",
        )
        .bind(secret)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 标记 TOTP 已开启，并写入恢复码哈希 JSON。
    pub async fn enable_totp(&self, user_id: i64, recovery_hashes_json: &str) -> Result<()> {
        sqlx::query("UPDATE users SET totp_enabled = 1, recovery_codes = ? WHERE id = ?")
            .bind(recovery_hashes_json)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新恢复码哈希 JSON（消费恢复码后回写）。
    pub async fn update_recovery_codes(
        &self,
        user_id: i64,
        recovery_hashes_json: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE users SET recovery_codes = ? WHERE id = ?")
            .bind(recovery_hashes_json)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 记录最近一次成功验证的 TOTP 时间步（防重放）。
    pub async fn update_last_totp_step(&self, user_id: i64, step: i64) -> Result<()> {
        sqlx::query("UPDATE users SET last_totp_step = ? WHERE id = ?")
            .bind(step)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// 当前 Unix 秒。
pub(crate) fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}
