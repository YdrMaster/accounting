use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::channel::Channel;
use accounting::id::{AccountId, ChannelId};

#[derive(FromRow)]
struct ChannelRow {
    id: i64,
    name: String,
    description: Option<String>,
    account_id: Option<i64>,
}

impl ChannelRow {
    fn into_channel(self) -> Channel {
        Channel {
            id: ChannelId(self.id),
            name: self.name,
            description: self.description,
            account_id: self.account_id.map(AccountId),
        }
    }
}

pub async fn channel_create(
    conn: &mut SqliteConnection,
    channel: &Channel,
) -> Result<ChannelId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO channels (name, description, account_id) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(&channel.name)
    .bind(&channel.description)
    .bind(channel.account_id.map(|id| id.0))
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(ChannelId(id))
}

pub async fn channel_get(
    conn: &mut SqliteConnection,
    id: ChannelId,
) -> Result<Option<Channel>, DbError> {
    let row: Option<ChannelRow> =
        sqlx::query_as("SELECT id, name, description, account_id FROM channels WHERE id = ?1")
            .bind(id.0)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_channel()))
}

pub async fn channel_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Channel>, DbError> {
    let row: Option<ChannelRow> =
        sqlx::query_as("SELECT id, name, description, account_id FROM channels WHERE name = ?1")
            .bind(name)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_channel()))
}

pub async fn channel_list(conn: &mut SqliteConnection) -> Result<Vec<Channel>, DbError> {
    let rows: Vec<ChannelRow> =
        sqlx::query_as("SELECT id, name, description, account_id FROM channels ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_channel()).collect())
}

pub async fn channel_count_transactions_by_id(
    conn: &mut SqliteConnection,
    channel_id: ChannelId,
) -> Result<i64, DbError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channel_paths WHERE channel_id = ?1")
        .bind(channel_id.0)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(count)
}

pub async fn channel_force_delete_by_id(
    conn: &mut SqliteConnection,
    channel_id: ChannelId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM channels WHERE id = ?1")
        .bind(channel_id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn channel_update(
    conn: &mut SqliteConnection,
    id: ChannelId,
    account_id: Option<AccountId>,
) -> Result<(), DbError> {
    sqlx::query("UPDATE channels SET account_id = ?1 WHERE id = ?2")
        .bind(account_id.map(|id| id.0))
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            name: "Alipay".to_string(),
            description: Some("支付宝".to_string()),
            account_id: None,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Alipay");
        assert!(fetched.account_id.is_none());
    }

    #[tokio::test]
    async fn test_create_with_account_id() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            name: "Huabei".to_string(),
            description: None,
            account_id: Some(AccountId(1)),
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.account_id, Some(AccountId(1)));
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            name: "WeChat".to_string(),
            description: None,
            account_id: None,
        };
        channel_create(&mut conn, &channel).await.unwrap();
        let list = channel_list(&mut conn).await.unwrap();
        assert!(list.iter().any(|c| c.name == "WeChat"));
    }

    #[tokio::test]
    async fn test_count_and_force_delete() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            name: "PayPal".to_string(),
            description: None,
            account_id: None,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let count = channel_count_transactions_by_id(&mut conn, id)
            .await
            .unwrap();
        assert_eq!(count, 0);
        channel_force_delete_by_id(&mut conn, id).await.unwrap();
        assert!(channel_get(&mut conn, id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_update_account_id() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            name: "CC".to_string(),
            description: None,
            account_id: None,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        channel_update(&mut conn, id, Some(AccountId(1)))
            .await
            .unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.account_id, Some(AccountId(1)));
    }
}
