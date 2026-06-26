use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::account_mapping::AccountMapping;
use accounting::id::{AccountId, ChannelId, MemberId};

#[derive(FromRow)]
struct AccountMappingRow {
    member_id: i64,
    channel_id: i64,
    category: String,
    account_id: i64,
}

impl AccountMappingRow {
    fn into_model(self) -> AccountMapping {
        AccountMapping {
            member_id: MemberId(self.member_id),
            channel_id: ChannelId(self.channel_id),
            category: self.category,
            account_id: AccountId(self.account_id),
        }
    }
}

/// 插入或更新映射（upsert 语义）
pub async fn mapping_upsert(
    conn: &mut SqliteConnection,
    mapping: &AccountMapping,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO account_mappings (member_id, channel_id, category, account_id)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(member_id, channel_id, category) DO UPDATE SET account_id = excluded.account_id",
    )
    .bind(mapping.member_id.0)
    .bind(mapping.channel_id.0)
    .bind(&mapping.category)
    .bind(mapping.account_id.0)
    .execute(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

/// 查找单条映射
pub async fn mapping_find(
    conn: &mut SqliteConnection,
    member_id: MemberId,
    channel_id: ChannelId,
    category: &str,
) -> Result<Option<AccountMapping>, DbError> {
    let row: Option<AccountMappingRow> = sqlx::query_as(
        "SELECT member_id, channel_id, category, account_id
         FROM account_mappings
         WHERE member_id = ?1 AND channel_id = ?2 AND category = ?3",
    )
    .bind(member_id.0)
    .bind(channel_id.0)
    .bind(category)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_model()))
}

/// 列出某个 (成员, 渠道) 的所有映射
pub async fn mapping_list(
    conn: &mut SqliteConnection,
    member_id: MemberId,
    channel_id: ChannelId,
) -> Result<Vec<AccountMapping>, DbError> {
    let rows: Vec<AccountMappingRow> = sqlx::query_as(
        "SELECT member_id, channel_id, category, account_id
         FROM account_mappings
         WHERE member_id = ?1 AND channel_id = ?2
         ORDER BY category",
    )
    .bind(member_id.0)
    .bind(channel_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_model()).collect())
}

/// 删除单条映射
pub async fn mapping_delete(
    conn: &mut SqliteConnection,
    member_id: MemberId,
    channel_id: ChannelId,
    category: &str,
) -> Result<bool, DbError> {
    let result = sqlx::query(
        "DELETE FROM account_mappings
         WHERE member_id = ?1 AND channel_id = ?2 AND category = ?3",
    )
    .bind(member_id.0)
    .bind(channel_id.0)
    .bind(category)
    .execute(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(result.rows_affected() > 0)
}

/// 列出全部映射
pub async fn mapping_list_all(conn: &mut SqliteConnection) -> Result<Vec<AccountMapping>, DbError> {
    let rows: Vec<AccountMappingRow> = sqlx::query_as(
        "SELECT member_id, channel_id, category, account_id
         FROM account_mappings
         ORDER BY member_id, channel_id, category",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_model()).collect())
}

/// 统计引用某账户的映射数量
pub async fn mapping_count_by_account(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<i64, DbError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM account_mappings WHERE account_id = ?1")
            .bind(account_id.0)
            .fetch_one(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::channel::Channel;
    use accounting::member::Member;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> (SqliteConnection, MemberId, ChannelId, AccountId) {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();

        // 创建成员
        let member = Member {
            id: MemberId(0),
            name: "测试".to_string(),
        };
        let member_id = crate::repo::member::member_create(&mut conn, &member)
            .await
            .unwrap();

        // 创建渠道
        let channel = Channel {
            id: ChannelId(0),
            name: "TestPay".to_string(),
            description: None,
            account_id: None,
            is_system: false,
        };
        let channel_id = crate::repo::channel::channel_create(&mut conn, &channel)
            .await
            .unwrap();

        // 使用种子数据中的 Assets 账户 (id=1)
        let account_id = AccountId(1);

        (conn, member_id, channel_id, account_id)
    }

    #[tokio::test]
    async fn test_upsert_and_find() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let mapping = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        mapping_upsert(&mut conn, &mapping).await.unwrap();
        let found = mapping_find(&mut conn, member_id, channel_id, "收支:餐饮美食")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().account_id, account_id);
    }

    #[tokio::test]
    async fn test_upsert_overwrite() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let mapping1 = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        let mapping2 = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id: AccountId(2),
        };
        mapping_upsert(&mut conn, &mapping1).await.unwrap();
        mapping_upsert(&mut conn, &mapping2).await.unwrap();
        let found = mapping_find(&mut conn, member_id, channel_id, "收支:餐饮美食")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.account_id, AccountId(2));
    }

    #[tokio::test]
    async fn test_list() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let m1 = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        let m2 = AccountMapping {
            member_id,
            channel_id,
            category: "资产:信用卡".to_string(),
            account_id: AccountId(2),
        };
        mapping_upsert(&mut conn, &m1).await.unwrap();
        mapping_upsert(&mut conn, &m2).await.unwrap();
        let list = mapping_list(&mut conn, member_id, channel_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let mapping = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        mapping_upsert(&mut conn, &mapping).await.unwrap();
        let deleted = mapping_delete(&mut conn, member_id, channel_id, "收支:餐饮美食")
            .await
            .unwrap();
        assert!(deleted);
        let found = mapping_find(&mut conn, member_id, channel_id, "收支:餐饮美食")
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let (mut conn, member_id, channel_id, _account_id) = setup().await;
        let deleted = mapping_delete(&mut conn, member_id, channel_id, "收支:不存在")
            .await
            .unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_count_by_account() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let m1 = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        let m2 = AccountMapping {
            member_id,
            channel_id,
            category: "收支:交通出行".to_string(),
            account_id,
        };
        mapping_upsert(&mut conn, &m1).await.unwrap();
        mapping_upsert(&mut conn, &m2).await.unwrap();
        let count = mapping_count_by_account(&mut conn, account_id)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_delete_account_blocked_by_mapping() {
        let (mut conn, member_id, channel_id, account_id) = setup().await;
        let mapping = AccountMapping {
            member_id,
            channel_id,
            category: "收支:餐饮美食".to_string(),
            account_id,
        };
        mapping_upsert(&mut conn, &mapping).await.unwrap();

        // 验证 count > 0
        let count = mapping_count_by_account(&mut conn, account_id)
            .await
            .unwrap();
        assert!(count > 0);

        // 直接删除账户应被外键约束拒绝
        let result = crate::repo::account::account_delete(&mut conn, account_id).await;
        assert!(result.is_err(), "被映射引用的账户不应被删除");
    }
}
