use accounting::amount;
use accounting::budget::{Budget, BudgetLimit};
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, BudgetId, CommodityId};
use rust_decimal::Decimal;
use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use crate::names::BUDGET_NAMES;

#[derive(FromRow)]
struct BudgetRow {
    id: i64,
    period: i64,
    commodity_id: i64,
}

impl BudgetRow {
    fn into_budget(self) -> Result<Budget, DbError> {
        let period = FinancePeriod::from_i64(self.period).ok_or_else(|| {
            DbError::Database(format!("Invalid budget period value: {}", self.period))
        })?;
        Ok(Budget {
            id: BudgetId(self.id),
            period,
            commodity_id: CommodityId(self.commodity_id),
        })
    }
}

#[derive(FromRow)]
struct BudgetLimitRow {
    budget_id: i64,
    account_id: i64,
    amount: i64,
}

pub async fn budget_create(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
    period: FinancePeriod,
    commodity_id: CommodityId,
    limits: &[(AccountId, Decimal)],
) -> Result<BudgetId, DbError> {
    // Get commodity precision for amount conversion
    let precision: i64 = sqlx::query_scalar("SELECT precision FROM commodities WHERE id = ?1")
        .bind(commodity_id.0)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    // 名字全局唯一（不区分大小写），校验通过才创建
    BUDGET_NAMES
        .ensure_available(conn, None, None, name)
        .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO budgets (period, commodity_id) VALUES (?1, ?2) RETURNING id",
    )
    .bind(period.as_i64())
    .bind(commodity_id.0)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    // 名字语言由调用方传入
    BUDGET_NAMES
        .insert(conn, id, lang, name, false, true)
        .await?;

    for (account_id, amount) in limits {
        let db_amount = amount::to_db_amount(*amount, precision as u8);
        sqlx::query(
            "INSERT INTO budget_limits (budget_id, account_id, amount) VALUES (?1, ?2, ?3)",
        )
        .bind(id)
        .bind(account_id.0)
        .bind(db_amount)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    Ok(BudgetId(id))
}

pub async fn budget_get(
    conn: &mut SqliteConnection,
    id: BudgetId,
) -> Result<Option<Budget>, DbError> {
    let row: Option<BudgetRow> =
        sqlx::query_as("SELECT id, period, commodity_id FROM budgets WHERE id = ?1")
            .bind(id.0)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    match row {
        Some(r) => Ok(Some(r.into_budget()?)),
        None => Ok(None),
    }
}

