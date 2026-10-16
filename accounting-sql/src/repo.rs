/// Account Repository
pub mod account;
/// Account Mapping Repository
pub mod account_mapping;
/// Attachment Repository
pub mod attachment;
/// Budget Repository
pub mod budget;
/// Channel Repository
pub mod channel;
/// ChannelPath Repository
pub mod channel_path;
/// Commodity Repository
pub mod commodity;
/// 门面清单：db/transaction 双侧委托方法的宏与清单
pub mod facade;
/// Member Repository
pub mod member;
/// Posting Repository
pub mod posting;
/// SavingPlan Repository
pub mod saving_plan;
/// Tag Repository
pub mod tag;
/// Transaction Repository
pub mod transaction;

use chrono::NaiveDate;
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

/// deadline 以 TEXT 'YYYY-MM-DD' 存储
pub(crate) fn format_deadline(deadline: Option<NaiveDate>) -> Option<String> {
    deadline.map(|d| d.format("%Y-%m-%d").to_string())
}

pub(crate) fn parse_deadline(raw: &str) -> Result<NaiveDate, DbError> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map_err(|e| DbError::Database(format!("Invalid deadline value: {raw} ({e})")))
}
