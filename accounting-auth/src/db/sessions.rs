//! sessions 表 CRUD

use super::{Db, users::now_unix};
use crate::error::Result;

/// 会话记录。DB 只存 token 的 SHA-256 哈希，不存原文。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    /// 会话 ID。
    pub id: i64,
    /// token 的 SHA-256 哈希（hex）。
    pub token_hash: String,
    /// 所属用户 ID。
    pub user_id: i64,
    /// 是否为等待 TOTP 第二步的 pending 会话。
    pub pending: bool,
    /// pending 会话上 TOTP 连续失败次数。
    pub totp_attempts: i64,
    /// 过期时间（Unix 秒）。
    pub expires_at: i64,
    /// 创建时间（Unix 秒）。
    pub created_at: i64,
}

impl Db {
    /// 创建会话，返回记录 ID。
    pub async fn create_session(
        &self,
        token_hash: &str,
        user_id: i64,
        pending: bool,
        expires_at: i64,
    ) -> Result<i64> {
        let now = now_unix();
        let r = sqlx::query(
            "INSERT INTO sessions (token_hash, user_id, pending, expires_at, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(token_hash)
        .bind(user_id)
        .bind(pending)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(r.last_insert_rowid())
    }

    /// 按 token 哈希查找会话。
    pub async fn find_session_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;
        Ok(session)
    }

    /// 按 ID 删除会话（登出/作废）。
    pub async fn delete_session(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 滑动续期：更新会话过期时间。
    pub async fn touch_session(&self, id: i64, expires_at: i64) -> Result<()> {
        sqlx::query("UPDATE sessions SET expires_at = ? WHERE id = ?")
            .bind(expires_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// pending 会话转正：清除 pending 标记并按正式会话重新设置过期时间。
    pub async fn promote_session(&self, id: i64, expires_at: i64) -> Result<()> {
        sqlx::query(
            "UPDATE sessions SET pending = 0, totp_attempts = 0, expires_at = ? WHERE id = ?",
        )
        .bind(expires_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// TOTP 失败计数 +1，返回最新次数。
    pub async fn increment_totp_attempts(&self, id: i64) -> Result<i64> {
        sqlx::query("UPDATE sessions SET totp_attempts = totp_attempts + 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        let (attempts,): (i64,) = sqlx::query_as("SELECT totp_attempts FROM sessions WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(attempts)
    }
}
