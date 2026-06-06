use accounting::id::TagId;
use accounting::tag::Tag;
use rusqlite::{Connection, params};

/// Tag 仓库 trait
pub trait TagRepo {
    /// 根据名称查询标签
    fn get_by_name(
        &self,
        conn: &Connection,
        name: &str,
    ) -> Result<Option<Tag>, crate::error::DbError>;
    /// 列出所有标签
    fn list(&self, conn: &Connection) -> Result<Vec<Tag>, crate::error::DbError>;
    /// 创建标签
    fn create(&self, conn: &Connection, tag: &Tag) -> Result<TagId, crate::error::DbError>;
    /// 删除标签
    fn delete(&self, conn: &Connection, name: &str) -> Result<(), crate::error::DbError>;
}

/// SQLite TagRepo 实现
pub struct SqliteTagRepo;

impl TagRepo for SqliteTagRepo {
    fn get_by_name(
        &self,
        conn: &Connection,
        name: &str,
    ) -> Result<Option<Tag>, crate::error::DbError> {
        let mut stmt =
            conn.prepare("SELECT id, name, description, is_system FROM tags WHERE name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Tag {
                id: TagId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Tag>, crate::error::DbError> {
        let mut stmt =
            conn.prepare("SELECT id, name, description, is_system FROM tags ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: TagId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                is_system: row.get::<_, i32>(3)? != 0,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn create(&self, conn: &Connection, tag: &Tag) -> Result<TagId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO tags (name, description, is_system) VALUES (?1, ?2, ?3)",
            params![tag.name, tag.description, tag.is_system as i32],
        )?;
        Ok(TagId(conn.last_insert_rowid()))
    }

    fn delete(&self, conn: &Connection, name: &str) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM tags WHERE name = ?1", params![name])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteTagRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn, "en").unwrap();
        (conn, SqliteTagRepo)
    }

    #[test]
    fn test_get_by_name() {
        let (conn, repo) = setup();
        let found = repo.get_by_name(&conn, "repayment").unwrap();
        assert!(found.is_some());
        let tag = found.unwrap();
        assert!(tag.is_system);
    }

    #[test]
    fn test_list() {
        let (conn, repo) = setup();
        let list = repo.list(&conn).unwrap();
        assert!(!list.is_empty());
        assert!(list.iter().any(|t| t.name == "repayment"));
    }

    #[test]
    fn test_create() {
        let (conn, repo) = setup();
        let tag = Tag {
            id: TagId(0),
            name: "travel".to_string(),
            description: Some("旅行".to_string()),
            is_system: false,
        };
        let id = repo.create(&conn, &tag).unwrap();
        let fetched = repo.get_by_name(&conn, "travel").unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.description, Some("旅行".to_string()));
    }

    #[test]
    fn test_delete() {
        let (conn, repo) = setup();
        let tag = Tag {
            id: TagId(0),
            name: "temp".to_string(),
            description: None,
            is_system: false,
        };
        repo.create(&conn, &tag).unwrap();
        repo.delete(&conn, "temp").unwrap();
        assert!(repo.get_by_name(&conn, "temp").unwrap().is_none());
    }
}
