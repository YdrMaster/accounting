use accounting::attachment::Attachment;
use accounting::id::{AttachmentId, TransactionId};
use rusqlite::{Connection, params};

/// Attachment 仓库 trait
pub trait AttachmentRepo {
    /// 创建附件，返回新附件 ID
    fn create(
        &self,
        conn: &Connection,
        attachment: &Attachment,
    ) -> Result<AttachmentId, crate::error::DbError>;
    /// 列出某交易的所有附件
    fn list_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<Vec<Attachment>, crate::error::DbError>;
    /// 删除附件
    fn delete(&self, conn: &Connection, id: AttachmentId) -> Result<(), crate::error::DbError>;
}

/// SQLite AttachmentRepo 实现
pub struct SqliteAttachmentRepo;

impl AttachmentRepo for SqliteAttachmentRepo {
    fn create(
        &self,
        conn: &Connection,
        attachment: &Attachment,
    ) -> Result<AttachmentId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO attachments (transaction_id, filename, data) VALUES (?1, ?2, ?3)",
            params![
                attachment.transaction_id.0,
                attachment.filename,
                attachment.data,
            ],
        )?;
        Ok(AttachmentId(conn.last_insert_rowid()))
    }

    fn list_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<Vec<Attachment>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, filename, data FROM attachments WHERE transaction_id = ?1",
        )?;
        let rows = stmt.query_map(params![transaction_id.0], |row| {
            Ok(Attachment {
                id: AttachmentId(row.get(0)?),
                transaction_id: TransactionId(row.get(1)?),
                filename: row.get(2)?,
                data: row.get(3)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn delete(&self, conn: &Connection, id: AttachmentId) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM attachments WHERE id = ?1", params![id.0])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteAttachmentRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
        (conn, SqliteAttachmentRepo)
    }

    fn insert_test_transaction(conn: &Connection) -> TransactionId {
        conn.execute(
            "INSERT INTO transactions (date, description, is_template) VALUES ('2024-01-01', 'test', 0)",
            [],
        )
        .unwrap();
        TransactionId(conn.last_insert_rowid())
    }

    #[test]
    fn test_create_and_list() {
        let (conn, repo) = setup();
        let tx_id = insert_test_transaction(&conn);
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "receipt.pdf".to_string(),
            data: vec![0x01, 0x02, 0x03],
        };
        let id = repo.create(&conn, &attachment).unwrap();
        let list = repo.list_by_transaction(&conn, tx_id).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].filename, "receipt.pdf");
    }

    #[test]
    fn test_delete() {
        let (conn, repo) = setup();
        let tx_id = insert_test_transaction(&conn);
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "doc.txt".to_string(),
            data: vec![0xAA],
        };
        let id = repo.create(&conn, &attachment).unwrap();
        repo.delete(&conn, id).unwrap();
        let list = repo.list_by_transaction(&conn, tx_id).unwrap();
        assert!(list.is_empty());
    }
}
