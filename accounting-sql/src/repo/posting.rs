use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
use accounting::posting::Posting;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;

/// Posting 仓库 trait
pub trait PostingRepo {
    /// 插入分录，返回新分录 ID
    fn insert(
        &self,
        conn: &Connection,
        posting: &Posting,
    ) -> Result<PostingId, crate::error::DbError>;
    /// 列出某交易的所有分录
    fn list_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<Vec<Posting>, crate::error::DbError>;
    /// 列出某账户的所有分录
    fn list_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<Posting>, crate::error::DbError>;
    /// 按商品汇总某账户的余额
    fn sum_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError>;
    /// 删除某交易的所有分录
    fn delete_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<(), crate::error::DbError>;
}

/// SQLite PostingRepo 实现
pub struct SqlitePostingRepo;

impl PostingRepo for SqlitePostingRepo {
    fn insert(
        &self,
        conn: &Connection,
        posting: &Posting,
    ) -> Result<PostingId, crate::error::DbError> {
        let amount_i64 = accounting::amount::to_db_amount(posting.amount, 2);
        let cost_i64 = posting.cost.map(|c| accounting::amount::to_db_amount(c, 2));
        conn.execute(
            "INSERT INTO postings
             (transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                posting.transaction_id.0,
                posting.account_id.0,
                posting.commodity_id.0,
                amount_i64,
                cost_i64,
                posting.cost_commodity_id.map(|id| id.0),
                posting.description,
                posting.member_id.map(|id| id.0),
                posting.channel_id.map(|id| id.0),
            ],
        )?;
        Ok(PostingId(conn.last_insert_rowid()))
    }

    fn list_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<Vec<Posting>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE transaction_id = ?1"
        )?;
        let rows = stmt.query_map(params![transaction_id.0], map_posting)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn list_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<Posting>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE account_id = ?1 ORDER BY transaction_id"
        )?;
        let rows = stmt.query_map(params![account_id.0], map_posting)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn sum_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT commodity_id, SUM(amount) FROM postings WHERE account_id = ?1 GROUP BY commodity_id"
        )?;
        let rows = stmt.query_map(params![account_id.0], |row| {
            let commodity_id = CommodityId(row.get(0)?);
            let amount: i64 = row.get(1)?;
            let decimal = accounting::amount::from_db_amount(amount, 2);
            Ok((commodity_id, decimal))
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn delete_by_transaction(
        &self,
        conn: &Connection,
        transaction_id: TransactionId,
    ) -> Result<(), crate::error::DbError> {
        conn.execute(
            "DELETE FROM postings WHERE transaction_id = ?1",
            params![transaction_id.0],
        )?;
        Ok(())
    }
}

fn map_posting(row: &rusqlite::Row) -> Result<Posting, rusqlite::Error> {
    let amount: i64 = row.get(4)?;
    let cost: Option<i64> = row.get(5)?;
    Ok(Posting {
        id: PostingId(row.get(0)?),
        transaction_id: TransactionId(row.get(1)?),
        account_id: AccountId(row.get(2)?),
        commodity_id: CommodityId(row.get(3)?),
        amount: accounting::amount::from_db_amount(amount, 2),
        cost: cost.map(|c| accounting::amount::from_db_amount(c, 2)),
        cost_commodity_id: row.get::<_, Option<i64>>(6)?.map(CommodityId),
        description: row.get(7)?,
        member_id: row.get::<_, Option<i64>>(8)?.map(accounting::id::MemberId),
        channel_id: row.get::<_, Option<i64>>(9)?.map(accounting::id::ChannelId),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{AccountId, CommodityId};
    use rusqlite::Connection;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn setup() -> (Connection, SqlitePostingRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn).unwrap();
        (conn, SqlitePostingRepo)
    }

    fn insert_transaction(conn: &Connection) -> TransactionId {
        conn.execute(
            "INSERT INTO transactions (date, description, is_template) VALUES ('2024-01-01', 'test', 0)",
            [],
        )
        .unwrap();
        TransactionId(conn.last_insert_rowid())
    }

    fn insert_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, opened_at, is_system) VALUES (?1, 1, '2024-01-01', 0)",
            [name],
        )
        .unwrap();
        AccountId(conn.last_insert_rowid())
    }

    fn sample_posting(tx_id: TransactionId, account_id: AccountId, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id,
            commodity_id: CommodityId(1), // CNY seed commodity
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_insert_and_list_by_transaction() {
        let (conn, repo) = setup();
        let tx_id = insert_transaction(&conn);
        let a1 = insert_account(&conn, "Assets:A");
        let a2 = insert_account(&conn, "Assets:B");
        let p1 = sample_posting(tx_id, a1, "100.00");
        let p2 = sample_posting(tx_id, a2, "-100.00");
        repo.insert(&conn, &p1).unwrap();
        repo.insert(&conn, &p2).unwrap();

        let list = repo.list_by_transaction(&conn, tx_id).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_list_by_account() {
        let (conn, repo) = setup();
        let tx_id = insert_transaction(&conn);
        let a = insert_account(&conn, "Assets:C");
        let p = sample_posting(tx_id, a, "50.00");
        repo.insert(&conn, &p).unwrap();

        let list = repo.list_by_account(&conn, a).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].amount, Decimal::from_str("50.00").unwrap());
    }

    #[test]
    fn test_sum_by_account() {
        let (conn, repo) = setup();
        let tx_id = insert_transaction(&conn);
        let a = insert_account(&conn, "Assets:D");
        let p1 = sample_posting(tx_id, a, "100.00");
        let p2 = sample_posting(tx_id, a, "50.00");
        repo.insert(&conn, &p1).unwrap();
        repo.insert(&conn, &p2).unwrap();

        let sums = repo.sum_by_account(&conn, a).unwrap();
        assert_eq!(sums.len(), 1);
        assert_eq!(sums[0].0, CommodityId(1));
        assert_eq!(sums[0].1, Decimal::from_str("150.00").unwrap());
    }

    #[test]
    fn test_delete_by_transaction() {
        let (conn, repo) = setup();
        let tx_id = insert_transaction(&conn);
        let a = insert_account(&conn, "Assets:E");
        let p = sample_posting(tx_id, a, "10.00");
        repo.insert(&conn, &p).unwrap();
        repo.delete_by_transaction(&conn, tx_id).unwrap();
        let list = repo.list_by_transaction(&conn, tx_id).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_amount_roundtrip() {
        let (conn, repo) = setup();
        let tx_id = insert_transaction(&conn);
        let a = insert_account(&conn, "Assets:F");
        let mut p = sample_posting(tx_id, a, "123.45");
        p.cost = Some(Decimal::from_str("67.89").unwrap());
        p.cost_commodity_id = Some(CommodityId(1));
        repo.insert(&conn, &p).unwrap();

        let list = repo.list_by_transaction(&conn, tx_id).unwrap();
        assert_eq!(list[0].amount, Decimal::from_str("123.45").unwrap());
        assert_eq!(list[0].cost, Some(Decimal::from_str("67.89").unwrap()));
    }
}
