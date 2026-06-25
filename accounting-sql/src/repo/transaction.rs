use chrono::NaiveDateTime;
use sqlx::{QueryBuilder, SqliteConnection};

use crate::error::DbError;
use accounting::datetime_utils;
use accounting::id::{TagId, TransactionId};
use accounting::transaction::Transaction;
use accounting::transaction::TransactionKind;
use accounting::transaction_filter::TransactionFilter;

pub async fn transaction_insert(
    conn: &mut SqliteConnection,
    tx: &Transaction,
    tag_ids: &[TagId],
) -> Result<TransactionId, DbError> {
    let tx_id: i64 = sqlx::query_scalar(
        "INSERT INTO transactions (date_time, description, member_id, kind)
         VALUES (?1, ?2, ?3, ?4) RETURNING id",
    )
    .bind(tx.date_time.to_string())
    .bind(&tx.description)
    .bind(tx.member_id.map(|id| id.0))
    .bind(tx.kind as i32)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    let tx_id = TransactionId(tx_id);
    for tag_id in tag_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
        )
        .bind(tx_id.0)
        .bind(tag_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }
    Ok(tx_id)
}

pub async fn transaction_get(
    conn: &mut SqliteConnection,
    id: TransactionId,
) -> Result<Option<Transaction>, DbError> {
    let row: Option<TransactionRow> = sqlx::query_as(
        "SELECT id, date_time, description, kind, member_id FROM transactions WHERE id = ?1",
    )
    .bind(id.0)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn transaction_list(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
    limit: usize,
    offset: usize,
) -> Result<Vec<Transaction>, DbError> {
    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT DISTINCT transactions.id, transactions.date_time, transactions.description, transactions.kind, transactions.member_id FROM transactions ",
    );

    // 可报销过滤需要 JOIN postings 表
    if filter.has_reimbursable == Some(true) {
        builder.push("JOIN postings p_reimb ON p_reimb.transaction_id = transactions.id ");
    }

    // 渠道过滤需要 JOIN channel_paths 表
    if !filter.channel_ids.is_empty() {
        builder.push("JOIN channel_paths cp_filter ON cp_filter.transaction_id = transactions.id ");
    }

    builder.push("WHERE 1=1 ");

    apply_transaction_filter(&mut builder, filter);

    builder.push("ORDER BY transactions.date_time DESC, transactions.id DESC LIMIT ");
    builder.push_bind(limit as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset as i64);

    let rows: Vec<TransactionRow> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    rows.into_iter()
        .map(|r| r.try_into())
        .collect::<Result<_, _>>()
}

pub async fn transaction_count(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
) -> Result<usize, DbError> {
    let mut builder: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new("SELECT COUNT(DISTINCT transactions.id) FROM transactions ");

    if filter.has_reimbursable == Some(true) {
        builder.push("JOIN postings p_reimb ON p_reimb.transaction_id = transactions.id ");
    }

    if !filter.channel_ids.is_empty() {
        builder.push("JOIN channel_paths cp_filter ON cp_filter.transaction_id = transactions.id ");
    }

    builder.push("WHERE 1=1 ");

    apply_transaction_filter(&mut builder, filter);

    let count: i64 = builder
        .build_query_scalar()
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(count as usize)
}

fn apply_transaction_filter(builder: &mut QueryBuilder<sqlx::Sqlite>, filter: &TransactionFilter) {
    if let Some(start) = filter.start_date {
        builder.push("AND transactions.date_time >= ");
        builder.push_bind(datetime_utils::start_of_day(start).to_string());
        builder.push(" ");
    }
    if let Some(end) = filter.end_date {
        builder.push("AND transactions.date_time <= ");
        builder.push_bind(datetime_utils::end_of_day(end).to_string());
        builder.push(" ");
    }
    if !filter.member_ids.is_empty() {
        builder.push("AND transactions.member_id IN (");
        let mut separated = builder.separated(", ");
        for member in &filter.member_ids {
            separated.push_bind(member.0);
        }
        builder.push(") ");
    }
    if let Some(ref keyword) = filter.keyword {
        builder.push("AND transactions.description LIKE ");
        builder.push_bind(format!("%{}%", keyword));
        builder.push(" ");
    }
    // 账户过滤使用 EXISTS 子查询，避免 JOIN 导致行膨胀
    if !filter.account_ids.is_empty() {
        builder.push(
            "AND EXISTS (SELECT 1 FROM postings p WHERE p.transaction_id = transactions.id AND p.account_id IN (",
        );
        let mut separated = builder.separated(", ");
        for account in &filter.account_ids {
            separated.push_bind(account.0);
        }
        builder.push(")) ");
    }
    // 标签过滤使用 EXISTS 子查询，避免 JOIN 导致行膨胀
    if !filter.tag_ids.is_empty() {
        builder.push(
            "AND EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = transactions.id AND tt.tag_id IN (",
        );
        let mut separated = builder.separated(", ");
        for tag in &filter.tag_ids {
            separated.push_bind(tag.0);
        }
        builder.push(")) ");
    }
    // 渠道过滤通过 channel_paths JOIN 实现（语义：链路中包含指定渠道的交易）
    if !filter.channel_ids.is_empty() {
        builder.push("AND cp_filter.channel_id IN (");
        let mut separated = builder.separated(", ");
        for channel in &filter.channel_ids {
            separated.push_bind(channel.0);
        }
        builder.push(") ");
    }
    if filter.has_reimbursable == Some(true) {
        builder.push("AND p_reimb.is_reimbursable = 1 ");
    }
}

