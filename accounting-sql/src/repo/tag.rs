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
            conn.prepare("SELECT id, name, description, is_system FROM tags ORDER BY name")?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteTagRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
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
}
