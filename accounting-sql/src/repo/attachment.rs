use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::attachment::Attachment;
use accounting::id::{AttachmentId, TransactionId};

#[derive(FromRow)]
struct AttachmentRow {
    id: i64,
    transaction_id: i64,
    filename: String,
    data: Vec<u8>,
}

impl AttachmentRow {
    fn into_attachment(self) -> Attachment {
        Attachment {
            id: AttachmentId(self.id),
            transaction_id: TransactionId(self.transaction_id),
            filename: self.filename,
            data: self.data,
        }
    }
}

pub async fn attachment_create(
    conn: &mut SqliteConnection,
    attachment: &Attachment,
) -> Result<AttachmentId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO attachments (transaction_id, filename, data) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(attachment.transaction_id.0)
    .bind(&attachment.filename)
    .bind(&attachment.data)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(AttachmentId(id))
}

pub async fn attachment_list_by_transaction(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
) -> Result<Vec<Attachment>, DbError> {
    let rows: Vec<AttachmentRow> = sqlx::query_as(
        "SELECT id, transaction_id, filename, data FROM attachments WHERE transaction_id = ?1",
    )
    .bind(transaction_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_attachment()).collect())
}

pub async fn attachment_delete(
    conn: &mut SqliteConnection,
    id: AttachmentId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM attachments WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
async fn insert_test_transaction(conn: &mut SqliteConnection) -> TransactionId {
    let member_id: i64 = sqlx::query_scalar("INSERT INTO members DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    sqlx::query("INSERT INTO member_names (member_id, lang, name, is_system, is_display) VALUES (?1, 'en', 'Test Member', 0, 1)")
        .bind(member_id)
        .execute(&mut *conn)
        .await
        .unwrap();
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-01-01 00:00:00', 'test', ?1) RETURNING id",
    )
    .bind(member_id)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    TransactionId(id)
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
    async fn test_create_and_list() {
        let mut conn = setup().await;
        let tx_id = insert_test_transaction(&mut conn).await;
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "receipt.pdf".to_string(),
            data: vec![0x01, 0x02, 0x03],
        };
        let id = attachment_create(&mut conn, &attachment).await.unwrap();
        let list = attachment_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].filename, "receipt.pdf");
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let tx_id = insert_test_transaction(&mut conn).await;
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "doc.txt".to_string(),
            data: vec![0xAA],
        };
        let id = attachment_create(&mut conn, &attachment).await.unwrap();
        attachment_delete(&mut conn, id).await.unwrap();
        let list = attachment_list_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        assert!(list.is_empty());
    }
}
