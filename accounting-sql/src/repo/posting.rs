use std::collections::HashMap;

use rust_decimal::Decimal;
use sqlx::{QueryBuilder, SqliteConnection};

use crate::error::DbError;
use accounting::amount::{from_db_amount, to_db_amount};
use accounting::datetime_utils;
use accounting::id::{
    AccountId, ChannelId, CommodityId, MemberId, PostingId, TagId, TransactionId,
};
use accounting::posting::Posting;
use accounting::transaction_filter::TransactionFilter;

pub async fn posting_insert(
    conn: &mut SqliteConnection,
    posting: &Posting,
) -> Result<PostingId, DbError> {
    let precision = get_precision(conn, posting.commodity_id).await?;
    let amount_i64 = to_db_amount(posting.amount, precision);
    let cost_precision = match posting.cost_commodity_id {
        Some(id) => get_precision(conn, id).await?,
        None => precision,
    };
    let cost_i64 = posting.cost.map(|c| to_db_amount(c, cost_precision));

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO postings
         (transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, is_reimbursable, linked_posting_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id",
    )
    .bind(posting.transaction_id.0)
    .bind(posting.account_id.0)
    .bind(posting.commodity_id.0)
    .bind(amount_i64)
    .bind(cost_i64)
    .bind(posting.cost_commodity_id.map(|id| id.0))
    .bind(posting.is_reimbursable as i32)
    .bind(posting.linked_posting_id.map(|id| id.0))
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(PostingId(id))
}

pub async fn posting_get(
    conn: &mut SqliteConnection,
    id: PostingId,
) -> Result<Option<Posting>, DbError> {
    let row: Option<PostingRawRow> = sqlx::query_as(
        "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, is_reimbursable, linked_posting_id, reversal_total
         FROM postings WHERE id = ?1",
    )
    .bind(id.0)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    match row {
        Some(raw) => Ok(Some(raw.into_posting(&mut *conn).await?)),
        None => Ok(None),
    }
}

pub async fn posting_list_by_transaction(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
) -> Result<Vec<Posting>, DbError> {
    let precisions = load_precisions(conn).await?;
    let rows: Vec<PostingRawRow> = sqlx::query_as(
        "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, is_reimbursable, linked_posting_id, reversal_total
         FROM postings WHERE transaction_id = ?1",
    )
    .bind(transaction_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    rows.into_iter()
        .map(|r| r.into_posting_with_precisions(&precisions))
        .collect::<Result<_, _>>()
}

pub async fn posting_list_by_account(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<Vec<Posting>, DbError> {
    let precisions = load_precisions(conn).await?;
    let rows: Vec<PostingRawRow> = sqlx::query_as(
        "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, is_reimbursable, linked_posting_id, reversal_total
         FROM postings WHERE account_id = ?1 ORDER BY transaction_id",
    )
    .bind(account_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    rows.into_iter()
        .map(|r| r.into_posting_with_precisions(&precisions))
        .collect::<Result<_, _>>()
}

pub async fn posting_has_postings(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<bool, DbError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM postings WHERE account_id = ?1")
        .bind(account_id.0)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(count > 0)
}

pub async fn posting_sum_by_account(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<Vec<(CommodityId, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT commodity_id, SUM(amount) FROM postings WHERE account_id = ?1 GROUP BY commodity_id",
    )
    .bind(account_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(commodity_id, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (CommodityId(commodity_id), from_db_amount(amount, precision))
        })
        .collect())
}

pub async fn posting_delete_by_transaction(
    conn: &mut SqliteConnection,
    transaction_id: TransactionId,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM postings WHERE transaction_id = ?1")
        .bind(transaction_id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn posting_sum_with_ancestors(
    conn: &mut SqliteConnection,
    ancestor_id: AccountId,
) -> Result<Vec<(CommodityId, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT p.commodity_id, SUM(p.amount)
         FROM postings p
         WHERE p.account_id IN (
             SELECT account_id FROM account_ancestors WHERE ancestor_id = ?1
             UNION SELECT ?1
         )
         GROUP BY p.commodity_id",
    )
    .bind(ancestor_id.0)
    .bind(ancestor_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(commodity_id, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (CommodityId(commodity_id), from_db_amount(amount, precision))
        })
        .collect())
}

pub async fn posting_sum_by_tag(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
) -> Result<Vec<(TagId, CommodityId, String, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT tt.tag_id, p.commodity_id, ra.name, SUM(p.amount) as total
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN account_ancestors aa ON a.id = aa.account_id AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
         JOIN accounts ra ON aa.ancestor_id = ra.id
         JOIN transactions t ON p.transaction_id = t.id
         JOIN transaction_tags tt ON tt.transaction_id = t.id
         WHERE 1=1 ",
    );

    apply_posting_filter(&mut builder, filter, false);

    builder.push(" GROUP BY tt.tag_id, p.commodity_id, ra.name");

    let rows: Vec<(i64, i64, String, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(tag_id, commodity_id, root_name, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (
                TagId(tag_id),
                CommodityId(commodity_id),
                root_name,
                from_db_amount(amount, precision),
            )
        })
        .collect())
}

