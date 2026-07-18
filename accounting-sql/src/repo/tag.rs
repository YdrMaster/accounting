use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use crate::names::TAG_NAMES;
use accounting::id::{TagId, TransactionId};
use accounting::tag::Tag;
use std::collections::HashMap;

#[derive(FromRow)]
struct TagRow {
    id: i64,
    description: Option<String>,
    is_system: i32,
}

impl TagRow {
    fn into_tag(self) -> Tag {
        Tag {
            id: TagId(self.id),
            description: self.description,
            is_system: self.is_system != 0,
        }
    }
}

pub async fn tag_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Tag>, DbError> {
    let row: Option<TagRow> =
        sqlx::query_as("SELECT t.id, t.description, t.is_system FROM tags t JOIN tag_names tn ON tn.tag_id = t.id WHERE tn.name = ?1")
            .bind(name)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_tag()))
}

pub async fn tag_list(conn: &mut SqliteConnection) -> Result<Vec<Tag>, DbError> {
    let rows: Vec<TagRow> =
        sqlx::query_as("SELECT id, description, is_system FROM tags ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_tag()).collect())
}

pub async fn tag_create(conn: &mut SqliteConnection, tag: &Tag) -> Result<TagId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO tags (description, is_system) VALUES (?1, ?2) RETURNING id",
    )
    .bind(&tag.description)
    .bind(tag.is_system as i32)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(TagId(id))
}

/// 更新标签描述并按需改名。
///
/// 名字与当前 (tag, lang) 显示名相同则不触发改名（系统标签也可仅更新描述）；
/// 不同则走 names 改名路径：系统名降级为非显示（文本保留）、撞名拒绝、该语言无显示名时新增。
pub async fn tag_update(
    conn: &mut SqliteConnection,
    id: TagId,
    name: &str,
    description: Option<&str>,
    lang: &str,
) -> Result<(), DbError> {
    if tag_get_by_id(conn, id).await?.is_none() {
        return Err(DbError::Database(format!("标签 {} 不存在", id.0)));
    }

    sqlx::query("UPDATE tags SET description = ?1 WHERE id = ?2")
        .bind(description)
        .bind(id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let current = TAG_NAMES.resolve_display(conn, id.0, lang).await?;
    if current.as_deref() != Some(name) {
        TAG_NAMES
            .rename_display(conn, id.0, None, lang, name)
            .await?;
    }
    Ok(())
}

pub async fn tag_get_by_id(conn: &mut SqliteConnection, id: TagId) -> Result<Option<Tag>, DbError> {
    let row: Option<TagRow> =
        sqlx::query_as("SELECT id, description, is_system FROM tags WHERE id = ?1")
            .bind(id.0)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_tag()))
}

