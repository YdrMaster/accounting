use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use crate::names::CHANNEL_NAMES;
use accounting::channel::Channel;
use accounting::id::{AccountId, ChannelId};

#[derive(FromRow)]
struct ChannelRow {
    id: i64,
    description: Option<String>,
    account_id: Option<i64>,
    is_system: i32,
}

impl ChannelRow {
    fn into_channel(self) -> Channel {
        Channel {
            id: ChannelId(self.id),
            description: self.description,
            account_id: self.account_id.map(AccountId),
            is_system: self.is_system != 0,
        }
    }
}

fn validate_channel_name(name: &str) -> Result<(), DbError> {
    if name.contains("->") || name.contains("&") || name.contains('*') || name.contains('√') {
        return Err(DbError::Database(
            "渠道名称不能包含 ->、&、* 或 √".to_string(),
        ));
    }
    Ok(())
}

pub async fn channel_create(
    conn: &mut SqliteConnection,
    channel: &Channel,
) -> Result<ChannelId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO channels (description, account_id, is_system) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(&channel.description)
    .bind(channel.account_id.map(|id| id.0))
    .bind(channel.is_system as i32)
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
        sqlx::query_as("SELECT id, description, account_id, is_system FROM channels WHERE id = ?1")
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
    let row: Option<ChannelRow> = sqlx::query_as(
        "SELECT c.id, c.description, c.account_id, c.is_system FROM channels c JOIN channel_names cn ON cn.channel_id = c.id WHERE cn.name = ?1",
    )
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_channel()))
}

/// 通过渠道名称查找渠道，大小写不敏感。
pub async fn channel_resolve_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Channel>, DbError> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }

    let row: Option<ChannelRow> = sqlx::query_as(
        "SELECT c.id, c.description, c.account_id, c.is_system FROM channels c JOIN channel_names cn ON cn.channel_id = c.id WHERE LOWER(cn.name) = LOWER(?1)",
    )
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_channel()))
}

