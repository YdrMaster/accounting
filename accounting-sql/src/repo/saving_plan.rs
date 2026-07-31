use accounting::amount;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId, SavingPlanId};
use accounting::saving_plan::{SavingPlan, SavingPlanAccount};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Connection, FromRow, SqliteConnection};

use crate::error::DbError;
use crate::names::SAVING_PLAN_NAMES;

/// deadline 以 TEXT 'YYYY-MM-DD' 存储
fn format_deadline(deadline: Option<NaiveDate>) -> Option<String> {
    deadline.map(|d| d.format("%Y-%m-%d").to_string())
}

fn parse_deadline(raw: &str) -> Result<NaiveDate, DbError> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map_err(|e| DbError::Database(format!("Invalid deadline value: {} ({})", raw, e)))
}

#[derive(FromRow)]
struct SavingPlanRow {
    id: i64,
    period: Option<i64>,
    deadline: Option<String>,
    commodity_id: i64,
    target_amount: i64,
}

impl SavingPlanRow {
    fn into_saving_plan(self, precision: u8) -> Result<SavingPlan, DbError> {
        let period = self
            .period
            .map(|v| {
                FinancePeriod::from_i64(v).ok_or_else(|| {
                    DbError::Database(format!("Invalid saving plan period value: {}", v))
                })
            })
            .transpose()?;
        Ok(SavingPlan {
            id: SavingPlanId(self.id),
            period,
            deadline: self.deadline.map(|d| parse_deadline(&d)).transpose()?,
            commodity_id: CommodityId(self.commodity_id),
            target_amount: amount::from_db_amount(self.target_amount, precision),
        })
    }
}

async fn get_precision(
    conn: &mut SqliteConnection,
    commodity_id: CommodityId,
) -> Result<u8, DbError> {
    let precision: i64 = sqlx::query_scalar("SELECT precision FROM commodities WHERE id = ?1")
        .bind(commodity_id.0)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(precision as u8)
}