pub async fn posting_sum_by_member(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
) -> Result<Vec<(MemberId, CommodityId, String, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT t.member_id, p.commodity_id, ra.name, SUM(p.amount) as total
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN account_ancestors aa ON a.id = aa.account_id AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
         JOIN accounts ra ON aa.ancestor_id = ra.id
         JOIN transactions t ON p.transaction_id = t.id
         WHERE t.member_id IS NOT NULL ",
    );

    apply_posting_filter(&mut builder, filter, true);

    builder.push(" GROUP BY t.member_id, p.commodity_id, ra.name");

    let rows: Vec<(i64, i64, String, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(member_id, commodity_id, root_name, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (
                MemberId(member_id),
                CommodityId(commodity_id),
                root_name,
                from_db_amount(amount, precision),
            )
        })
        .collect())
}

pub async fn posting_sum_by_channel(
    conn: &mut SqliteConnection,
    filter: &TransactionFilter,
) -> Result<Vec<(ChannelId, CommodityId, String, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT cp.channel_id, p.commodity_id, ra.name, SUM(p.amount) as total
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN account_ancestors aa ON a.id = aa.account_id AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
         JOIN accounts ra ON aa.ancestor_id = ra.id
         JOIN transactions t ON p.transaction_id = t.id
         JOIN channel_paths cp ON cp.transaction_id = t.id
         WHERE 1=1 ",
    );

    apply_posting_filter(&mut builder, filter, true);

    builder.push(" GROUP BY cp.channel_id, p.commodity_id, ra.name");

    let rows: Vec<(i64, i64, String, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(channel_id, commodity_id, root_name, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (
                ChannelId(channel_id),
                CommodityId(commodity_id),
                root_name,
                from_db_amount(amount, precision),
            )
        })
        .collect())
}

/// 收支汇总结果
pub struct PostingSummary {
    /// 收入（资产类分录正金额之和）
    pub income: Decimal,
    /// 支出（资产类分录负金额之和的绝对值）
    pub expense: Decimal,
}