pub async fn channel_list(conn: &mut SqliteConnection) -> Result<Vec<Channel>, DbError> {
    let rows: Vec<ChannelRow> =
        sqlx::query_as("SELECT id, description, account_id, is_system FROM channels ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_channel()).collect())
}

pub async fn channel_upsert_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
    description: Option<&str>,
    account_id: Option<AccountId>,
) -> Result<ChannelId, DbError> {
    validate_channel_name(name)?;
    if let Some(existing) = channel_get_by_name(conn, name).await? {
        sqlx::query("UPDATE channels SET description = ?1, account_id = ?2 WHERE id = ?3")
            .bind(description)
            .bind(account_id.map(|id| id.0))
            .bind(existing.id.0)
            .execute(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(existing.id)
    } else {
        let channel = Channel {
            id: ChannelId(0),
            description: description.map(|s| s.to_string()),
            account_id,
            is_system: false,
        };
        let channel_id = channel_create(conn, &channel).await?;
        CHANNEL_NAMES
            .insert(conn, channel_id.0, lang, name, false, true)
            .await?;
        Ok(channel_id)
    }
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
    let is_system: i32 = sqlx::query_scalar("SELECT is_system FROM channels WHERE id = ?1")
        .bind(channel_id.0)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?
        .ok_or_else(|| DbError::Database(format!("渠道 {} 不存在", channel_id.0)))?;

    if is_system != 0 {
        return Err(DbError::Database("系统内置渠道不可删除".to_string()));
    }

    sqlx::query("DELETE FROM channels WHERE id = ?1")
        .bind(channel_id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

/// 渠道改名：把渠道在 `lang` 语言下的显示名改为 `new_name`
///（系统名字不可改文本：降级为非显示名并插入用户自定义显示名）
pub async fn channel_rename(
    conn: &mut SqliteConnection,
    id: ChannelId,
    new_name: &str,
    lang: &str,
) -> Result<(), DbError> {
    validate_channel_name(new_name)?;
    if channel_get(conn, id).await?.is_none() {
        return Err(DbError::Database(format!("渠道 {} 不存在", id.0)));
    }
    CHANNEL_NAMES
        .rename_display(conn, id.0, None, lang, new_name)
        .await
}

pub async fn channel_update(
    conn: &mut SqliteConnection,
    id: ChannelId,
    description: Option<&str>,
    account_id: Option<AccountId>,
) -> Result<(), DbError> {
    sqlx::query("UPDATE channels SET description = ?1, account_id = ?2 WHERE id = ?3")
        .bind(description)
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
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: Some("测试支付".to_string()),
            account_id: None,
            is_system: false,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert!(fetched.account_id.is_none());
    }

    #[tokio::test]
    async fn test_create_with_account_id() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: Some(AccountId(1)),
            is_system: false,
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
            description: None,
            account_id: None,
            is_system: false,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let list = channel_list(&mut conn).await.unwrap();
        assert!(list.iter().any(|c| c.id == id));
    }

    #[tokio::test]
    async fn test_count_and_force_delete() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: false,
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
            description: None,
            account_id: None,
            is_system: false,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        channel_update(&mut conn, id, None, Some(AccountId(1)))
            .await
            .unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.account_id, Some(AccountId(1)));
    }

    #[tokio::test]
    async fn test_is_system_read_write() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: true,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert!(fetched.is_system);

        let user_channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: false,
        };
        let id2 = channel_create(&mut conn, &user_channel).await.unwrap();
        let fetched2 = channel_get(&mut conn, id2).await.unwrap().unwrap();
        assert!(!fetched2.is_system);
    }

    #[tokio::test]
    async fn test_system_channel_delete_rejected() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: true,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        let result = channel_force_delete_by_id(&mut conn, id).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("系统内置渠道不可删除")
        );
    }

    #[tokio::test]
    async fn test_user_channel_delete_allowed() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: false,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        channel_force_delete_by_id(&mut conn, id).await.unwrap();
        assert!(channel_get(&mut conn, id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_channel_name_with_ampersand_rejected() {
        let mut conn = setup().await;
        let result = channel_upsert_by_name(&mut conn, "A&B", "en", None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能包含"));
    }

    #[tokio::test]
    async fn test_upsert_channel_name_with_arrow_rejected() {
        let mut conn = setup().await;
        let result = channel_upsert_by_name(&mut conn, "A->B", "en", None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能包含"));
    }

    #[tokio::test]
    async fn test_channel_upsert_creates_with_name() {
        let mut conn = setup().await;

        // 创建成功：渠道与名字行一并写入，语言为调用方传入值
        let id = channel_upsert_by_name(&mut conn, "云闪付", "zh-CN", Some("银联"), None)
            .await
            .unwrap();
        let (lang, is_display): (String, i32) = sqlx::query_as(
            "SELECT lang, is_display FROM channel_names WHERE channel_id = ?1 AND name = '云闪付'",
        )
        .bind(id.0)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(lang, "zh-CN");
        assert_eq!(is_display, 1);

        // 按名字（NOCASE / 任意语言）命中
        let found = channel_resolve_by_name(&mut conn, "云闪付").await.unwrap();
        assert_eq!(found.unwrap().id, id);

        // 再次 upsert 同名 → 更新而非新建
        let again = channel_upsert_by_name(&mut conn, "云闪付", "zh-CN", Some("银联云闪付"), None)
            .await
            .unwrap();
        assert_eq!(again, id);
        let fetched = channel_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.description, Some("银联云闪付".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_channel_by_name_case_insensitive() {
        let mut conn = setup().await;
        let channel = Channel {
            id: ChannelId(0),
            description: None,
            account_id: None,
            is_system: false,
        };
        let id = channel_create(&mut conn, &channel).await.unwrap();
        sqlx::query("INSERT INTO channel_names (channel_id, lang, name, is_display) VALUES (?1, 'en', ?2, 1)")
            .bind(id.0)
            .bind("TestPay")
            .execute(&mut conn)
            .await
            .unwrap();

        let found = channel_resolve_by_name(&mut conn, "testpay").await.unwrap();
        assert_eq!(found.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_resolve_channel_by_alias() {
        // 种子数据中应存在 Alipay 及其名称记录，可通过大小写不敏感查找定位
        let mut conn = setup().await;
        let found = channel_resolve_by_name(&mut conn, "alipay").await.unwrap();
        assert!(found.is_some());

        let found = channel_resolve_by_name(&mut conn, "支付宝").await.unwrap();
        assert!(found.is_some());
    }
}
