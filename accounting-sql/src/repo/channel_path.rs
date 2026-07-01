use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::channel_path::{ChannelPath, ChannelPathNode, ChannelPathStatus};
use accounting::id::{ChannelId, ChannelPathId, TransactionId};

#[derive(FromRow)]
struct ChannelPathRow {
    id: i64,
    transaction_id: i64,
    position: i32,
    channel_id: i64,
    status: i32,
}

impl ChannelPathRow {
    fn into_channel_path(self) -> ChannelPath {
        ChannelPath {
            id: ChannelPathId(self.id),
            transaction_id: TransactionId(self.transaction_id),
            position: self.position,
            channel_id: ChannelId(self.channel_id),
            status: ChannelPathStatus::from_i32(self.status),
        }
    }
}

/// 创建单条链路记录
pub async fn channel_path_create(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
    node: &ChannelPathNode,
) -> Result<ChannelPathId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO channel_paths (transaction_id, position, channel_id, status)
         VALUES (?1, ?2, ?3, ?4) RETURNING id",
    )
    .bind(transaction_id.0)
    .bind(node.position)
    .bind(node.channel_id.0)
    .bind(node.status.as_i32())
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(ChannelPathId(id))
}

/// 查询指定交易的所有链路记录，按 position 升序、id 升序排列
pub async fn channel_path_list_by_transaction(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
) -> Result<Vec<ChannelPath>, DbError> {
    let rows: Vec<ChannelPathRow> = sqlx::query_as(
        "SELECT id, transaction_id, position, channel_id, status
         FROM channel_paths
         WHERE transaction_id = ?1
         ORDER BY position ASC, id ASC",
    )
    .bind(transaction_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_channel_path()).collect())
}

