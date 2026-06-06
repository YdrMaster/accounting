use accounting::id::{
    AccountId, ChannelId, CommodityId, MemberId, PostingId, TagId, TransactionId,
};
use accounting::posting::Posting;
use accounting::transaction_filter::TransactionFilter;
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
    /// 通过闭包表聚合查询某账户及其所有后代的余额
    fn sum_with_ancestors(
        &self,
        conn: &Connection,
        ancestor_id: AccountId,
    ) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError>;
    /// 按标签汇总分录金额（支持 TransactionFilter 过滤）
    ///
    /// 返回 `(TagId, CommodityId, account_type, Decimal)` 列表，
    /// 其中 `account_type` 为 4(Income) 或 5(Expense)，用于区分收入/支出方向。
    fn sum_by_tag(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(TagId, CommodityId, i64, Decimal)>, crate::error::DbError>;
    /// 按成员汇总分录金额（支持 TransactionFilter 过滤）
    fn sum_by_member(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(MemberId, CommodityId, i64, Decimal)>, crate::error::DbError>;
    /// 按渠道汇总分录金额（支持 TransactionFilter 过滤）
    fn sum_by_channel(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(ChannelId, CommodityId, i64, Decimal)>, crate::error::DbError>;
}

/// SQLite PostingRepo 实现
pub struct SqlitePostingRepo;

impl PostingRepo for SqlitePostingRepo {
    fn insert(
        &self,
        conn: &Connection,
        posting: &Posting,
    ) -> Result<PostingId, crate::error::DbError> {
        let precision = get_precision(conn, posting.commodity_id)?;
        let amount_i64 = accounting::amount::to_db_amount(posting.amount, precision);
        let cost_precision = posting
            .cost_commodity_id
            .map(|id| get_precision(conn, id))
            .transpose()?
            .unwrap_or(precision);
        let cost_i64 = posting
            .cost
            .map(|c| accounting::amount::to_db_amount(c, cost_precision));
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
        // 准备查询语句，获取指定交易的所有原始分录数据
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE transaction_id = ?1"
        )?;
        // 先以原始 i64 形式读取所有行，避免在闭包中查询精度导致编译问题
        let raw_rows: Vec<_> = stmt
            .query_map(params![transaction_id.0], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
        // 逐行根据商品精度还原 Decimal 金额并构造 Posting
        let mut postings = Vec::new();
        for (
            id,
            tx_id,
            account_id,
            commodity_id,
            amount,
            cost,
            cost_commodity_id,
            description,
            member_id,
            channel_id,
        ) in raw_rows
        {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            let cost_precision = cost_commodity_id
                .map(|cid| get_precision(conn, CommodityId(cid)))
                .transpose()?
                .unwrap_or(precision);
            postings.push(Posting {
                id: PostingId(id),
                transaction_id: TransactionId(tx_id),
                account_id: AccountId(account_id),
                commodity_id: CommodityId(commodity_id),
                amount: accounting::amount::from_db_amount(amount, precision),
                cost: cost.map(|c| accounting::amount::from_db_amount(c, cost_precision)),
                cost_commodity_id: cost_commodity_id.map(CommodityId),
                description,
                member_id: member_id.map(accounting::id::MemberId),
                channel_id: channel_id.map(accounting::id::ChannelId),
            });
        }
        Ok(postings)
    }

    fn list_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<Posting>, crate::error::DbError> {
        // 准备查询语句，按交易 ID 排序获取该账户下所有原始分录
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id
             FROM postings WHERE account_id = ?1 ORDER BY transaction_id"
        )?;
        // 先以原始 i64 读取所有行，避免闭包内查询精度
        let raw_rows: Vec<_> = stmt
            .query_map(params![account_id.0], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
        // 逐行还原精度并构造 Posting 对象
        let mut postings = Vec::new();
        for (
            id,
            tx_id,
            account_id,
            commodity_id,
            amount,
            cost,
            cost_commodity_id,
            description,
            member_id,
            channel_id,
        ) in raw_rows
        {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            let cost_precision = cost_commodity_id
                .map(|cid| get_precision(conn, CommodityId(cid)))
                .transpose()?
                .unwrap_or(precision);
            postings.push(Posting {
                id: PostingId(id),
                transaction_id: TransactionId(tx_id),
                account_id: AccountId(account_id),
                commodity_id: CommodityId(commodity_id),
                amount: accounting::amount::from_db_amount(amount, precision),
                cost: cost.map(|c| accounting::amount::from_db_amount(c, cost_precision)),
                cost_commodity_id: cost_commodity_id.map(CommodityId),
                description,
                member_id: member_id.map(accounting::id::MemberId),
                channel_id: channel_id.map(accounting::id::ChannelId),
            });
        }
        Ok(postings)
    }

    fn sum_by_account(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT commodity_id, SUM(amount) FROM postings WHERE account_id = ?1 GROUP BY commodity_id"
        )?;
        let raw_rows: Vec<_> = stmt
            .query_map(params![account_id.0], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<_, _>>()?;
        let mut result = Vec::new();
        for (commodity_id, amount) in raw_rows {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            result.push((
                CommodityId(commodity_id),
                accounting::amount::from_db_amount(amount, precision),
            ));
        }
        Ok(result)
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

    fn sum_with_ancestors(
        &self,
        conn: &Connection,
        ancestor_id: AccountId,
    ) -> Result<Vec<(CommodityId, Decimal)>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT p.commodity_id, SUM(p.amount)
             FROM postings p
             WHERE p.account_id IN (
                 SELECT account_id FROM account_ancestors WHERE ancestor_id = ?1
                 UNION SELECT ?1
             )
             GROUP BY p.commodity_id",
        )?;
        let raw_rows: Vec<_> = stmt
            .query_map(params![ancestor_id.0], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<_, _>>()?;
        let mut result = Vec::new();
        for (commodity_id, amount) in raw_rows {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            result.push((
                CommodityId(commodity_id),
                accounting::amount::from_db_amount(amount, precision),
            ));
        }
        Ok(result)
    }

    fn sum_by_tag(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
    ) -> Result<Vec<(TagId, CommodityId, i64, Decimal)>, crate::error::DbError> {
        let mut conditions = vec!["1=1"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("t.date_time >= ?");
            params_vec.push(Box::new(start.and_hms_opt(0, 0, 0).unwrap().to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("t.date_time <= ?");
            params_vec.push(Box::new(end.and_hms_opt(23, 59, 59).unwrap().to_string()));
        }
        if let Some(account) = filter.account_id {
            conditions.push("p.account_id = ?");
            params_vec.push(Box::new(account.0));
        }
        if let Some(member) = filter.member_id {
            conditions.push("t.member_id = ?");
            params_vec.push(Box::new(member.0));
        }
        if let Some(channel) = filter.channel_id {
            conditions.push("p.channel_id = ?");
            params_vec.push(Box::new(channel.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("t.description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("t.is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT tt.tag_id, p.commodity_id, a.account_type, SUM(p.amount) as total
             FROM postings p
             JOIN accounts a ON p.account_id = a.id
             JOIN transactions t ON p.transaction_id = t.id
             JOIN transaction_tags tt ON tt.transaction_id = t.id
             WHERE {}
             GROUP BY tt.tag_id, p.commodity_id, a.account_type",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let raw_rows: Vec<_> = stmt
            .query_map(rusqlite::params_from_iter(param_refs), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<_, _>>()?;

        let mut result = Vec::new();
        for (tag_id, commodity_id, account_type, amount) in raw_rows {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            result.push((
                TagId(tag_id),
                CommodityId(commodity_id),
                account_type,
                accounting::amount::from_db_amount(amount, precision),
            ));
        }
        Ok(result)
    }

    fn sum_by_member(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
    ) -> Result<Vec<(MemberId, CommodityId, i64, Decimal)>, crate::error::DbError> {
        let mut conditions = vec!["1=1", "t.member_id IS NOT NULL"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("t.date_time >= ?");
            params_vec.push(Box::new(start.and_hms_opt(0, 0, 0).unwrap().to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("t.date_time <= ?");
            params_vec.push(Box::new(end.and_hms_opt(23, 59, 59).unwrap().to_string()));
        }
        if let Some(account) = filter.account_id {
            conditions.push("p.account_id = ?");
            params_vec.push(Box::new(account.0));
        }
        if let Some(channel) = filter.channel_id {
            conditions.push("p.channel_id = ?");
            params_vec.push(Box::new(channel.0));
        }
        if let Some(tag) = filter.tag_id {
            conditions.push(
                "EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id = ?)"
            );
            params_vec.push(Box::new(tag.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("t.description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("t.is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT t.member_id, p.commodity_id, a.account_type, SUM(p.amount) as total
             FROM postings p
             JOIN accounts a ON p.account_id = a.id
             JOIN transactions t ON p.transaction_id = t.id
             WHERE {}
             GROUP BY t.member_id, p.commodity_id, a.account_type",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let raw_rows: Vec<_> = stmt
            .query_map(rusqlite::params_from_iter(param_refs), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<_, _>>()?;

        let mut result = Vec::new();
        for (member_id, commodity_id, account_type, amount) in raw_rows {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            result.push((
                MemberId(member_id),
                CommodityId(commodity_id),
                account_type,
                accounting::amount::from_db_amount(amount, precision),
            ));
        }
        Ok(result)
    }

    fn sum_by_channel(
        &self,
        conn: &Connection,
        filter: &TransactionFilter,
    ) -> Result<Vec<(ChannelId, CommodityId, i64, Decimal)>, crate::error::DbError> {
        let mut conditions = vec!["1=1", "p.channel_id IS NOT NULL"];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(start) = filter.start_date {
            conditions.push("t.date_time >= ?");
            params_vec.push(Box::new(start.and_hms_opt(0, 0, 0).unwrap().to_string()));
        }
        if let Some(end) = filter.end_date {
            conditions.push("t.date_time <= ?");
            params_vec.push(Box::new(end.and_hms_opt(23, 59, 59).unwrap().to_string()));
        }
        if let Some(account) = filter.account_id {
            conditions.push("p.account_id = ?");
            params_vec.push(Box::new(account.0));
        }
        if let Some(member) = filter.member_id {
            conditions.push("t.member_id = ?");
            params_vec.push(Box::new(member.0));
        }
        if let Some(tag) = filter.tag_id {
            conditions.push(
                "EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id = ?)"
            );
            params_vec.push(Box::new(tag.0));
        }
        if let Some(ref keyword) = filter.keyword {
            conditions.push("t.description LIKE ?");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        if let Some(is_template) = filter.is_template {
            conditions.push("t.is_template = ?");
            params_vec.push(Box::new(is_template as i64));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT p.channel_id, p.commodity_id, a.account_type, SUM(p.amount) as total
             FROM postings p
             JOIN accounts a ON p.account_id = a.id
             JOIN transactions t ON p.transaction_id = t.id
             WHERE {}
             GROUP BY p.channel_id, p.commodity_id, a.account_type",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let raw_rows: Vec<_> = stmt
            .query_map(rusqlite::params_from_iter(param_refs), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<_, _>>()?;

        let mut result = Vec::new();
        for (channel_id, commodity_id, account_type, amount) in raw_rows {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            result.push((
                ChannelId(channel_id),
                CommodityId(commodity_id),
                account_type,
                accounting::amount::from_db_amount(amount, precision),
            ));
        }
        Ok(result)
    }
}

/// 查询商品的精度
fn get_precision(
    conn: &Connection,
    commodity_id: CommodityId,
) -> Result<u8, crate::error::DbError> {
    let precision: i32 = conn.query_row(
        "SELECT precision FROM commodities WHERE id = ?1",
        params![commodity_id.0],
        |row| row.get(0),
    )?;
    Ok(precision as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{AccountId, ChannelId, CommodityId, MemberId, TagId};
    use accounting::transaction_filter::TransactionFilter;
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn setup() -> (Connection, SqlitePostingRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn, "en").unwrap();
        (conn, SqlitePostingRepo)
    }

    fn insert_transaction(conn: &Connection) -> TransactionId {
        conn.execute(
            "INSERT INTO transactions (date_time, description, is_template) VALUES ('2024-01-01 00:00:00', 'test', 0)",
            [],
        )
        .unwrap();
        TransactionId(conn.last_insert_rowid())
    }

    fn insert_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, is_system) VALUES (?1, 1, 0)",
            [name],
        )
        .unwrap();
        AccountId(conn.last_insert_rowid())
    }

    /// 插入 Income 类型账户（account_type = 4）
    fn insert_income_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, is_system) VALUES (?1, 4, 0)",
            [name],
        )
        .unwrap();
        AccountId(conn.last_insert_rowid())
    }

    /// 插入 Expense 类型账户（account_type = 5）
    fn insert_expense_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, is_system) VALUES (?1, 5, 0)",
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

    /// 测试按标签汇总分录金额
    #[test]
    fn test_sum_by_tag() {
        let (conn, repo) = setup();
        let income = insert_income_account(&conn, "Income:Salary");
        let expense = insert_expense_account(&conn, "Expenses:Food");

        // 创建标签
        conn.execute(
            "INSERT INTO tags (name, description, is_system) VALUES ('餐饮', NULL, 0)",
            [],
        )
        .unwrap();
        let tag_id = TagId(conn.last_insert_rowid());

        // 插入交易
        conn.execute(
            "INSERT INTO transactions (date_time, description, is_template) VALUES ('2024-01-15 00:00:00', 'lunch', 0)",
            [],
        )
        .unwrap();
        let tx_id = TransactionId(conn.last_insert_rowid());

        // 关联标签
        conn.execute(
            "INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)",
            [tx_id.0, tag_id.0],
        )
        .unwrap();

        // 插入分录
        let p1 = sample_posting(tx_id, income, "100.00");
        let p2 = sample_posting(tx_id, expense, "-100.00");
        repo.insert(&conn, &p1).unwrap();
        repo.insert(&conn, &p2).unwrap();

        // 无过滤查询
        let results = repo
            .sum_by_tag(&conn, &TransactionFilter::default())
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == 4).unwrap();
        let expense_row = results.iter().find(|r| r.2 == 5).unwrap();
        assert_eq!(income_row.0, tag_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("100.00").unwrap());
        assert_eq!(expense_row.0, tag_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-100.00").unwrap());

        // 日期过滤（包含）
        let filter_include = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            ..Default::default()
        };
        let results = repo.sum_by_tag(&conn, &filter_include).unwrap();
        assert_eq!(results.len(), 2);

        // 日期过滤（排除）
        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 2, 28).unwrap()),
            ..Default::default()
        };
        let results = repo.sum_by_tag(&conn, &filter_exclude).unwrap();
        assert!(results.is_empty());
    }

    /// 测试按成员汇总分录金额
    #[test]
    fn test_sum_by_member() {
        let (conn, repo) = setup();
        let income = insert_income_account(&conn, "Income:Bonus");
        let expense = insert_expense_account(&conn, "Expenses:Transport");

        // 创建成员
        conn.execute("INSERT INTO members (name) VALUES ('Alice')", [])
            .unwrap();
        let member_id = MemberId(conn.last_insert_rowid());

        // 插入交易（关联成员）
        conn.execute(
            "INSERT INTO transactions (date_time, description, member_id, is_template) VALUES ('2024-03-10 00:00:00', 'commute', ?1, 0)",
            [member_id.0],
        )
        .unwrap();
        let tx_id = TransactionId(conn.last_insert_rowid());

        // 插入分录
        let p1 = sample_posting(tx_id, income, "200.00");
        let p2 = sample_posting(tx_id, expense, "-200.00");
        repo.insert(&conn, &p1).unwrap();
        repo.insert(&conn, &p2).unwrap();

        // 无过滤查询
        let results = repo
            .sum_by_member(&conn, &TransactionFilter::default())
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == 4).unwrap();
        let expense_row = results.iter().find(|r| r.2 == 5).unwrap();
        assert_eq!(income_row.0, member_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("200.00").unwrap());
        assert_eq!(expense_row.0, member_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-200.00").unwrap());

        // member_id 过滤条件应被忽略（维度自身不过滤自身）
        let filter_member = TransactionFilter {
            member_id: Some(member_id),
            ..Default::default()
        };
        let results = repo.sum_by_member(&conn, &filter_member).unwrap();
        assert_eq!(results.len(), 2);

        // 日期过滤（排除）
        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 4, 30).unwrap()),
            ..Default::default()
        };
        let results = repo.sum_by_member(&conn, &filter_exclude).unwrap();
        assert!(results.is_empty());
    }

    /// 测试按渠道汇总分录金额
    #[test]
    fn test_sum_by_channel() {
        let (conn, repo) = setup();
        let income = insert_income_account(&conn, "Income:Refund");
        let expense = insert_expense_account(&conn, "Expenses:Shopping");

        // 创建渠道
        conn.execute(
            "INSERT INTO channels (name, description) VALUES ('Alipay', NULL)",
            [],
        )
        .unwrap();
        let channel_id = ChannelId(conn.last_insert_rowid());

        // 插入交易
        conn.execute(
            "INSERT INTO transactions (date_time, description, is_template) VALUES ('2024-06-01 00:00:00', 'online shopping', 0)",
            [],
        )
        .unwrap();
        let tx_id = TransactionId(conn.last_insert_rowid());

        // 插入分录（带渠道）
        let mut p1 = sample_posting(tx_id, income, "300.00");
        p1.channel_id = Some(channel_id);
        let mut p2 = sample_posting(tx_id, expense, "-300.00");
        p2.channel_id = Some(channel_id);
        repo.insert(&conn, &p1).unwrap();
        repo.insert(&conn, &p2).unwrap();

        // 无过滤查询
        let results = repo
            .sum_by_channel(&conn, &TransactionFilter::default())
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == 4).unwrap();
        let expense_row = results.iter().find(|r| r.2 == 5).unwrap();
        assert_eq!(income_row.0, channel_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("300.00").unwrap());
        assert_eq!(expense_row.0, channel_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-300.00").unwrap());

        // channel_id 过滤条件应被忽略（维度自身不过滤自身）
        let filter_channel = TransactionFilter {
            channel_id: Some(channel_id),
            ..Default::default()
        };
        let results = repo.sum_by_channel(&conn, &filter_channel).unwrap();
        assert_eq!(results.len(), 2);

        // 日期过滤（排除）
        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 7, 31).unwrap()),
            ..Default::default()
        };
        let results = repo.sum_by_channel(&conn, &filter_exclude).unwrap();
        assert!(results.is_empty());
    }
}
