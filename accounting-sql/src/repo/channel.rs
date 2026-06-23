use accounting::channel::Channel;
use accounting::id::ChannelId;
use rusqlite::{Connection, params};

/// Channel 仓库 trait
pub trait ChannelRepo {
    /// 创建渠道，返回新渠道 ID
    fn create(
        &self,
        conn: &Connection,
        channel: &Channel,
    ) -> Result<ChannelId, crate::error::DbError>;
    /// 根据 ID 查询渠道
    fn get(
        &self,
        conn: &Connection,
        id: ChannelId,
    ) -> Result<Option<Channel>, crate::error::DbError>;
    /// 列出所有渠道
    fn list(&self, conn: &Connection) -> Result<Vec<Channel>, crate::error::DbError>;
}

/// SQLite ChannelRepo 实现
#[derive(Clone)]
pub struct SqliteChannelRepo;

impl ChannelRepo for SqliteChannelRepo {
    fn create(
        &self,
        conn: &Connection,
        channel: &Channel,
    ) -> Result<ChannelId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO channels (name, description) VALUES (?1, ?2)",
            params![channel.name, channel.description],
        )?;
        Ok(ChannelId(conn.last_insert_rowid()))
    }

    fn get(
        &self,
        conn: &Connection,
        id: ChannelId,
    ) -> Result<Option<Channel>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM channels WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Channel {
                id: ChannelId(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Channel>, crate::error::DbError> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM channels ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Channel {
                id: ChannelId(row.get(0)?),
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

    fn setup() -> (Connection, SqliteChannelRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn, "en").unwrap();
        (conn, SqliteChannelRepo)
    }

    #[test]
    fn test_create_and_get() {
        let (conn, repo) = setup();
        let channel = Channel {
            id: ChannelId(0),
            name: "Alipay".to_string(),
            description: Some("支付宝".to_string()),
        };
        let id = repo.create(&conn, &channel).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.name, "Alipay");
    }

    #[test]
    fn test_list() {
        let (conn, repo) = setup();
        let channel = Channel {
            id: ChannelId(0),
            name: "WeChat".to_string(),
            description: None,
        };
        repo.create(&conn, &channel).unwrap();
        let list = repo.list(&conn).unwrap();
        assert!(list.iter().any(|c| c.name == "WeChat"));
    }
}