pub async fn posting_summary(
    conn: &mut SqliteConnection,
    start: Option<chrono::NaiveDate>,
    end: Option<chrono::NaiveDate>,
) -> Result<PostingSummary, DbError> {
    let precisions = load_precisions(conn).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT p.commodity_id, SUM(p.amount)
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN account_ancestors aa ON a.id = aa.account_id
         JOIN accounts ra ON aa.ancestor_id = ra.id AND aa.depth = (
             SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id
         )
         JOIN transactions t ON p.transaction_id = t.id
         WHERE ra.name IN ('Assets', 'Asset', '资产') ",
    );

    if start.is_some() || end.is_some() {
        builder.push("AND 1=1 ");
    }

    if let Some(start) = start {
        builder.push("AND t.date_time >= ");
        builder.push_bind(datetime_utils::start_of_day(start).to_string());
        builder.push(" ");
    }
    if let Some(end) = end {
        builder.push("AND t.date_time <= ");
        builder.push_bind(datetime_utils::end_of_day(end).to_string());
        builder.push(" ");
    }

    builder.push(" GROUP BY p.commodity_id");

    let rows: Vec<(i64, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let mut income = Decimal::ZERO;
    let mut expense = Decimal::ZERO;
    for (commodity_id, amount) in rows {
        let precision = precisions
            .get(&CommodityId(commodity_id))
            .copied()
            .unwrap_or(2);
        let d = from_db_amount(amount, precision);
        if d > Decimal::ZERO {
            income += d;
        } else {
            expense += d.abs();
        }
    }

    Ok(PostingSummary { income, expense })
}

fn apply_posting_filter(
    builder: &mut QueryBuilder<sqlx::Sqlite>,
    filter: &TransactionFilter,
    skip_member_channel: bool,
) {
    if let Some(start) = filter.start_date {
        builder.push("AND t.date_time >= ");
        builder.push_bind(datetime_utils::start_of_day(start).to_string());
        builder.push(" ");
    }
    if let Some(end) = filter.end_date {
        builder.push("AND t.date_time <= ");
        builder.push_bind(datetime_utils::end_of_day(end).to_string());
        builder.push(" ");
    }
    if !filter.account_ids.is_empty() {
        builder.push("AND p.account_id IN (");
        let mut separated = builder.separated(", ");
        for account in &filter.account_ids {
            separated.push_bind(account.0);
        }
        builder.push(") ");
    }
    if !filter.member_ids.is_empty() && !skip_member_channel {
        builder.push("AND t.member_id IN (");
        let mut separated = builder.separated(", ");
        for member in &filter.member_ids {
            separated.push_bind(member.0);
        }
        builder.push(") ");
    }
    if !filter.channel_ids.is_empty() && !skip_member_channel {
        builder.push(
            "AND EXISTS (SELECT 1 FROM channel_paths cp WHERE cp.transaction_id = t.id AND cp.channel_id IN (",
        );
        let mut separated = builder.separated(", ");
        for channel in &filter.channel_ids {
            separated.push_bind(channel.0);
        }
        builder.push(")) ");
    }
    if !filter.tag_ids.is_empty() {
        builder.push(
            "AND EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id IN (",
        );
        let mut separated = builder.separated(", ");
        for tag in &filter.tag_ids {
            separated.push_bind(tag.0);
        }
        builder.push(")) ");
    }
    if let Some(ref keyword) = filter.keyword {
        builder.push("AND t.description LIKE ");
        builder.push_bind(format!("%{}%", keyword));
        builder.push(" ");
    }
}

/// 按账户组汇总分录金额（含闭包表后代聚合，排除指定标签的交易，仅统计指定币种）
///
/// - 通过 account_ancestors 闭包表找到每个账户的所有后代
/// - 排除带 exclude_tag_ids 中任一标签的交易
/// - 仅统计 commodity_id 匹配的分录
pub async fn sum_by_account_with_descendants(
    conn: &mut SqliteConnection,
    account_ids: &[AccountId],
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    exclude_tag_ids: &[TagId],
    commodity_id: CommodityId,
) -> Result<Vec<(AccountId, Decimal)>, DbError> {
    if account_ids.is_empty() {
        return Ok(vec![]);
    }

    let precision = get_precision(conn, commodity_id).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT anc.ancestor_id, SUM(p.amount) as total
         FROM postings p
         JOIN account_ancestors anc ON p.account_id = anc.account_id
         JOIN transactions t ON p.transaction_id = t.id
         WHERE anc.ancestor_id IN (",
    );

    let mut separated = builder.separated(", ");
    for id in account_ids {
        separated.push_bind(id.0);
    }
    builder.push(") AND t.date_time >= ");
    builder.push_bind(datetime_utils::start_of_day(start_date).to_string());
    builder.push(" AND t.date_time <= ");
    builder.push_bind(datetime_utils::end_of_day(end_date).to_string());
    builder.push(" AND p.commodity_id = ");
    builder.push_bind(commodity_id.0);

    if !exclude_tag_ids.is_empty() {
        builder.push(" AND NOT EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id IN (");
        let mut sep = builder.separated(", ");
        for tid in exclude_tag_ids {
            sep.push_bind(tid.0);
        }
        builder.push("))");
    }

    builder.push(" GROUP BY anc.ancestor_id");

    let rows: Vec<(i64, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(account_id, amount)| (AccountId(account_id), from_db_amount(amount, precision)))
        .collect())
}