pub async fn transaction_delete(
    conn: &mut SqliteConnection,
    id: TransactionId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM transactions WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn transaction_update(
    conn: &mut SqliteConnection,
    tx: &Transaction,
    tag_ids: &[TagId],
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE transactions
         SET date_time = ?1, description = ?2, member_id = ?3, kind = ?4
         WHERE id = ?5",
    )
    .bind(tx.date_time.to_string())
    .bind(&tx.description)
    .bind(tx.member_id.map(|id| id.0))
    .bind(tx.kind as i32)
    .bind(tx.id.0)
    .execute(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM transaction_tags WHERE transaction_id = ?1")
        .bind(tx.id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    for tag_id in tag_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
        )
        .bind(tx.id.0)
        .bind(tag_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct TransactionRow {
    id: i64,
    date_time: String,
    description: String,
    kind: i32,
    member_id: Option<i64>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = DbError;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        let date_time = NaiveDateTime::parse_from_str(&row.date_time, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| DbError::Database(e.to_string()))?;

        Ok(Transaction {
            id: TransactionId(row.id),
            date_time,
            description: row.description,
            kind: TransactionKind::from_db(row.kind).unwrap_or(TransactionKind::Normal),
            member_id: row.member_id.map(accounting::id::MemberId),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::MemberId;
    use chrono::NaiveDate;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
        conn
    }

    fn sample_tx() -> Transaction {
        Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Grocery shopping".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        }
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let mut conn = setup().await;
        let tx = sample_tx();
        let id = transaction_insert(&mut conn, &tx, &[]).await.unwrap();
        let fetched = transaction_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.description, "Grocery shopping");
        assert_eq!(fetched.date_time, tx.date_time);
    }

    #[tokio::test]
    async fn test_insert_with_tags() {
        let mut conn = setup().await;
        let tx = sample_tx();
        let tag_id = TagId(1); // repayment seed tag
        let id = transaction_insert(&mut conn, &tx, &[tag_id]).await.unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM transaction_tags WHERE transaction_id = ?1")
                .bind(id.0)
                .fetch_one(&mut conn)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = setup().await;
        let tx = sample_tx();
        let id = transaction_insert(&mut conn, &tx, &[]).await.unwrap();
        transaction_delete(&mut conn, id).await.unwrap();
        assert!(transaction_get(&mut conn, id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_update() {
        let mut conn = setup().await;
        let tx = sample_tx();
        let id = transaction_insert(&mut conn, &tx, &[]).await.unwrap();
        let mut updated = tx.clone();
        updated.id = id;
        updated.description = "Updated desc".to_string();
        transaction_update(&mut conn, &updated, &[]).await.unwrap();
        let fetched = transaction_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.description, "Updated desc");
    }

    #[tokio::test]
    async fn test_list_filter_by_date() {
        let mut conn = setup().await;
        let mut tx1 = sample_tx();
        tx1.date_time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let mut tx2 = sample_tx();
        tx2.date_time = NaiveDate::from_ymd_opt(2024, 12, 31)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        transaction_insert(&mut conn, &tx1, &[]).await.unwrap();
        transaction_insert(&mut conn, &tx2, &[]).await.unwrap();

        let filter = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..Default::default()
        };
        let list = transaction_list(&mut conn, &filter, 10, 0).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].date_time, tx2.date_time);
    }

    #[tokio::test]
    async fn test_list_filter_by_keyword() {
        let mut conn = setup().await;
        let mut tx1 = sample_tx();
        tx1.description = "Buy coffee".to_string();
        let mut tx2 = sample_tx();
        tx2.description = "Pay rent".to_string();
        transaction_insert(&mut conn, &tx1, &[]).await.unwrap();
        transaction_insert(&mut conn, &tx2, &[]).await.unwrap();

        let filter = TransactionFilter {
            keyword: Some("rent".to_string()),
            ..Default::default()
        };
        let list = transaction_list(&mut conn, &filter, 10, 0).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].description, "Pay rent");
    }

    #[tokio::test]
    async fn test_count() {
        let mut conn = setup().await;
        let tx = sample_tx();
        transaction_insert(&mut conn, &tx, &[]).await.unwrap();
        transaction_insert(&mut conn, &tx, &[]).await.unwrap();

        let filter = TransactionFilter::default();
        let count = transaction_count(&mut conn, &filter).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_list_filter_by_member() {
        let mut conn = setup().await;
        let member_id: i64 =
            sqlx::query_scalar("INSERT INTO members (name) VALUES ('Alice') RETURNING id")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        let member_id = MemberId(member_id);
        let mut tx = sample_tx();
        tx.member_id = Some(member_id);
        transaction_insert(&mut conn, &tx, &[]).await.unwrap();

        let filter = TransactionFilter {
            member_ids: vec![member_id],
            ..Default::default()
        };
        let list = transaction_list(&mut conn, &filter, 10, 0).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_list_filter_by_channel() {
        let mut conn = setup().await;

        let channel_id: i64 =
            sqlx::query_scalar("INSERT INTO channels (name) VALUES ('Alipay') RETURNING id")
                .fetch_one(&mut conn)
                .await
                .unwrap();

        let tx = sample_tx();
        let tx_id = transaction_insert(&mut conn, &tx, &[]).await.unwrap();

        // Add a channel_path for this transaction
        sqlx::query(
            "INSERT INTO channel_paths (transaction_id, position, channel_id) VALUES (?1, 0, ?2)",
        )
        .bind(tx_id.0)
        .bind(channel_id)
        .execute(&mut conn)
        .await
        .unwrap();

        let filter = TransactionFilter {
            channel_ids: vec![accounting::id::ChannelId(channel_id)],
            ..Default::default()
        };
        let list = transaction_list(&mut conn, &filter, 10, 0).await.unwrap();
        assert_eq!(list.len(), 1);
    }
}