#[allow(clippy::too_many_arguments)]
pub async fn saving_plan_create(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
    period: Option<FinancePeriod>,
    deadline: Option<NaiveDate>,
    commodity_id: CommodityId,
    target_amount: Decimal,
    account_ids: &[AccountId],
) -> Result<SavingPlanId, DbError> {
    // 显式事务：header、名字、账户关联要么全部落库要么全部回滚（失败随 drop 回滚）
    let mut tx = conn
        .begin()
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let precision = get_precision(&mut tx, commodity_id).await?;

    // 名字全局唯一（不区分大小写），校验通过才创建
    SAVING_PLAN_NAMES
        .ensure_available(&mut tx, None, None, name)
        .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO saving_plans (period, deadline, commodity_id, target_amount)
         VALUES (?1, ?2, ?3, ?4) RETURNING id",
    )
    .bind(period.map(|p| p.as_i64()))
    .bind(format_deadline(deadline))
    .bind(commodity_id.0)
    .bind(amount::to_db_amount(target_amount, precision))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    // 名字语言由调用方传入
    SAVING_PLAN_NAMES
        .insert(&mut tx, id, lang, name, false, true)
        .await?;

    for account_id in account_ids {
        sqlx::query("INSERT INTO saving_plan_accounts (plan_id, account_id) VALUES (?1, ?2)")
            .bind(id)
            .bind(account_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(SavingPlanId(id))
}

pub async fn saving_plan_get(
    conn: &mut SqliteConnection,
    id: SavingPlanId,
) -> Result<Option<SavingPlan>, DbError> {
    let row: Option<SavingPlanRow> = sqlx::query_as(
        "SELECT id, period, deadline, commodity_id, target_amount FROM saving_plans WHERE id = ?1",
    )
    .bind(id.0)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    match row {
        Some(r) => {
            let precision = get_precision(conn, CommodityId(r.commodity_id)).await?;
            Ok(Some(r.into_saving_plan(precision)?))
        }
        None => Ok(None),
    }
}

pub async fn saving_plan_list(conn: &mut SqliteConnection) -> Result<Vec<SavingPlan>, DbError> {
    let rows: Vec<SavingPlanRow> = sqlx::query_as(
        "SELECT id, period, deadline, commodity_id, target_amount FROM saving_plans ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let precision = get_precision(&mut *conn, CommodityId(row.commodity_id)).await?;
        result.push(row.into_saving_plan(precision)?);
    }
    Ok(result)
}

pub async fn saving_plan_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<SavingPlan>, DbError> {
    let row: Option<SavingPlanRow> = sqlx::query_as(
        "SELECT p.id, p.period, p.deadline, p.commodity_id, p.target_amount
         FROM saving_plans p
         JOIN saving_plan_names pn ON pn.plan_id = p.id
         WHERE pn.name = ?1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    match row {
        Some(r) => {
            let precision = get_precision(conn, CommodityId(r.commodity_id)).await?;
            Ok(Some(r.into_saving_plan(precision)?))
        }
        None => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn saving_plan_update(
    conn: &mut SqliteConnection,
    plan_id: SavingPlanId,
    name: &str,
    lang: &str,
    period: Option<FinancePeriod>,
    deadline: Option<NaiveDate>,
    commodity_id: CommodityId,
    target_amount: Decimal,
    account_ids: &[AccountId],
) -> Result<(), DbError> {
    // 显式事务：header、改名、账户关联替换原子生效（失败随 drop 回滚）
    let mut tx = conn
        .begin()
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let precision = get_precision(&mut tx, commodity_id).await?;

    // Update plan header
    let result = sqlx::query(
        "UPDATE saving_plans SET period = ?1, deadline = ?2, commodity_id = ?3, target_amount = ?4
         WHERE id = ?5",
    )
    .bind(period.map(|p| p.as_i64()))
    .bind(format_deadline(deadline))
    .bind(commodity_id.0)
    .bind(amount::to_db_amount(target_amount, precision))
    .bind(plan_id.0)
    .execute(&mut *tx)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DbError::Database(format!(
            "Saving plan not found: {}",
            plan_id.0
        )));
    }

    // 改名：(plan, lang) 显示名更新为新文本；撞名拒绝
    SAVING_PLAN_NAMES
        .rename_display(&mut tx, plan_id.0, None, lang, name)
        .await?;

    // Delete old account links and insert new ones
    sqlx::query("DELETE FROM saving_plan_accounts WHERE plan_id = ?1")
        .bind(plan_id.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    for account_id in account_ids {
        sqlx::query("INSERT INTO saving_plan_accounts (plan_id, account_id) VALUES (?1, ?2)")
            .bind(plan_id.0)
            .bind(account_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn saving_plan_upsert_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    lang: &str,
    period: Option<FinancePeriod>,
    deadline: Option<NaiveDate>,
    commodity_id: CommodityId,
    target_amount: Decimal,
    account_ids: &[AccountId],
) -> Result<SavingPlanId, DbError> {
    // Find existing plan by name via saving_plan_names table
    let existing_id: Option<i64> = sqlx::query_scalar(
        "SELECT p.id FROM saving_plans p
         JOIN saving_plan_names pn ON pn.plan_id = p.id
         WHERE pn.name = ?1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    if let Some(plan_id) = existing_id {
        let plan_id = SavingPlanId(plan_id);
        saving_plan_update(
            conn,
            plan_id,
            name,
            lang,
            period,
            deadline,
            commodity_id,
            target_amount,
            account_ids,
        )
        .await?;
        Ok(plan_id)
    } else {
        saving_plan_create(
            conn,
            name,
            lang,
            period,
            deadline,
            commodity_id,
            target_amount,
            account_ids,
        )
        .await
    }
}

pub async fn saving_plan_delete(
    conn: &mut SqliteConnection,
    id: SavingPlanId,
) -> Result<(), DbError> {
    let result = sqlx::query("DELETE FROM saving_plans WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DbError::Database(format!(
            "Saving plan not found: {}",
            id.0
        )));
    }
    Ok(())
}

pub async fn saving_plan_get_accounts(
    conn: &mut SqliteConnection,
    plan_id: SavingPlanId,
) -> Result<Vec<SavingPlanAccount>, DbError> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT plan_id, account_id FROM saving_plan_accounts WHERE plan_id = ?1 ORDER BY account_id",
    )
    .bind(plan_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(plan_id, account_id)| SavingPlanAccount {
            plan_id: SavingPlanId(plan_id),
            account_id: AccountId(account_id),
        })
        .collect())
}