/// 按账户组汇总分录金额（不含闭包表后代聚合，排除指定标签的交易，仅统计指定币种）
///
/// 与 `sum_by_account_with_descendants` 不同，此方法仅统计指定账户自身的分录，
/// 不包含后代账户。用于资金流量表和预算执行表。
pub async fn posting_sum_by_period(
    conn: &mut SqliteConnection,
    account_ids: &[AccountId],
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    exclude_tag_ids: &[TagId],
    commodity_id: CommodityId,
) -> Result<Vec<(AccountId, Decimal)>, DbError> {
    if account_ids.is_empty() {
        return Ok(vec![]);
    }

    let precision = get_precision(conn, commodity_id).await?;

    let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT p.account_id, SUM(p.amount) as total
         FROM postings p
         JOIN transactions t ON p.transaction_id = t.id
         WHERE p.account_id IN (",
    );

    let mut separated = builder.separated(", ");
    for id in account_ids {
        separated.push_bind(id.0);
    }
    builder.push(") AND t.date_time >= ");
    builder.push_bind(datetime_utils::start_of_day(start_date).to_string());
    builder.push(" AND t.date_time <= ");
    builder.push_bind(datetime_utils::end_of_day(end_date).to_string());
    builder.push(" AND p.commodity_id = ");
    builder.push_bind(commodity_id.0);

    if !exclude_tag_ids.is_empty() {
        builder.push(" AND NOT EXISTS (SELECT 1 FROM transaction_tags tt WHERE tt.transaction_id = t.id AND tt.tag_id IN (");
        let mut sep = builder.separated(", ");
        for tid in exclude_tag_ids {
            sep.push_bind(tid.0);
        }
        builder.push("))");
    }

    builder.push(" GROUP BY p.account_id");

    let rows: Vec<(i64, i64)> = builder
        .build_query_as()
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(account_id, amount)| (AccountId(account_id), from_db_amount(amount, precision)))
        .collect())
}

/// 统计所有资产类账户的余额（单条 SQL）
///
/// 返回 (account_id, commodity_id, balance) 列表，仅包含资产根账户下的账户。
pub async fn posting_sum_all_assets(
    conn: &mut SqliteConnection,
) -> Result<Vec<(AccountId, CommodityId, Decimal)>, DbError> {
    let precisions = load_precisions(conn).await?;

    let rows: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT p.account_id, p.commodity_id, SUM(p.amount)
         FROM postings p
         JOIN accounts a ON p.account_id = a.id
         JOIN account_ancestors aa ON a.id = aa.account_id
         JOIN accounts root ON aa.ancestor_id = root.id
           AND aa.depth = (SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id)
         WHERE root.name IN ('Assets', '资产')
         GROUP BY p.account_id, p.commodity_id
         HAVING SUM(p.amount) != 0",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(account_id, commodity_id, amount)| {
            let precision = precisions
                .get(&CommodityId(commodity_id))
                .copied()
                .unwrap_or(2);
            (
                AccountId(account_id),
                CommodityId(commodity_id),
                from_db_amount(amount, precision),
            )
        })
        .collect())
}