pub async fn tag_upsert_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    description: Option<&str>,
    lang: &str,
) -> Result<TagId, DbError> {
    if let Some(existing) = tag_get_by_name(conn, name).await? {
        sqlx::query("UPDATE tags SET description = ?1 WHERE id = ?2")
            .bind(description)
            .bind(existing.id.0)
            .execute(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(existing.id)
    } else {
        let tag = Tag {
            id: TagId(0),
            description: description.map(|s| s.to_string()),
            is_system: false,
        };
        let tag_id = tag_create(conn, &tag).await?;
        TAG_NAMES
            .insert(conn, tag_id.0, lang, name, false, true)
            .await?;
        Ok(tag_id)
    }
}

pub async fn tag_delete(conn: &mut SqliteConnection, name: &str) -> Result<(), DbError> {
    // Find tag via tag_names and check is_system before deleting
    let row: Option<(i64, i32)> = sqlx::query_as(
        "SELECT t.id, t.is_system FROM tags t JOIN tag_names tn ON tn.tag_id = t.id WHERE tn.name = ?1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    match row {
        Some((_tag_id, is_sys)) if is_sys != 0 => {
            return Err(DbError::Database("系统内置标签不可删除".to_string()));
        }
        None => {
            return Err(DbError::Database(format!("Tag not found: {}", name)));
        }
        Some((tag_id, _)) => {
            sqlx::query("DELETE FROM tags WHERE id = ?1")
                .bind(tag_id)
                .execute(conn)
                .await
                .map_err(|e| DbError::Database(e.to_string()))?;
        }
    }
    Ok(())
}

/// 取各交易所打标签的显示名（回退链：所选语言 → en → zh-CN → 插入序）。
///
/// 每个标签最多返回一个名字，不会因多语言名字而扇出；批量解析避免 N+1。
pub async fn tag_names_by_transactions(
    conn: &mut SqliteConnection,
    transaction_ids: &[TransactionId],
    lang: &str,
) -> Result<HashMap<TransactionId, Vec<String>>, DbError> {
    if transaction_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // 1. 取交易-标签关联
    let mut query = sqlx::QueryBuilder::new(
        "SELECT tt.transaction_id, tt.tag_id FROM transaction_tags tt WHERE tt.transaction_id IN (",
    );
    let mut separated = query.separated(", ");
    for id in transaction_ids {
        separated.push_bind(id.0);
    }
    query.push(") ORDER BY tt.transaction_id, tt.tag_id");

    let rows: Vec<(i64, i64)> = query
        .build_query_as()
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    // 2. 批量解析标签显示名（回退链）
    let tag_ids: Vec<i64> = rows
        .iter()
        .map(|(_, tag_id)| *tag_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let names = TAG_NAMES
        .batch_resolve_display(conn, &tag_ids, lang)
        .await?;

    // 3. 组装；没有任何名字的标签（异常数据）跳过
    let mut map: HashMap<TransactionId, Vec<String>> = HashMap::new();
    for (tx_id, tag_id) in rows {
        if let Some(name) = names.get(&tag_id) {
            map.entry(TransactionId(tx_id))
                .or_default()
                .push(name.clone());
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let mut conn = setup().await;
        let found = tag_get_by_name(&mut conn, "repayment").await.unwrap();
        assert!(found.is_some());
        let tag = found.unwrap();
        assert!(tag.is_system);
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let list = tag_list(&mut conn).await.unwrap();
        assert!(!list.is_empty());
    }

    #[tokio::test]
    async fn test_create() {
        let mut conn = setup().await;
        let tag = Tag {
            id: TagId(0),
            description: Some("旅行".to_string()),
            is_system: false,
        };
        let id = tag_create(&mut conn, &tag).await.unwrap();
        // Insert a tag_names entry so we can look up by name
        sqlx::query(
            "INSERT INTO tag_names (tag_id, lang, name, is_system, is_display) VALUES (?1, 'en', 'travel', 0, 1)",
        )
        .bind(id.0)
        .execute(&mut conn)
        .await
        .unwrap();
        let fetched = tag_get_by_name(&mut conn, "travel").await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.description, Some("旅行".to_string()));
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let tag = Tag {
            id: TagId(0),
            description: None,
            is_system: false,
        };
        let id = tag_create(&mut conn, &tag).await.unwrap();
        // Insert a tag_names entry so we can look up and delete by name
        sqlx::query(
            "INSERT INTO tag_names (tag_id, lang, name, is_system, is_display) VALUES (?1, 'en', 'temp', 0, 1)",
        )
        .bind(id.0)
        .execute(&mut conn)
        .await
        .unwrap();
        tag_delete(&mut conn, "temp").await.unwrap();
        assert!(tag_get_by_name(&mut conn, "temp").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_system_tag_rejected() {
        let mut conn = setup().await;
        // "repayment" is a system tag from seed data
        let result = tag_delete(&mut conn, "repayment").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("系统内置标签"));

        // Tag should still exist
        assert!(
            tag_get_by_name(&mut conn, "repayment")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_tag_update_renames() {
        let mut conn = setup().await;
        let id = tag_upsert_by_name(&mut conn, "travel", Some("旅行"), "en")
            .await
            .unwrap();

        tag_update(&mut conn, id, "journey", Some("旅途"), "en")
            .await
            .unwrap();

        assert!(
            tag_get_by_name(&mut conn, "journey")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            tag_get_by_name(&mut conn, "travel")
                .await
                .unwrap()
                .is_none()
        );
        let tag = tag_get_by_id(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(tag.description, Some("旅途".to_string()));

        // 不存在的标签显式报错
        assert!(
            tag_update(&mut conn, TagId(99999), "x", None, "en")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_tag_update_system_tag() {
        let mut conn = setup().await;
        let pending = tag_get_by_name(&mut conn, "pending")
            .await
            .unwrap()
            .unwrap();

        // 改系统标签的显示名 → 系统名降级为非显示（文本保留），用户名成为显示名
        tag_update(&mut conn, pending.id, "pending2", None, "en")
            .await
            .unwrap();
        // 系统名文本保留，任意名字仍可命中
        assert!(
            tag_get_by_name(&mut conn, "pending")
                .await
                .unwrap()
                .is_some()
        );
        // 新显示名生效
        assert!(
            tag_get_by_name(&mut conn, "pending2")
                .await
                .unwrap()
                .is_some()
        );

        // 名字未变化时允许仅更新描述
        tag_update(&mut conn, pending.id, "pending", Some("新描述"), "en")
            .await
            .unwrap();
        let tag = tag_get_by_id(&mut conn, pending.id).await.unwrap().unwrap();
        assert_eq!(tag.description, Some("新描述".to_string()));
    }

    #[tokio::test]
    async fn test_tag_upsert_by_name_marks_lang() {
        let mut conn = setup().await;
        // 导入场景：自动创建的名字标 'und'
        let id = tag_upsert_by_name(&mut conn, "临时标记", None, "und")
            .await
            .unwrap();
        let lang: String = sqlx::query_scalar(
            "SELECT lang FROM tag_names WHERE tag_id = ?1 AND name = '临时标记'",
        )
        .bind(id.0)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(lang, "und");

        // 再次 upsert 同名字（NOCASE）命中已有标签，不产生重复
        let again = tag_upsert_by_name(&mut conn, "临时标记", None, "und")
            .await
            .unwrap();
        assert_eq!(again, id);
    }

    #[tokio::test]
    async fn test_tag_names_by_transactions_no_fanout_with_fallback() {
        let mut conn = setup().await;

        // 标签 A：en + zh-CN 双语名字；标签 B：只有 zh-CN 名字
        let tag_a = tag_upsert_by_name(&mut conn, "food", None, "en")
            .await
            .unwrap();
        TAG_NAMES
            .insert(&mut conn, tag_a.0, "zh-CN", "餐饮", false, true)
            .await
            .unwrap();
        let tag_b = tag_upsert_by_name(&mut conn, "交通", None, "zh-CN")
            .await
            .unwrap();

        let member_id: i64 = sqlx::query_scalar("INSERT INTO members DEFAULT VALUES RETURNING id")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        let tx_id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-01-01 00:00:00', 'test', ?1) RETURNING id",
        )
        .bind(member_id)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        for tag_id in [tag_a.0, tag_b.0] {
            sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)")
                .bind(tx_id)
                .bind(tag_id)
                .execute(&mut conn)
                .await
                .unwrap();
        }
        let tx = TransactionId(tx_id);

        // 英文查询：A 取英文名；B 无英文名 → 回退中文名，不消失；每标签恰好一条，不扇出
        let map = tag_names_by_transactions(&mut conn, &[tx], "en")
            .await
            .unwrap();
        let names = map.get(&tx).unwrap();
        assert_eq!(names.len(), 2, "多语言名字不应扇出");
        assert!(names.contains(&"food".to_string()));
        assert!(names.contains(&"交通".to_string()));

        // 中文查询：均取中文名
        let map = tag_names_by_transactions(&mut conn, &[tx], "zh-CN")
            .await
            .unwrap();
        let names = map.get(&tx).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"餐饮".to_string()));
        assert!(names.contains(&"交通".to_string()));
    }
}