pub async fn saving_plan_list_all_with_accounts(
    conn: &mut SqliteConnection,
) -> Result<Vec<(SavingPlan, Vec<SavingPlanAccount>)>, DbError> {
    let plans = saving_plan_list(conn).await?;
    let mut result = Vec::new();
    for plan in plans {
        let accounts = saving_plan_get_accounts(conn, plan.id).await?;
        result.push((plan, accounts));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::account::{account_create_with_closure, account_get_by_name};
    use accounting::account::Account;
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

    async fn insert_asset_account(conn: &mut SqliteConnection, name: &str) -> AccountId {
        let root_id = account_get_by_name(conn, "Assets")
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

    #[tokio::test]
    async fn test_saving_plan_create_and_get() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "Alipay").await;
        let a2 = insert_asset_account(&mut conn, "WeChat").await;
        let deadline = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();

        let id = saving_plan_create(
            &mut conn,
            "Trip Fund",
            "en",
            None,
            Some(deadline),
            CommodityId(1),
            Decimal::from_str("5000").unwrap(),
            &[a1, a2],
        )
        .await
        .unwrap();

        let plan = saving_plan_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(plan.period, None);
        assert_eq!(plan.deadline, Some(deadline));
        assert_eq!(plan.commodity_id, CommodityId(1));
        assert_eq!(plan.target_amount, Decimal::from_str("5000").unwrap());

        // 账户集合读写往返（金额按币种精度缩放）
        let accounts = saving_plan_get_accounts(&mut conn, id).await.unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().any(|a| a.account_id == a1));
        assert!(accounts.iter().any(|a| a.account_id == a2));
        assert!(accounts.iter().all(|a| a.plan_id == id));

        // 金额精度往返（非整百值）
        let id2 = saving_plan_create(
            &mut conn,
            "Rent",
            "en",
            Some(FinancePeriod::Monthly),
            None,
            CommodityId(1),
            Decimal::from_str("1234.56").unwrap(),
            &[a1],
        )
        .await
        .unwrap();
        let plan2 = saving_plan_get(&mut conn, id2).await.unwrap().unwrap();
        assert_eq!(plan2.period, Some(FinancePeriod::Monthly));
        assert_eq!(plan2.deadline, None);
        assert_eq!(plan2.target_amount, Decimal::from_str("1234.56").unwrap());
    }

    #[tokio::test]
    async fn test_saving_plan_create_duplicate_name_rejected() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "Card").await;
        saving_plan_create(
            &mut conn,
            "Trip Fund",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("5000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        let result = saving_plan_create(
            &mut conn,
            "trip fund",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("100").unwrap(),
            &[a1],
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));
    }

    #[tokio::test]
    async fn test_saving_plan_list() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankA").await;
        saving_plan_create(
            &mut conn,
            "Plan A",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("1000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();
        saving_plan_create(
            &mut conn,
            "Plan B",
            "en",
            Some(FinancePeriod::Yearly),
            None,
            CommodityId(1),
            Decimal::from_str("2000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        let list = saving_plan_list(&mut conn).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].period, None);
        assert_eq!(list[1].period, Some(FinancePeriod::Yearly));
    }

    #[tokio::test]
    async fn test_saving_plan_get_by_name() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankB").await;
        saving_plan_create(
            &mut conn,
            "Emergency",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("10000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        assert!(
            saving_plan_get_by_name(&mut conn, "Emergency")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            saving_plan_get_by_name(&mut conn, "nonexistent")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_saving_plan_update() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankC").await;
        let a2 = insert_asset_account(&mut conn, "BankD").await;
        let id = saving_plan_create(
            &mut conn,
            "Old Plan",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("1000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        let deadline = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        saving_plan_update(
            &mut conn,
            id,
            "New Plan",
            "en",
            Some(FinancePeriod::Monthly),
            Some(deadline),
            CommodityId(1),
            Decimal::from_str("6000").unwrap(),
            &[a1, a2],
        )
        .await
        .unwrap();

        let plan = saving_plan_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(plan.period, Some(FinancePeriod::Monthly));
        assert_eq!(plan.deadline, Some(deadline));
        assert_eq!(plan.target_amount, Decimal::from_str("6000").unwrap());

        let accounts = saving_plan_get_accounts(&mut conn, id).await.unwrap();
        assert_eq!(accounts.len(), 2);

        // 改名真正生效
        assert!(
            saving_plan_get_by_name(&mut conn, "New Plan")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            saving_plan_get_by_name(&mut conn, "Old Plan")
                .await
                .unwrap()
                .is_none()
        );

        // 更新不存在的计划 → 报错
        assert!(
            saving_plan_update(
                &mut conn,
                SavingPlanId(999),
                "X",
                "en",
                None,
                None,
                CommodityId(1),
                Decimal::from_str("1").unwrap(),
                &[a1],
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_saving_plan_update_rename_collision_rolls_back() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankX").await;
        let a2 = insert_asset_account(&mut conn, "BankY").await;
        let deadline_b = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        saving_plan_create(
            &mut conn,
            "Plan A",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("100").unwrap(),
            &[a1],
        )
        .await
        .unwrap();
        let id_b = saving_plan_create(
            &mut conn,
            "Plan B",
            "en",
            Some(FinancePeriod::Monthly),
            Some(deadline_b),
            CommodityId(1),
            Decimal::from_str("2000").unwrap(),
            &[a1, a2],
        )
        .await
        .unwrap();

        // 改名撞名失败 → 整个 update 回滚：header 与账户关联保持旧值
        let result = saving_plan_update(
            &mut conn,
            id_b,
            "plan a",
            "en",
            Some(FinancePeriod::Yearly),
            None,
            CommodityId(1),
            Decimal::from_str("1").unwrap(),
            &[a1],
        )
        .await;
        assert!(result.is_err());

        let b = saving_plan_get(&mut conn, id_b).await.unwrap().unwrap();
        assert_eq!(b.period, Some(FinancePeriod::Monthly));
        assert_eq!(b.deadline, Some(deadline_b));
        assert_eq!(b.commodity_id, CommodityId(1));
        assert_eq!(b.target_amount, Decimal::from_str("2000").unwrap());
        assert_eq!(
            saving_plan_get_accounts(&mut conn, id_b)
                .await
                .unwrap()
                .len(),
            2
        );
        // 名字也未变
        assert!(
            saving_plan_get_by_name(&mut conn, "Plan B")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_saving_plan_upsert_by_name() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankE").await;

        let id = saving_plan_upsert_by_name(
            &mut conn,
            "Upsert Plan",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("100").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        let id2 = saving_plan_upsert_by_name(
            &mut conn,
            "Upsert Plan",
            "en",
            Some(FinancePeriod::Daily),
            None,
            CommodityId(1),
            Decimal::from_str("200").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        assert_eq!(id, id2, "同名 upsert 应更新而非新建");
        let plan = saving_plan_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(plan.period, Some(FinancePeriod::Daily));
        assert_eq!(plan.target_amount, Decimal::from_str("200").unwrap());
        assert_eq!(saving_plan_list(&mut conn).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_saving_plan_delete_cascade() {
        let mut conn = setup().await;
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
            .unwrap();
        let a1 = insert_asset_account(&mut conn, "BankF").await;
        let id = saving_plan_create(
            &mut conn,
            "To Delete",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("1000").unwrap(),
            &[a1],
        )
        .await
        .unwrap();

        assert!(
            !saving_plan_get_accounts(&mut conn, id)
                .await
                .unwrap()
                .is_empty()
        );

        saving_plan_delete(&mut conn, id).await.unwrap();

        assert!(saving_plan_get(&mut conn, id).await.unwrap().is_none());
        // 关联表与名字表级联删除
        assert!(
            saving_plan_get_accounts(&mut conn, id)
                .await
                .unwrap()
                .is_empty()
        );
        let names: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM saving_plan_names WHERE plan_id = ?1")
                .bind(id.0)
                .fetch_one(&mut conn)
                .await
                .unwrap();
        assert_eq!(names, 0);

        // 删除不存在的计划 → 报错
        assert!(saving_plan_delete(&mut conn, id).await.is_err());
    }

    #[tokio::test]
    async fn test_saving_plan_list_all_with_accounts() {
        let mut conn = setup().await;
        let a1 = insert_asset_account(&mut conn, "BankG").await;
        let a2 = insert_asset_account(&mut conn, "BankH").await;
        saving_plan_create(
            &mut conn,
            "P1",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("100").unwrap(),
            &[a1],
        )
        .await
        .unwrap();
        saving_plan_create(
            &mut conn,
            "P2",
            "en",
            None,
            None,
            CommodityId(1),
            Decimal::from_str("200").unwrap(),
            &[a1, a2],
        )
        .await
        .unwrap();

        let all = saving_plan_list_all_with_accounts(&mut conn).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].1.len(), 1);
        assert_eq!(all[1].1.len(), 2);
    }
}