async fn load_precisions(conn: &mut SqliteConnection) -> Result<HashMap<CommodityId, u8>, DbError> {
    let rows: Vec<(i64, i32)> = sqlx::query_as("SELECT id, precision FROM commodities")
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(id, p)| (CommodityId(id), p as u8))
        .collect())
}

async fn get_precision(
    conn: &mut SqliteConnection,
    commodity_id: CommodityId,
) -> Result<u8, DbError> {
    let precision: i32 = sqlx::query_scalar("SELECT precision FROM commodities WHERE id = ?1")
        .bind(commodity_id.0)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(precision as u8)
}

#[derive(sqlx::FromRow)]
struct PostingRawRow {
    id: i64,
    transaction_id: i64,
    account_id: i64,
    commodity_id: i64,
    amount: i64,
    cost: Option<i64>,
    cost_commodity_id: Option<i64>,
    is_reimbursable: i32,
    linked_posting_id: Option<i64>,
    reversal_total: i64,
}

impl PostingRawRow {
    async fn into_posting(self, conn: &mut SqliteConnection) -> Result<Posting, DbError> {
        let precision = get_precision(conn, CommodityId(self.commodity_id)).await?;
        let cost_precision = match self.cost_commodity_id {
            Some(cid) => get_precision(conn, CommodityId(cid)).await?,
            None => precision,
        };
        Ok(self.build(precision, cost_precision))
    }

    fn into_posting_with_precisions(
        self,
        precisions: &HashMap<CommodityId, u8>,
    ) -> Result<Posting, DbError> {
        let precision = precisions
            .get(&CommodityId(self.commodity_id))
            .copied()
            .unwrap_or(2);
        let cost_precision = match self.cost_commodity_id {
            Some(cid) => precisions.get(&CommodityId(cid)).copied().unwrap_or(2),
            None => precision,
        };
        Ok(self.build(precision, cost_precision))
    }

    fn build(self, precision: u8, cost_precision: u8) -> Posting {
        Posting {
            id: PostingId(self.id),
            transaction_id: TransactionId(self.transaction_id),
            account_id: AccountId(self.account_id),
            commodity_id: CommodityId(self.commodity_id),
            amount: from_db_amount(self.amount, precision),
            cost: self.cost.map(|c| from_db_amount(c, cost_precision)),
            cost_commodity_id: self.cost_commodity_id.map(CommodityId),
            is_reimbursable: self.is_reimbursable != 0,
            linked_posting_id: self.linked_posting_id.map(PostingId),
            reversal_total: from_db_amount(self.reversal_total, precision),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::account::{account_create_with_closure, account_get_by_name};
    use accounting::account::Account;
    use accounting::id::{AccountId, ChannelId, CommodityId, MemberId, TagId};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use sqlx::{Connection, SqliteConnection};
    use std::str::FromStr;

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
        conn
    }

    async fn ensure_test_member(conn: &mut SqliteConnection) -> i64 {
        sqlx::query("INSERT OR IGNORE INTO members (name) VALUES ('Test Member')")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query_scalar("SELECT id FROM members WHERE name = 'Test Member'")
            .fetch_one(conn)
            .await
            .unwrap()
    }

    async fn insert_transaction(conn: &mut SqliteConnection) -> TransactionId {
        let member_id = ensure_test_member(conn).await;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-01-01 00:00:00', 'test', ?1) RETURNING id",
        )
        .bind(member_id)
        .fetch_one(conn)
        .await
        .unwrap();
        TransactionId(id)
    }