/// 删除指定交易的所有链路记录
pub async fn channel_path_delete_by_transaction(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM channel_paths WHERE transaction_id = ?1")
        .bind(transaction_id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

/// 查询引用指定渠道的所有交易 ID（去重）
pub async fn channel_path_find_transactions_by_channel(
    conn: &mut SqliteConnection,
    channel_id: ChannelId,
) -> Result<Vec<TransactionId>, DbError> {
    let rows: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT transaction_id FROM channel_paths WHERE channel_id = ?1",
    )
    .bind(channel_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(TransactionId).collect())
}

/// 查询引用指定渠道的记录数
pub async fn channel_path_count_by_channel(
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

/// 更新链路节点的状态
pub async fn channel_path_update_status(
    conn: &mut SqliteConnection,
    id: ChannelPathId,
    status: ChannelPathStatus,
) -> Result<(), DbError> {
    sqlx::query("UPDATE channel_paths SET status = ?1 WHERE id = ?2")
        .bind(status.as_i32())
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

/// 按 ID 获取单条链路记录
pub async fn channel_path_get(
    conn: &mut SqliteConnection,
    id: ChannelPathId,
) -> Result<Option<ChannelPath>, DbError> {
    let row: Option<ChannelPathRow> = sqlx::query_as(
        "SELECT id, transaction_id, position, channel_id, status FROM channel_paths WHERE id = ?1",
    )
    .bind(id.0)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_channel_path()))
}

/// 批量创建链路记录
pub async fn channel_path_create_batch(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
    nodes: &[ChannelPathNode],
) -> Result<(), DbError> {
    for node in nodes {
        channel_path_create(conn, transaction_id, node).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{ChannelId, TransactionId};
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
        conn
    }

    async fn insert_channel(conn: &mut SqliteConnection, name: &str) -> ChannelId {
        let id: i64 = sqlx::query_scalar("INSERT INTO channels (name) VALUES (?1) RETURNING id")
            .bind(name)
            .fetch_one(conn)
            .await
            .unwrap();
        ChannelId(id)
    }

    async fn ensure_test_member(conn: &mut SqliteConnection) -> i64 {
        sqlx::query("INSERT OR IGNORE INTO members (name) VALUES ('Test Member')")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query_scalar("SELECT id FROM members WHERE name = 'Test Member'")
            .fetch_one(conn)
            .await
            .unwrap()
    }

    async fn insert_transaction(conn: &mut SqliteConnection) -> TransactionId {
        let member_id = ensure_test_member(conn).await;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-01-01 00:00:00', 'test', ?1) RETURNING id",
        )
        .bind(member_id)
        .fetch_one(conn)
        .await
        .unwrap();
        TransactionId(id)
    }

    #[tokio::test]
    async fn test_create_and_list() {
        let mut conn = setup().await;
        let ch1 = insert_channel(&mut conn, "Taobao").await;
        let ch2 = insert_channel(&mut conn, "TestPay").await;
        let ch3 = insert_channel(&mut conn, "Huabei").await;
        let tx_id = insert_transaction(&mut conn).await;

        let nodes = vec![
            ChannelPathNode {
                position: 0,
                channel_id: ch1,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 1,
                channel_id: ch2,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 2,
                channel_id: ch3,
                status: ChannelPathStatus::Default,
            },
        ];
        channel_path_create_batch(&mut conn, tx_id, &nodes)
            .await
            .unwrap();

        let paths = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].position, 0);
        assert_eq!(paths[1].position, 1);
        assert_eq!(paths[2].position, 2);
    }

    #[tokio::test]
    async fn test_terminal_multi_item() {
        let mut conn = setup().await;
        let ch1 = insert_channel(&mut conn, "Taobao").await;
        let ch2 = insert_channel(&mut conn, "TestPay").await;
        let ch3 = insert_channel(&mut conn, "Huabei").await;
        let ch4 = insert_channel(&mut conn, "CreditCard").await;
        let tx_id = insert_transaction(&mut conn).await;

        let nodes = vec![
            ChannelPathNode {
                position: 0,
                channel_id: ch1,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 1,
                channel_id: ch2,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 2,
                channel_id: ch3,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 2,
                channel_id: ch4,
                status: ChannelPathStatus::Default,
            },
        ];
        channel_path_create_batch(&mut conn, tx_id, &nodes)
            .await
            .unwrap();

        let paths = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert_eq!(paths.len(), 4);
        assert_eq!(paths[2].position, 2);
        assert_eq!(paths[3].position, 2);
    }

    #[tokio::test]
    async fn test_update_reconciled() {
        let mut conn = setup().await;
        let ch = insert_channel(&mut conn, "Cash").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node = ChannelPathNode {
            position: 0,
            channel_id: ch,
            status: ChannelPathStatus::Default,
        };
        let path_id = channel_path_create(&mut conn, tx_id, &node).await.unwrap();

        channel_path_update_status(&mut conn, path_id, ChannelPathStatus::Verified)
            .await
            .unwrap();

        let path = channel_path_get(&mut conn, path_id).await.unwrap().unwrap();
        assert_eq!(path.status, ChannelPathStatus::Verified);
    }

    #[tokio::test]
    async fn test_find_transactions_by_channel() {
        let mut conn = setup().await;
        let ch = insert_channel(&mut conn, "WeChat").await;
        let tx1 = insert_transaction(&mut conn).await;
        let tx2 = insert_transaction(&mut conn).await;

        let node = ChannelPathNode {
            position: 0,
            channel_id: ch,
            status: ChannelPathStatus::Default,
        };
        channel_path_create(&mut conn, tx1, &node).await.unwrap();
        channel_path_create(&mut conn, tx2, &node).await.unwrap();

        let tx_ids = channel_path_find_transactions_by_channel(&mut conn, ch)
            .await
            .unwrap();
        assert_eq!(tx_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_by_transaction() {
        let mut conn = setup().await;
        let ch = insert_channel(&mut conn, "Cash").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node = ChannelPathNode {
            position: 0,
            channel_id: ch,
            status: ChannelPathStatus::Default,
        };
        channel_path_create(&mut conn, tx_id, &node).await.unwrap();

        channel_path_delete_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        let paths = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert!(paths.is_empty());
    }

    #[tokio::test]
    async fn test_count_by_channel() {
        let mut conn = setup().await;
        let ch = insert_channel(&mut conn, "PayPal").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node = ChannelPathNode {
            position: 0,
            channel_id: ch,
            status: ChannelPathStatus::Default,
        };
        channel_path_create(&mut conn, tx_id, &node).await.unwrap();

        let count = channel_path_count_by_channel(&mut conn, ch).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_cascade_delete_on_transaction_delete() {
        // When a transaction is deleted, channel_paths should be cascade-deleted
        let mut conn = setup().await;
        let ch1 = insert_channel(&mut conn, "Taobao").await;
        let ch2 = insert_channel(&mut conn, "TestPay").await;
        let tx_id = insert_transaction(&mut conn).await;

        let nodes = vec![
            ChannelPathNode {
                position: 0,
                channel_id: ch1,
                status: ChannelPathStatus::Default,
            },
            ChannelPathNode {
                position: 1,
                channel_id: ch2,
                status: ChannelPathStatus::Default,
            },
        ];
        channel_path_create_batch(&mut conn, tx_id, &nodes)
            .await
            .unwrap();

        // Verify paths exist
        let paths = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert_eq!(paths.len(), 2);

        // Delete the transaction directly — CASCADE should clean up channel_paths
        sqlx::query("DELETE FROM transactions WHERE id = ?1")
            .bind(tx_id.0)
            .execute(&mut conn)
            .await
            .unwrap();

        let paths = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert!(paths.is_empty(), "channel_paths should be cascade-deleted");
    }

    #[tokio::test]
    async fn test_channel_delete_rejected_when_referenced() {
        // Verify that channel_count_transactions_by_id detects channel_path references
        let mut conn = setup().await;
        let ch = insert_channel(&mut conn, "TestPay").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node = ChannelPathNode {
            position: 0,
            channel_id: ch,
            status: ChannelPathStatus::Default,
        };
        channel_path_create(&mut conn, tx_id, &node).await.unwrap();

        let count = crate::repo::channel::channel_count_transactions_by_id(&mut conn, ch)
            .await
            .unwrap();
        assert!(
            count > 0,
            "Channel with channel_paths should report references"
        );
    }

    #[tokio::test]
    async fn test_status_update() {
        // Test updating channel path status
        let mut conn = setup().await;
        let ch1 = insert_channel(&mut conn, "Taobao").await;
        let ch2 = insert_channel(&mut conn, "TestPay").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node1 = ChannelPathNode {
            position: 0,
            channel_id: ch1,
            status: ChannelPathStatus::Default,
        };
        let node2 = ChannelPathNode {
            position: 1,
            channel_id: ch2,
            status: ChannelPathStatus::Default,
        };
        let id1 = channel_path_create(&mut conn, tx_id, &node1).await.unwrap();
        let id2 = channel_path_create(&mut conn, tx_id, &node2).await.unwrap();

        // Mark first as verified
        channel_path_update_status(&mut conn, id1, ChannelPathStatus::Verified)
            .await
            .unwrap();

        let path1 = channel_path_get(&mut conn, id1).await.unwrap().unwrap();
        let path2 = channel_path_get(&mut conn, id2).await.unwrap().unwrap();
        assert_eq!(path1.status, ChannelPathStatus::Verified);
        assert_eq!(
            path2.status,
            ChannelPathStatus::Default,
            "Second path should remain default"
        );

        // Unmark it
        channel_path_update_status(&mut conn, id1, ChannelPathStatus::Default)
            .await
            .unwrap();

        let path1 = channel_path_get(&mut conn, id1).await.unwrap().unwrap();
        assert_eq!(path1.status, ChannelPathStatus::Default);
    }

    #[tokio::test]
    async fn test_query_default_paths() {
        // Query only default status paths for a transaction
        let mut conn = setup().await;
        let ch1 = insert_channel(&mut conn, "Taobao").await;
        let ch2 = insert_channel(&mut conn, "TestPay").await;
        let tx_id = insert_transaction(&mut conn).await;

        let node1 = ChannelPathNode {
            position: 0,
            channel_id: ch1,
            status: ChannelPathStatus::Default,
        };
        let node2 = ChannelPathNode {
            position: 1,
            channel_id: ch2,
            status: ChannelPathStatus::Default,
        };
        let id1 = channel_path_create(&mut conn, tx_id, &node1).await.unwrap();
        let id2 = channel_path_create(&mut conn, tx_id, &node2).await.unwrap();

        // Mark first as verified
        channel_path_update_status(&mut conn, id1, ChannelPathStatus::Verified)
            .await
            .unwrap();

        // List all paths and filter default
        let all = channel_path_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        let default_paths: Vec<_> = all
            .iter()
            .filter(|p| p.status == ChannelPathStatus::Default)
            .collect();
        assert_eq!(default_paths.len(), 1);
        assert_eq!(default_paths[0].id, id2);
    }
}
