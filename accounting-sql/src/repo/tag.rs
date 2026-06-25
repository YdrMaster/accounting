use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::id::{TagId, TransactionId};
use accounting::tag::Tag;
use std::collections::HashMap;

#[derive(FromRow)]
struct TagRow {
    id: i64,
    name: String,
    description: Option<String>,
    is_system: i32,
}

impl TagRow {
    fn into_tag(self) -> Tag {
        Tag {
            id: TagId(self.id),
            name: self.name,
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
        sqlx::query_as("SELECT id, name, description, is_system FROM tags WHERE name = ?1")
            .bind(name)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_tag()))
}

pub async fn tag_list(conn: &mut SqliteConnection) -> Result<Vec<Tag>, DbError> {
    let rows: Vec<TagRow> =
        sqlx::query_as("SELECT id, name, description, is_system FROM tags ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_tag()).collect())
}

pub async fn tag_create(conn: &mut SqliteConnection, tag: &Tag) -> Result<TagId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO tags (name, description, is_system) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(&tag.name)
    .bind(&tag.description)
    .bind(tag.is_system as i32)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(TagId(id))
}

pub async fn tag_delete(conn: &mut SqliteConnection, name: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM tags WHERE name = ?1")
        .bind(name)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn tag_names_by_transactions(
    conn: &mut SqliteConnection,
    transaction_ids: &[TransactionId],
) -> Result<HashMap<TransactionId, Vec<String>>, DbError> {
    if transaction_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = sqlx::QueryBuilder::new(
        "SELECT tt.transaction_id, t.name FROM transaction_tags tt JOIN tags t ON tt.tag_id = t.id WHERE tt.transaction_id IN (",
    );
    let mut separated = query.separated(", ");
    for id in transaction_ids {
        separated.push_bind(id.0);
    }
    query.push(")");

    let rows: Vec<(i64, String)> = query
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let mut map: HashMap<TransactionId, Vec<String>> = HashMap::new();
    for (tx_id, name) in rows {
        map.entry(TransactionId(tx_id)).or_default().push(name);
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
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
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
        assert!(list.iter().any(|t| t.name == "repayment"));
    }

    #[tokio::test]
    async fn test_create() {
        let mut conn = setup().await;
        let tag = Tag {
            id: TagId(0),
            name: "travel".to_string(),
            description: Some("旅行".to_string()),
            is_system: false,
        };
        let id = tag_create(&mut conn, &tag).await.unwrap();
        let fetched = tag_get_by_name(&mut conn, "travel").await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.description, Some("旅行".to_string()));
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let tag = Tag {
            id: TagId(0),
            name: "temp".to_string(),
            description: None,
            is_system: false,
        };
        tag_create(&mut conn, &tag).await.unwrap();
        tag_delete(&mut conn, "temp").await.unwrap();
        assert!(tag_get_by_name(&mut conn, "temp").await.unwrap().is_none());
    }
}
