use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::id::MemberId;
use accounting::member::Member;

#[derive(FromRow)]
struct MemberRow {
    id: i64,
    name: String,
}

impl MemberRow {
    fn into_member(self) -> Member {
        Member {
            id: MemberId(self.id),
            name: self.name,
        }
    }
}

pub async fn member_create(
    conn: &mut SqliteConnection,
    member: &Member,
) -> Result<MemberId, DbError> {
    let id: i64 = sqlx::query_scalar("INSERT INTO members (name) VALUES (?1) RETURNING id")
        .bind(&member.name)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(MemberId(id))
}

pub async fn member_get(
    conn: &mut SqliteConnection,
    id: MemberId,
) -> Result<Option<Member>, DbError> {
    let row: Option<MemberRow> = sqlx::query_as("SELECT id, name FROM members WHERE id = ?1")
        .bind(id.0)
        .fetch_optional(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_member()))
}

pub async fn member_list(conn: &mut SqliteConnection) -> Result<Vec<Member>, DbError> {
    let rows: Vec<MemberRow> = sqlx::query_as("SELECT id, name FROM members ORDER BY id")
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_member()).collect())
}

pub async fn member_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Member>, DbError> {
    let row: Option<MemberRow> = sqlx::query_as("SELECT id, name FROM members WHERE name = ?1")
        .bind(name)
        .fetch_optional(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_member()))
}

pub async fn member_get_or_create_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<MemberId, DbError> {
    let row: Option<MemberRow> = sqlx::query_as("SELECT id, name FROM members WHERE name = ?1")
        .bind(name)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    if let Some(row) = row {
        return Ok(row.into_member().id);
    }

    let member = Member {
        id: MemberId(0),
        name: name.to_string(),
    };
    member_create(conn, &member).await
}

pub async fn member_rename(
    conn: &mut SqliteConnection,
    id: MemberId,
    new_name: &str,
) -> Result<(), DbError> {
    let result = sqlx::query("UPDATE members SET name = ?1 WHERE id = ?2")
        .bind(new_name)
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(DbError::Database(format!("成员 {} 不存在", id.0)));
    }
    Ok(())
}

pub async fn member_delete(conn: &mut SqliteConnection, id: MemberId) -> Result<(), DbError> {
    sqlx::query("DELETE FROM members WHERE id = ?1")
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
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let member = Member {
            id: MemberId(0),
            name: "Alice".to_string(),
        };
        let id = member_create(&mut conn, &member).await.unwrap();
        let fetched = member_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Alice");
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let member = Member {
            id: MemberId(0),
            name: "Bob".to_string(),
        };
        member_create(&mut conn, &member).await.unwrap();
        let list = member_list(&mut conn).await.unwrap();
        assert!(list.iter().any(|m| m.name == "Bob"));
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let member = Member {
            id: MemberId(0),
            name: "Charlie".to_string(),
        };
        let id = member_create(&mut conn, &member).await.unwrap();
        member_delete(&mut conn, id).await.unwrap();
        assert!(member_get(&mut conn, id).await.unwrap().is_none());
    }
}
