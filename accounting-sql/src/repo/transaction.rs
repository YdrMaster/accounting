use accounting::id::{TagId, TransactionId};
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use chrono::NaiveDate;
use rusqlite::{Connection, params};

/// Transaction 仓库 trait
pub trait TransactionRepo {
    /// 插入交易及标签关联，返回交易 ID
    fn insert(
        &self,
        conn: &Connection,
        tx: &Transaction,
        tag_ids: &[TagId],
    ) -> Result<TransactionId, crate::error::DbError>;
    /// 根据 ID 查询交易
    fn get(
        &self,
        conn: &Connection,
        id: TransactionId,
    ) -> Result<Option<Transaction>, crate::error::DbError>;
    /// 多条件筛选查询交易列表
    fn list(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Transaction>, crate::error::DbError>;
    /// 多条件筛选统计交易数量
    fn count(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
    ) -> Result<usize, crate::error::DbError>;
    /// 删除交易（级联删除分录、附件、标签关联）
    fn delete(&self, conn: &Connection, id: TransactionId) -> Result<(), crate::error::DbError>;
    /// 更新交易及标签关联
    fn update(
        &self,
        conn: &Connection,
        tx: &Transaction,
        tag_ids: &[TagId],
    ) -> Result<(), crate::error::DbError>;
}

/// SQLite TransactionRepo 实现
pub struct SqliteTransactionRepo;

impl TransactionRepo for SqliteTransactionRepo {
    fn insert(
        &self,
        conn: &Connection,
        tx: &Transaction,
        tag_ids: &[TagId],
    ) -> Result<TransactionId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO transactions (date, description, member_id, is_template)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                tx.date.to_string(),
                tx.description,
                tx.member_id.map(|id| id.0),
                tx.is_template as i32,
            ],
        )?;
        let tx_id = TransactionId(conn.last_insert_rowid());
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
                params![tx_id.0, tag_id.0],
            )?;
        }
        Ok(tx_id)
    }

    fn get(
        &self,
        conn: &Connection,
        id: TransactionId,
    ) -> Result<Option<Transaction>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, date, description, member_id, is_template FROM transactions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    fn list(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Transaction>, crate::error::DbError> {
        let mut joins = Vec::new();
        let mut conditions = vec!["1=1"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("transactions.date >= ?");
            params_vec.push(Box::new(start.to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("transactions.date <= ?");
            params_vec.push(Box::new(end.to_string()));
        }
        if let Some(member) = filter.member_id {
            conditions.push("transactions.member_id = ?");
            params_vec.push(Box::new(member.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("transactions.description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("transactions.is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }
        if let Some(account) = filter.account_id {
            joins.push("JOIN postings p ON p.transaction_id = transactions.id");
            conditions.push("p.account_id = ?");
            params_vec.push(Box::new(account.0));
        }
        if let Some(tag) = filter.tag_id {
            joins.push("JOIN transaction_tags tt ON tt.transaction_id = transactions.id");
            conditions.push("tt.tag_id = ?");
            params_vec.push(Box::new(tag.0));
        }
        if let Some(channel) = filter.channel_id {
            if !joins.iter().any(|j| j.contains("postings")) {
                joins.push("JOIN postings p ON p.transaction_id = transactions.id");
            }
            conditions.push("p.channel_id = ?");
            params_vec.push(Box::new(channel.0));
        }

        let join_clause = joins.join(" ");
        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT DISTINCT transactions.id, transactions.date, transactions.description, transactions.member_id, transactions.is_template
             FROM transactions {}
             WHERE {}
             ORDER BY transactions.date DESC, transactions.id DESC
             LIMIT ? OFFSET ?",
            join_clause,
            where_clause
        );
        params_vec.push(Box::new(limit as i64));
        params_vec.push(Box::new(offset as i64));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), map_transaction)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn count(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
    ) -> Result<usize, crate::error::DbError> {
        let mut joins = Vec::new();
        let mut conditions = vec!["1=1"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("transactions.date >= ?");
            params_vec.push(Box::new(start.to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("transactions.date <= ?");
            params_vec.push(Box::new(end.to_string()));
        }
        if let Some(member) = filter.member_id {
            conditions.push("transactions.member_id = ?");
            params_vec.push(Box::new(member.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("transactions.description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("transactions.is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }
        if let Some(account) = filter.account_id {
            joins.push("JOIN postings p ON p.transaction_id = transactions.id");
            conditions.push("p.account_id = ?");
            params_vec.push(Box::new(account.0));
        }
        if let Some(tag) = filter.tag_id {
            joins.push("JOIN transaction_tags tt ON tt.transaction_id = transactions.id");
            conditions.push("tt.tag_id = ?");
            params_vec.push(Box::new(tag.0));
        }
        if let Some(channel) = filter.channel_id {
            if !joins.iter().any(|j| j.contains("postings")) {
                joins.push("JOIN postings p ON p.transaction_id = transactions.id");
            }
            conditions.push("p.channel_id = ?");
            params_vec.push(Box::new(channel.0));
        }

        let join_clause = joins.join(" ");
        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT COUNT(DISTINCT transactions.id) FROM transactions {} WHERE {}",
            join_clause, where_clause
        );
        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, rusqlite::params_from_iter(param_refs), |row| {
            row.get(0)
        })?;
        Ok(count as usize)
    }

    fn delete(&self, conn: &Connection, id: TransactionId) -> Result<(), crate::error::DbError> {
        conn.execute("DELETE FROM transactions WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    fn update(
        &self,
        conn: &Connection,
        tx: &Transaction,
        tag_ids: &[TagId],
    ) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE transactions
             SET date = ?1, description = ?2, member_id = ?3, is_template = ?4
             WHERE id = ?5",
            params![
                tx.date.to_string(),
                tx.description,
                tx.member_id.map(|id| id.0),
                tx.is_template as i32,
                tx.id.0,
            ],
        )?;
        conn.execute(
            "DELETE FROM transaction_tags WHERE transaction_id = ?1",
            params![tx.id.0],
        )?;
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
                params![tx.id.0, tag_id.0],
            )?;
        }
        Ok(())
    }
}

fn map_transaction(row: &rusqlite::Row) -> Result<Transaction, rusqlite::Error> {
    let date_str: String = row.get(1)?;
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_default();

    Ok(Transaction {
        id: TransactionId(row.get(0)?),
        date,
        description: row.get(2)?,
        member_id: row.get::<_, Option<i64>>(3)?.map(accounting::id::MemberId),
        is_template: row.get::<_, i32>(4)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::MemberId;
    use chrono::NaiveDate;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteTransactionRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
        (conn, SqliteTransactionRepo)
    }

    fn sample_tx() -> Transaction {
        Transaction {
            id: TransactionId(0),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            description: "Grocery shopping".to_string(),
            member_id: None,
            is_template: false,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let (conn, repo) = setup();
        let tx = sample_tx();
        let id = repo.insert(&conn, &tx, &[]).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.description, "Grocery shopping");
        assert_eq!(fetched.date, tx.date);
    }

    #[test]
    fn test_insert_with_tags() {
        let (conn, repo) = setup();
        let tx = sample_tx();
        let tag_id = TagId(1); // repayment seed tag
        let id = repo.insert(&conn, &tx, &[tag_id]).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transaction_tags WHERE transaction_id = ?1",
                params![id.0],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete() {
        let (conn, repo) = setup();
        let tx = sample_tx();
        let id = repo.insert(&conn, &tx, &[]).unwrap();
        repo.delete(&conn, id).unwrap();
        assert!(repo.get(&conn, id).unwrap().is_none());
    }

    #[test]
    fn test_update() {
        let (conn, repo) = setup();
        let tx = sample_tx();
        let id = repo.insert(&conn, &tx, &[]).unwrap();
        let mut updated = tx.clone();
        updated.id = id;
        updated.description = "Updated desc".to_string();
        repo.update(&conn, &updated, &[]).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.description, "Updated desc");
    }

    #[test]
    fn test_list_filter_by_date() {
        let (conn, repo) = setup();
        let mut tx1 = sample_tx();
        tx1.date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut tx2 = sample_tx();
        tx2.date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        repo.insert(&conn, &tx1, &[]).unwrap();
        repo.insert(&conn, &tx2, &[]).unwrap();

        let filter = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..Default::default()
        };
        let list = repo.list(&conn, &filter, 10, 0).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].date, tx2.date);
    }

    #[test]
    fn test_list_filter_by_keyword() {
        let (conn, repo) = setup();
        let mut tx1 = sample_tx();
        tx1.description = "Buy coffee".to_string();
        let mut tx2 = sample_tx();
        tx2.description = "Pay rent".to_string();
        repo.insert(&conn, &tx1, &[]).unwrap();
        repo.insert(&conn, &tx2, &[]).unwrap();

        let filter = TransactionFilter {
            keyword: Some("rent".to_string()),
            ..Default::default()
        };
        let list = repo.list(&conn, &filter, 10, 0).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].description, "Pay rent");
    }

    #[test]
    fn test_count() {
        let (conn, repo) = setup();
        let tx = sample_tx();
        repo.insert(&conn, &tx, &[]).unwrap();
        repo.insert(&conn, &tx, &[]).unwrap();

        let filter = TransactionFilter::default();
        let count = repo.count(&conn, &filter).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_list_filter_by_member() {
        let (conn, repo) = setup();
        conn.execute("INSERT INTO members (name) VALUES ('Alice')", [])
            .unwrap();
        let member_id = MemberId(conn.last_insert_rowid());
        let mut tx = sample_tx();
        tx.member_id = Some(member_id);
        repo.insert(&conn, &tx, &[]).unwrap();

        let filter = TransactionFilter {
            member_id: Some(member_id),
            ..Default::default()
        };
        let list = repo.list(&conn, &filter, 10, 0).unwrap();
        assert_eq!(list.len(), 1);
    }
}
