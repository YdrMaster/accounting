use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use crate::names::MEMBER_NAMES;
use accounting::id::MemberId;
use accounting::member::Member;

#[derive(FromRow)]
struct MemberRow {
    id: i64,
}

impl MemberRow {
    fn into_member(self) -> Member {
        Member {
            id: MemberId(self.id),
        }
    }
}

pub async fn member_create(
    conn: &mut SqliteConnection,
    _member: &Member,
) -> Result<MemberId, DbError> {
    let id: i64 = sqlx::query_scalar("INSERT INTO members DEFAULT VALUES RETURNING id")
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(MemberId(id))
}

pub async fn member_get(
    conn: &mut SqliteConnection,
    id: MemberId,
) -> Result<Option<Member>, DbError> {
    let row: Option<MemberRow> = sqlx::query_as("SELECT id FROM members WHERE id = ?1")
        .bind(id.0)
        .fetch_optional(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_member()))
}

pub async fn member_list(conn: &mut SqliteConnection) -> Result<Vec<Member>, DbError> {
    let rows: Vec<MemberRow> = sqlx::query_as("SELECT id FROM members ORDER BY id")
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_member()).collect())
}

pub async fn member_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Member>, DbError> {
    let row: Option<MemberRow> = sqlx::query_as(
        "SELECT m.id FROM members m JOIN member_names mn ON mn.member_id = m.id WHERE mn.name = ?1",
    )
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_member()))
}

pub async fn member_get_or_create_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
) -> Result<MemberId, DbError> {
    let existing_id: Option<MemberId> = sqlx::query_scalar(
        "SELECT m.id FROM members m JOIN member_names mn ON mn.member_id = m.id WHERE mn.name = ?1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?
    .map(MemberId);

    if let Some(id) = existing_id {
        return Ok(id);
    }

    let member = Member { id: MemberId(0) };
    let id = member_create(conn, &member).await?;

    MEMBER_NAMES
        .insert(conn, id.0, lang, name, false, true)
        .await?;

    Ok(id)
}

/// 改名：按 (member_id, lang) 定位该语言的显示名单条记录改文本；
/// 该语言无显示名时新增显示名，其余语言名字不受影响。
pub async fn member_rename(
    conn: &mut SqliteConnection,
    id: MemberId,
    new_name: &str,
    lang: &str,
) -> Result<(), DbError> {
    if member_get(conn, id).await?.is_none() {
        return Err(DbError::Database(format!("成员 {} 不存在", id.0)));
    }
    MEMBER_NAMES
        .rename_display(conn, id.0, None, lang, new_name)
        .await
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
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let member = Member { id: MemberId(0) };
        let id = member_create(&mut conn, &member).await.unwrap();
        let fetched = member_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let member = Member { id: MemberId(0) };
        let id = member_create(&mut conn, &member).await.unwrap();
        let list = member_list(&mut conn).await.unwrap();
        assert!(list.iter().any(|m| m.id == id));
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let member = Member { id: MemberId(0) };
        let id = member_create(&mut conn, &member).await.unwrap();
        member_delete(&mut conn, id).await.unwrap();
        assert!(member_get(&mut conn, id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_or_create_by_name_marks_lang() {
        let mut conn = setup().await;
        // 用户场景：名字标注调用方语言；导入场景传 'und'
        let id = member_get_or_create_by_name(&mut conn, "小明", "zh-CN")
            .await
            .unwrap();
        let lang: String = sqlx::query_scalar(
            "SELECT lang FROM member_names WHERE member_id = ?1 AND name = '小明'",
        )
        .bind(id.0)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(lang, "zh-CN");

        // 重复调用命中同一成员
        let again = member_get_or_create_by_name(&mut conn, "小明", "und")
            .await
            .unwrap();
        assert_eq!(again, id);
    }

    #[tokio::test]
    async fn test_member_rename_by_lang() {
        let mut conn = setup().await;
        let id = member_get_or_create_by_name(&mut conn, "Alice", "en")
            .await
            .unwrap();

        // 英文改名：定位 (member, en) 单条更新
        member_rename(&mut conn, id, "Alicia", "en").await.unwrap();
        assert!(
            member_get_by_name(&mut conn, "Alicia")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            member_get_by_name(&mut conn, "Alice")
                .await
                .unwrap()
                .is_none()
        );

        // 中文改名：该语言无显示名 → 新增显示名，英文名保持不动
        member_rename(&mut conn, id, "爱丽丝", "zh-CN")
            .await
            .unwrap();
        assert!(
            member_get_by_name(&mut conn, "爱丽丝")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            member_get_by_name(&mut conn, "Alicia")
                .await
                .unwrap()
                .is_some(),
            "其余语言显示名不应被抹掉"
        );

        // 不存在的成员显式报错
        assert!(
            member_rename(&mut conn, MemberId(99999), "X", "en")
                .await
                .is_err()
        );
    }
}