    async fn insert_account(conn: &mut SqliteConnection, name: &str) -> AccountId {
        let root_id = account_get_by_name(conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        let account = Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id: Some(root_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        account_create_with_closure(conn, &account).await.unwrap()
    }

    async fn insert_income_account(conn: &mut SqliteConnection, name: &str) -> AccountId {
        let root_id = account_get_by_name(conn, "Income")
            .await
            .unwrap()
            .unwrap()
            .id;
        let account = Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id: Some(root_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        account_create_with_closure(conn, &account).await.unwrap()
    }

    async fn insert_expense_account(conn: &mut SqliteConnection, name: &str) -> AccountId {
        let root_id = account_get_by_name(conn, "Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let account = Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id: Some(root_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        account_create_with_closure(conn, &account).await.unwrap()
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
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    #[tokio::test]
    async fn test_insert_and_list_by_transaction() {
        let mut conn = setup().await;
        let tx_id = insert_transaction(&mut conn).await;
        let a1 = insert_account(&mut conn, "Assets:A").await;
        let a2 = insert_account(&mut conn, "Assets:B").await;
        let p1 = sample_posting(tx_id, a1, "100.00");
        let p2 = sample_posting(tx_id, a2, "-100.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        let list = posting_list_by_transaction(&mut conn, tx_id).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_account() {
        let mut conn = setup().await;
        let tx_id = insert_transaction(&mut conn).await;
        let a = insert_account(&mut conn, "Assets:C").await;
        let p = sample_posting(tx_id, a, "50.00");
        posting_insert(&mut conn, &p).await.unwrap();

        let list = posting_list_by_account(&mut conn, a).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].amount, Decimal::from_str("50.00").unwrap());
    }

    #[tokio::test]
    async fn test_sum_by_account() {
        let mut conn = setup().await;
        let tx_id = insert_transaction(&mut conn).await;
        let a = insert_account(&mut conn, "Assets:D").await;
        let p1 = sample_posting(tx_id, a, "100.00");
        let p2 = sample_posting(tx_id, a, "50.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        let sums = posting_sum_by_account(&mut conn, a).await.unwrap();
        assert_eq!(sums.len(), 1);
        assert_eq!(sums[0].0, CommodityId(1));
        assert_eq!(sums[0].1, Decimal::from_str("150.00").unwrap());
    }

    #[tokio::test]
    async fn test_delete_by_transaction() {
        let mut conn = setup().await;
        let tx_id = insert_transaction(&mut conn).await;
        let a = insert_account(&mut conn, "Assets:E").await;
        let p = sample_posting(tx_id, a, "10.00");
        posting_insert(&mut conn, &p).await.unwrap();
        posting_delete_by_transaction(&mut conn, tx_id)
            .await
            .unwrap();
        let list = posting_list_by_transaction(&mut conn, tx_id).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_amount_roundtrip() {
        let mut conn = setup().await;
        let tx_id = insert_transaction(&mut conn).await;
        let a = insert_account(&mut conn, "Assets:F").await;
        let mut p = sample_posting(tx_id, a, "123.45");
        p.cost = Some(Decimal::from_str("67.89").unwrap());
        p.cost_commodity_id = Some(CommodityId(1));
        posting_insert(&mut conn, &p).await.unwrap();

        let list = posting_list_by_transaction(&mut conn, tx_id).await.unwrap();
        assert_eq!(list[0].amount, Decimal::from_str("123.45").unwrap());
        assert_eq!(list[0].cost, Some(Decimal::from_str("67.89").unwrap()));
    }

    #[tokio::test]
    async fn test_sum_by_tag() {
        let mut conn = setup().await;
        let income = insert_income_account(&mut conn, "Income:Salary").await;
        let expense = insert_expense_account(&mut conn, "Expenses:Food").await;

        let tag_id: i64 = sqlx::query_scalar(
            "INSERT INTO tags (name, description, is_system) VALUES ('餐饮', NULL, 0) RETURNING id",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let tag_id = TagId(tag_id);

        let member_id = ensure_test_member(&mut conn).await;
        let tx_id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-01-15 00:00:00', 'lunch', ?1) RETURNING id",
        )
        .bind(member_id)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let tx_id = TransactionId(tx_id);

        sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)")
            .bind(tx_id.0)
            .bind(tag_id.0)
            .execute(&mut conn)
            .await
            .unwrap();

        let p1 = sample_posting(tx_id, income, "100.00");
        let p2 = sample_posting(tx_id, expense, "-100.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        let results = posting_sum_by_tag(&mut conn, &TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == "Income").unwrap();
        let expense_row = results.iter().find(|r| r.2 == "Expenses").unwrap();
        assert_eq!(income_row.0, tag_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("100.00").unwrap());
        assert_eq!(expense_row.0, tag_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-100.00").unwrap());

        let filter_include = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            ..Default::default()
        };
        let results = posting_sum_by_tag(&mut conn, &filter_include)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 2, 28).unwrap()),
            ..Default::default()
        };
        let results = posting_sum_by_tag(&mut conn, &filter_exclude)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_sum_by_member() {
        let mut conn = setup().await;
        let income = insert_income_account(&mut conn, "Income:Bonus").await;
        let expense = insert_expense_account(&mut conn, "Expenses:Transport").await;

        let member_id: i64 =
            sqlx::query_scalar("INSERT INTO members (name) VALUES ('Alice') RETURNING id")
                .fetch_one(&mut conn)
                .await
                .unwrap();
        let member_id = MemberId(member_id);

        let tx_id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-03-10 00:00:00', 'commute', ?1) RETURNING id",
        )
        .bind(member_id.0)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let tx_id = TransactionId(tx_id);

        let p1 = sample_posting(tx_id, income, "200.00");
        let p2 = sample_posting(tx_id, expense, "-200.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        let results = posting_sum_by_member(&mut conn, &TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == "Income").unwrap();
        let expense_row = results.iter().find(|r| r.2 == "Expenses").unwrap();
        assert_eq!(income_row.0, member_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("200.00").unwrap());
        assert_eq!(expense_row.0, member_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-200.00").unwrap());

        let filter_member = TransactionFilter {
            member_ids: vec![member_id],
            ..Default::default()
        };
        let results = posting_sum_by_member(&mut conn, &filter_member)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 4, 30).unwrap()),
            ..Default::default()
        };
        let results = posting_sum_by_member(&mut conn, &filter_exclude)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_sum_by_channel() {
        let mut conn = setup().await;
        let income = insert_income_account(&mut conn, "Income:Refund").await;
        let expense = insert_expense_account(&mut conn, "Expenses:Shopping").await;

        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (name, description) VALUES ('TestPay', NULL) RETURNING id",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let channel_id = ChannelId(channel_id);

        let member_id = ensure_test_member(&mut conn).await;
        let tx_id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES ('2024-06-01 00:00:00', 'online shopping', ?1) RETURNING id",
        )
        .bind(member_id)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let tx_id = TransactionId(tx_id);

        // Add channel_path for this transaction
        sqlx::query(
            "INSERT INTO channel_paths (transaction_id, position, channel_id) VALUES (?1, 0, ?2)",
        )
        .bind(tx_id.0)
        .bind(channel_id.0)
        .execute(&mut conn)
        .await
        .unwrap();

        let p1 = sample_posting(tx_id, income, "300.00");
        let p2 = sample_posting(tx_id, expense, "-300.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        let results = posting_sum_by_channel(&mut conn, &TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let income_row = results.iter().find(|r| r.2 == "Income").unwrap();
        let expense_row = results.iter().find(|r| r.2 == "Expenses").unwrap();
        assert_eq!(income_row.0, channel_id);
        assert_eq!(income_row.1, CommodityId(1));
        assert_eq!(income_row.3, Decimal::from_str("300.00").unwrap());
        assert_eq!(expense_row.0, channel_id);
        assert_eq!(expense_row.1, CommodityId(1));
        assert_eq!(expense_row.3, Decimal::from_str("-300.00").unwrap());

        let filter_channel = TransactionFilter {
            channel_ids: vec![channel_id],
            ..Default::default()
        };
        let results = posting_sum_by_channel(&mut conn, &filter_channel)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let filter_exclude = TransactionFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2024, 7, 31).unwrap()),
            ..Default::default()
        };
        let results = posting_sum_by_channel(&mut conn, &filter_exclude)
            .await
            .unwrap();
        assert!(results.is_empty());
    }
}
