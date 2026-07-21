//! 按天收支汇总

use accounting::error::AccountingError;
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// 某日收支汇总
#[derive(Debug, Clone, PartialEq)]
pub struct DailySummaryItem {
    /// 日期
    pub date: NaiveDate,
    /// 收入（资产类分录正金额之和）
    pub income: Decimal,
    /// 支出（资产类分录负金额绝对值之和）
    pub expense: Decimal,
}

/// 按天收支汇总服务
pub struct DailySummaryService {
    db: SqliteDatabase,
}

impl DailySummaryService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 查询 [from, to] 范围内每个有交易日期的收支汇总，按日期升序返回。
    /// 无交易的日期不出现在结果中。
    pub async fn daily_summary(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DailySummaryItem>, AccountingError> {
        let rows = self
            .db
            .posting_daily_summary(from, to)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| DailySummaryItem {
                date: r.date,
                income: r.income,
                expense: r.expense,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{AccountId, CommodityId, MemberId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn bare_account(parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    fn sample_posting(account_id: AccountId, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
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

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    async fn create_test_member(db: &SqliteDatabase) -> MemberId {
        db.member_get_or_create_by_name("Test", "en").await.unwrap()
    }

    async fn insert_tx(
        db: &SqliteDatabase,
        member_id: MemberId,
        date: NaiveDate,
        postings: &[(AccountId, &str)],
    ) {
        let tx = Transaction {
            id: TransactionId(0),
            date_time: date.and_hms_opt(12, 0, 0).unwrap(),
            description: "Test".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let tx_id = db.transaction_insert(&tx, &[]).await.unwrap();
        for (account_id, amount) in postings {
            let mut p = sample_posting(*account_id, amount);
            p.transaction_id = tx_id;
            db.posting_insert(&p).await.unwrap();
        }
    }

    struct TestAccounts {
        bank: AccountId,
        cash: AccountId,
        food: AccountId,
        salary: AccountId,
    }

    async fn create_accounts(db: &SqliteDatabase) -> TestAccounts {
        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let income_id = db.account_get_by_name("Income").await.unwrap().unwrap().id;

        let bank = db
            .account_create_with_name(&bare_account(Some(assets_id)), "Bank", "en")
            .await
            .unwrap();
        let cash = db
            .account_create_with_name(&bare_account(Some(assets_id)), "Wallet", "en")
            .await
            .unwrap();
        let food = db
            .account_create_with_name(&bare_account(Some(expenses_id)), "Food", "en")
            .await
            .unwrap();
        let salary = db
            .account_create_with_name(&bare_account(Some(income_id)), "Salary", "en")
            .await
            .unwrap();
        TestAccounts {
            bank,
            cash,
            food,
            salary,
        }
    }

    #[tokio::test]
    async fn test_daily_summary_basic_transfer_and_cross_month() {
        let db = setup_db().await;
        let member_id = create_test_member(&db).await;
        let acc = create_accounts(&db).await;
        let service = DailySummaryService::new(db.clone());

        // 1 月 31 日：支出 100
        insert_tx(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            &[(acc.bank, "-100"), (acc.food, "100")],
        )
        .await;
        // 2 月 1 日：收入 500
        insert_tx(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            &[(acc.bank, "500"), (acc.salary, "-500")],
        )
        .await;
        // 2 月 1 日：转账 100（收支各计 100）
        insert_tx(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            &[(acc.bank, "-100"), (acc.cash, "100")],
        )
        .await;

        let items = service
            .daily_summary(
                NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                NaiveDate::from_ymd_opt(2024, 2, 2).unwrap(),
            )
            .await
            .unwrap();

        // 跨月范围：仅返回有交易的两天，1-30 与 2-02 无交易不出现
        assert_eq!(items.len(), 2);

        assert_eq!(items[0].date, NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
        assert_eq!(items[0].income, Decimal::from_str("0").unwrap());
        assert_eq!(items[0].expense, Decimal::from_str("100").unwrap());

        assert_eq!(items[1].date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        // 收入 500 + 转账正端 100
        assert_eq!(items[1].income, Decimal::from_str("600").unwrap());
        // 转账负端 100
        assert_eq!(items[1].expense, Decimal::from_str("100").unwrap());
    }

    #[tokio::test]
    async fn test_daily_summary_empty_range() {
        let db = setup_db().await;
        let service = DailySummaryService::new(db.clone());

        let items = service
            .daily_summary(
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            )
            .await
            .unwrap();

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_daily_summary_excludes_equity_postings() {
        let db = setup_db().await;
        let member_id = create_test_member(&db).await;
        let acc = create_accounts(&db).await;
        let service = DailySummaryService::new(db.clone());

        let opening_id = db
            .account_get_by_name("Equity:OpeningBalances")
            .await
            .unwrap()
            .unwrap()
            .id;
        let cashback_id = db
            .account_get_by_name("Equity:Cashback")
            .await
            .unwrap()
            .unwrap()
            .id;

        let day = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        // 期初余额入账：资产 +1000 / 权益 -1000
        insert_tx(
            &db,
            member_id,
            day,
            &[(acc.bank, "1000"), (opening_id, "-1000")],
        )
        .await;
        // 返现：资产 +10 / 权益 -10
        insert_tx(
            &db,
            member_id,
            day,
            &[(acc.cash, "10"), (cashback_id, "-10")],
        )
        .await;

        let items = service.daily_summary(day, day).await.unwrap();

        // 仅资产侧计入：income = 1000 + 10；权益侧分录不计入，expense 不双计
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].income, Decimal::from_str("1010").unwrap());
        assert_eq!(items[0].expense, Decimal::from_str("0").unwrap());
    }
}
