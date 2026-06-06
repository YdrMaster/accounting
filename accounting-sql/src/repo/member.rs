use accounting::id::MemberId;
use accounting::member::Member;
use rusqlite::{Connection, params};

/// Member 仓库 trait
pub trait MemberRepo {
    /// 创建成员，返回新成员 ID
    fn create(&self, conn: &Connection, member: &Member)
    -> Result<MemberId, crate::error::DbError>;
    /// 根据 ID 查询成员
    fn get(&self, conn: &Connection, id: MemberId)
    -> Result<Option<Member>, crate::error::DbError>;
    /// 列出所有成员
    fn list(&self, conn: &Connection) -> Result<Vec<Member>, crate::error::DbError>;
}

/// SQLite MemberRepo 实现
pub struct SqliteMemberRepo;

impl MemberRepo for SqliteMemberRepo {
    fn create(
        &self,
        conn: &Connection,
        member: &Member,
    ) -> Result<MemberId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO members (name, description) VALUES (?1, ?2)",
            params![member.name, member.description],
        )?;
        Ok(MemberId(conn.last_insert_rowid()))
    }

    fn get(
        &self,
        conn: &Connection,
        id: MemberId,
    ) -> Result<Option<Member>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM members WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Member {
                id: MemberId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Member>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM members ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Member {
                id: MemberId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteMemberRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
        (conn, SqliteMemberRepo)
    }

    #[test]
    fn test_create_and_get() {
        let (conn, repo) = setup();
        let member = Member {
            id: MemberId(0),
            name: "Alice".to_string(),
            description: Some("Tester".to_string()),
        };
        let id = repo.create(&conn, &member).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.name, "Alice");
        assert_eq!(fetched.description, Some("Tester".to_string()));
    }

    #[test]
    fn test_list() {
        let (conn, repo) = setup();
        let member = Member {
            id: MemberId(0),
            name: "Bob".to_string(),
            description: None,
        };
        repo.create(&conn, &member).unwrap();
        let list = repo.list(&conn).unwrap();
        assert!(list.iter().any(|m| m.name == "Bob"));
    }
}
