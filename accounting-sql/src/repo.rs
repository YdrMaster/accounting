/// Account Repository
pub mod account;
/// Attachment Repository
pub mod attachment;
/// Channel Repository
pub mod channel;
/// Commodity Repository
pub mod commodity;
/// Member Repository
pub mod member;
/// Posting Repository
pub mod posting;
/// Tag Repository
pub mod tag;
/// Transaction Repository
pub mod transaction;

use sqlx::SqliteConnection;

use crate::error::DbError;

pub async fn get_setting(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<Option<String>, DbError> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(value)
}

pub async fn set_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}