pub async fn budget_list(conn: &mut SqliteConnection) -> Result<Vec<Budget>, DbError> {
    let rows: Vec<BudgetRow> =
        sqlx::query_as("SELECT id, period, commodity_id FROM budgets ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    rows.into_iter()
        .map(|r| r.into_budget())
        .collect::<Result<Vec<_>, _>>()
}

pub async fn budget_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Budget>, DbError> {
    let row: Option<BudgetRow> =
        sqlx::query_as("SELECT b.id, b.period, b.commodity_id FROM budgets b JOIN budget_names bn ON bn.budget_id = b.id WHERE bn.name = ?1")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    match row {
        Some(r) => Ok(Some(r.into_budget()?)),
        None => Ok(None),
    }
}

pub async fn budget_update(
    conn: &mut SqliteConnection,
    budget_id: BudgetId,
    name: &str,
    lang: &str,
    period: FinancePeriod,
    commodity_id: CommodityId,
    limits: &[(AccountId, Decimal)],
) -> Result<(), DbError> {
    // Get commodity precision for amount conversion
    let precision: i64 = sqlx::query_scalar("SELECT precision FROM commodities WHERE id = ?1")
        .bind(commodity_id.0)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    // Update budget header
    let result = sqlx::query("UPDATE budgets SET period = ?1, commodity_id = ?2 WHERE id = ?3")
        .bind(period.as_i64())
        .bind(commodity_id.0)
        .bind(budget_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DbError::Database(format!(
            "Budget not found: {}",
            budget_id.0
        )));
    }

    // 改名：(budget, lang) 显示名更新为新文本；撞名拒绝
    BUDGET_NAMES
        .rename_display(conn, budget_id.0, None, lang, name)
        .await?;

    // Delete old limits and insert new ones
    sqlx::query("DELETE FROM budget_limits WHERE budget_id = ?1")
        .bind(budget_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    for (account_id, amount) in limits {
        let db_amount = amount::to_db_amount(*amount, precision as u8);
        sqlx::query(
            "INSERT INTO budget_limits (budget_id, account_id, amount) VALUES (?1, ?2, ?3)",
        )
        .bind(budget_id.0)
        .bind(account_id.0)
        .bind(db_amount)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    Ok(())
}

pub async fn budget_list_all_with_limits(
    conn: &mut SqliteConnection,
) -> Result<Vec<(Budget, Vec<BudgetLimit>)>, DbError> {
    let budgets = budget_list(conn).await?;
    let mut result = Vec::new();
    for budget in budgets {
        let limits = budget_get_limits(conn, budget.id).await?;
        result.push((budget, limits));
    }
    Ok(result)
}

pub async fn budget_upsert_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
    period: FinancePeriod,
    commodity_id: CommodityId,
    limits: &[(AccountId, Decimal)],
) -> Result<BudgetId, DbError> {
    // Find existing budget by name via budget_names table
    let existing_id: Option<i64> = sqlx::query_scalar(
        "SELECT b.id FROM budgets b JOIN budget_names bn ON bn.budget_id = b.id WHERE bn.name = ?1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    if let Some(budget_id) = existing_id {
        let budget_id = BudgetId(budget_id);
        budget_update(conn, budget_id, name, lang, period, commodity_id, limits).await?;
        Ok(budget_id)
    } else {
        budget_create(conn, name, lang, period, commodity_id, limits).await
    }
}

pub async fn budget_delete(conn: &mut SqliteConnection, id: BudgetId) -> Result<(), DbError> {
    let result = sqlx::query("DELETE FROM budgets WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DbError::Database(format!("Budget not found: {}", id.0)));
    }
    Ok(())
}

pub async fn budget_get_limits(
    conn: &mut SqliteConnection,
    budget_id: BudgetId,
) -> Result<Vec<BudgetLimit>, DbError> {
    // Get commodity precision
    let precision: Option<i64> = sqlx::query_scalar(
        "SELECT c.precision FROM commodities c JOIN budgets b ON b.commodity_id = c.id WHERE b.id = ?1",
    )
    .bind(budget_id.0)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    let precision = match precision {
        Some(p) => p,
        None => return Ok(vec![]), // Budget not found, return empty limits
    };

    let rows: Vec<BudgetLimitRow> = sqlx::query_as(
        "SELECT budget_id, account_id, amount FROM budget_limits WHERE budget_id = ?1",
    )
    .bind(budget_id.0)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    rows.into_iter()
        .map(|r| {
            Ok(BudgetLimit {
                budget_id: BudgetId(r.budget_id),
                account_id: AccountId(r.account_id),
                amount: amount::from_db_amount(r.amount, precision as u8),
            })
        })
        .collect::<Result<Vec<_>, DbError>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::account::{account_create_with_closure, account_get_by_name};
    use crate::repo::posting::{posting_insert, sum_by_account_with_descendants};
    use accounting::account::Account;
    use accounting::id::TagId;
    use accounting::posting::Posting;
    use chrono::NaiveDate;
    use sqlx::{Connection, SqliteConnection};
    use std::str::FromStr;

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    async fn insert_expense_account(conn: &mut SqliteConnection, name: &str) -> AccountId {
        let root_id = account_get_by_name(conn, "Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let account = Account {
            id: AccountId(0),
            parent_id: Some(root_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = account_create_with_closure(conn, &account).await.unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(id.0)
        .bind(name)
        .execute(&mut *conn)
        .await
        .unwrap();
        id
    }

    async fn ensure_test_member(conn: &mut SqliteConnection) -> i64 {
        // Check if member already exists via member_names
        if let Some(id) = sqlx::query_scalar::<_, i64>(
            "SELECT member_id FROM member_names WHERE name = ?1 AND lang = 'en' LIMIT 1",
        )
        .bind("Test Member")
        .fetch_optional(&mut *conn)
        .await
        .unwrap()
        {
            return id;
        }
        let id: i64 = sqlx::query_scalar("INSERT INTO members DEFAULT VALUES RETURNING id")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO member_names (member_id, lang, name, is_system, is_display)
             VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(id)
        .bind("Test Member")
        .execute(&mut *conn)
        .await
        .unwrap();
        id
    }

    async fn insert_transaction_at(
        conn: &mut SqliteConnection,
        date: &str,
    ) -> accounting::id::TransactionId {
        let member_id = ensure_test_member(conn).await;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO transactions (date_time, description, member_id) VALUES (?1, 'test', ?2) RETURNING id",
        )
        .bind(date)
        .bind(member_id)
        .fetch_one(conn)
        .await
        .unwrap();
        accounting::id::TransactionId(id)
    }

    fn sample_posting(
        tx_id: accounting::id::TransactionId,
        account_id: AccountId,
        amount: &str,
    ) -> Posting {
        Posting {
            id: accounting::id::PostingId(0),
            transaction_id: tx_id,
            account_id,
            commodity_id: CommodityId(1),
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    #[tokio::test]
    async fn test_budget_create_and_get() {
        let mut conn = setup().await;
        let id = budget_create(
            &mut conn,
            "Monthly Life",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[
                (AccountId(1), Decimal::from_str("2000").unwrap()),
                (AccountId(2), Decimal::from_str("500").unwrap()),
            ],
        )
        .await
        .unwrap();

        let budget = budget_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(budget.period, FinancePeriod::Monthly);
        assert_eq!(budget.commodity_id, CommodityId(1));

        let limits = budget_get_limits(&mut conn, id).await.unwrap();
        assert_eq!(limits.len(), 2);
    }

    #[tokio::test]
    async fn test_budget_create_duplicate_name_rejected() {
        let mut conn = setup().await;
        budget_create(
            &mut conn,
            "Monthly Life",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("2000").unwrap())],
        )
        .await
        .unwrap();

        // 同名（NOCASE）再次创建 → 拒绝
        let result = budget_create(
            &mut conn,
            "monthly life",
            "en",
            FinancePeriod::Yearly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("100").unwrap())],
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));
    }

    #[tokio::test]
    async fn test_budget_list() {
        let mut conn = setup().await;
        budget_create(
            &mut conn,
            "Budget A",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("1000").unwrap())],
        )
        .await
        .unwrap();
        budget_create(
            &mut conn,
            "Budget B",
            "en",
            FinancePeriod::Yearly,
            CommodityId(1),
            &[(AccountId(2), Decimal::from_str("20000").unwrap())],
        )
        .await
        .unwrap();

        let list = budget_list(&mut conn).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_budget_update() {
        let mut conn = setup().await;
        let id = budget_create(
            &mut conn,
            "Old Name",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("1000").unwrap())],
        )
        .await
        .unwrap();

        budget_update(
            &mut conn,
            id,
            "New Name",
            "en",
            FinancePeriod::Yearly,
            CommodityId(1),
            &[
                (AccountId(1), Decimal::from_str("3000").unwrap()),
                (AccountId(2), Decimal::from_str("5000").unwrap()),
            ],
        )
        .await
        .unwrap();

        let budget = budget_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(budget.period, FinancePeriod::Yearly);

        let limits = budget_get_limits(&mut conn, id).await.unwrap();
        assert_eq!(limits.len(), 2);

        // 改名真正生效：新名字命中，旧名字不再命中
        assert!(
            budget_get_by_name(&mut conn, "New Name")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            budget_get_by_name(&mut conn, "Old Name")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_budget_update_rename_collision_rejected() {
        let mut conn = setup().await;
        let id_a = budget_create(
            &mut conn,
            "Budget A",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("1000").unwrap())],
        )
        .await
        .unwrap();
        budget_create(
            &mut conn,
            "Budget B",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("1000").unwrap())],
        )
        .await
        .unwrap();

        // 改名为另一预算已占用的名字 → 拒绝
        assert!(
            budget_update(
                &mut conn,
                id_a,
                "budget b",
                "en",
                FinancePeriod::Monthly,
                CommodityId(1),
                &[(AccountId(1), Decimal::from_str("1000").unwrap())],
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_budget_delete_cascade() {
        let mut conn = setup().await;
        let id = budget_create(
            &mut conn,
            "To Delete",
            "en",
            FinancePeriod::Monthly,
            CommodityId(1),
            &[(AccountId(1), Decimal::from_str("1000").unwrap())],
        )
        .await
        .unwrap();

        // Verify limits exist
        let limits = budget_get_limits(&mut conn, id).await.unwrap();
        assert!(!limits.is_empty());

        // Delete
        budget_delete(&mut conn, id).await.unwrap();

        // Budget gone
        assert!(budget_get(&mut conn, id).await.unwrap().is_none());

        // Limits cascaded
        let limits = budget_get_limits(&mut conn, id).await.unwrap();
        assert!(limits.is_empty());
    }

    #[tokio::test]
    async fn test_sum_by_account_with_descendants() {
        let mut conn = setup().await;
        let food = insert_expense_account(&mut conn, "Food").await;
        let assets = account_get_by_name(&mut conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;

        // Insert transactions
        let tx1 = insert_transaction_at(&mut conn, "2024-06-10 00:00:00").await;
        let tx2 = insert_transaction_at(&mut conn, "2024-06-15 00:00:00").await;

        let p1 = sample_posting(tx1, food, "-100.00");
        let p2 = sample_posting(tx1, assets, "100.00");
        let p3 = sample_posting(tx2, food, "-50.00");
        let p4 = sample_posting(tx2, assets, "50.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();
        posting_insert(&mut conn, &p3).await.unwrap();
        posting_insert(&mut conn, &p4).await.unwrap();

        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        // Query by Expenses (parent) — should aggregate Food's postings
        let expenses_id = account_get_by_name(&mut conn, "Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let results = sum_by_account_with_descendants(
            &mut conn,
            &[expenses_id],
            start,
            end,
            &[],
            CommodityId(1),
        )
        .await
        .unwrap();

        let total: Decimal = results.iter().map(|(_, a)| *a).sum();
        assert_eq!(total, Decimal::from_str("-150.00").unwrap());
    }

    #[tokio::test]
    async fn test_sum_by_account_excludes_budget_tag() {
        let mut conn = setup().await;
        let food = insert_expense_account(&mut conn, "Snacks").await;
        let assets = account_get_by_name(&mut conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;

        // Create "exclude-from-budget" tag
        let exclude_tag_id: i64 =
            sqlx::query_scalar("SELECT t.id FROM tags t JOIN tag_names tn ON tn.tag_id = t.id WHERE tn.name = 'exclude-from-budget'")
                .fetch_one(&mut conn)
                .await
                .unwrap();

        // Transaction 1: normal
        let tx1 = insert_transaction_at(&mut conn, "2024-07-01 00:00:00").await;
        let p1 = sample_posting(tx1, food, "-100.00");
        let p2 = sample_posting(tx1, assets, "100.00");
        posting_insert(&mut conn, &p1).await.unwrap();
        posting_insert(&mut conn, &p2).await.unwrap();

        // Transaction 2: tagged with exclude-from-budget
        let tx2 = insert_transaction_at(&mut conn, "2024-07-05 00:00:00").await;
        let p3 = sample_posting(tx2, food, "-200.00");
        let p4 = sample_posting(tx2, assets, "200.00");
        posting_insert(&mut conn, &p3).await.unwrap();
        posting_insert(&mut conn, &p4).await.unwrap();

        sqlx::query("INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?1, ?2)")
            .bind(tx2.0)
            .bind(exclude_tag_id)
            .execute(&mut conn)
            .await
            .unwrap();

        let start = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 7, 31).unwrap();

        let expenses_id = account_get_by_name(&mut conn, "Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;

        // Without exclusion: -300
        let results = sum_by_account_with_descendants(
            &mut conn,
            &[expenses_id],
            start,
            end,
            &[],
            CommodityId(1),
        )
        .await
        .unwrap();
        let total: Decimal = results.iter().map(|(_, a)| *a).sum();
        assert_eq!(total, Decimal::from_str("-300.00").unwrap());

        // With exclusion: -100 (tx2 excluded)
        let results = sum_by_account_with_descendants(
            &mut conn,
            &[expenses_id],
            start,
            end,
            &[TagId(exclude_tag_id)],
            CommodityId(1),
        )
        .await
        .unwrap();
        let total: Decimal = results.iter().map(|(_, a)| *a).sum();
        assert_eq!(total, Decimal::from_str("-100.00").unwrap());
    }
}
