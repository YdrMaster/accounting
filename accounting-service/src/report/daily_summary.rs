//! 按天收支汇总（资产根「逐交易净额」口径）

use accounting::error::AccountingError;
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// 某日收支汇总
#[derive(Debug, Clone, PartialEq)]
pub struct DailySummaryItem {
    /// 日期
    pub date: NaiveDate,
    /// 收入（每笔交易的资产根分录净额为正之和）
    pub income: Decimal,
    /// 支出（每笔交易的资产根分录净额为负之绝对值之和）
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
    /// 口径为资产根分录「逐交易净额」：纯转账（净额 0）不计入，
    /// 因此某日所有交易净额均为 0 时该日不出现在结果中；无交易的日期同样不返回。
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
        // 2 月 1 日：转账 100（资产净额 0，不计入收支）
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
        // 收入 500；同日转账净额 0 不计入
        assert_eq!(items[1].income, Decimal::from_str("500").unwrap());
        // 转账净额 0，不产生支出
        assert_eq!(items[1].expense, Decimal::from_str("0").unwrap());
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

    #[tokio::test]
    async fn test_daily_summary_per_transaction_net_transfer_and_fee() {
        let db = setup_db().await;
        let member_id = create_test_member(&db).await;
        let acc = create_accounts(&db).await;
        let service = DailySummaryService::new(db.clone());

        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let fee_id = db
            .account_create_with_name(&bare_account(Some(expenses_id)), "Fee", "en")
            .await
            .unwrap();
        let opening_id = db
            .account_get_by_name("Equity:OpeningBalances")
            .await
            .unwrap()
            .unwrap()
            .id;

        // 3-01 纯转账 50（资产 -50 / +50，净额 0）→ 当日不返回
        let d1 = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        insert_tx(&db, member_id, d1, &[(acc.bank, "-50"), (acc.cash, "50")]).await;

        // 3-02 含手续费转账：资产 -1010 / +1000，手续费 +10 → 资产净额 -10 → expense 10
        let d2 = NaiveDate::from_ymd_opt(2024, 3, 2).unwrap();
        insert_tx(
            &db,
            member_id,
            d2,
            &[(acc.bank, "-1010"), (acc.cash, "1000"), (fee_id, "10")],
        )
        .await;

        // 3-03 同日支出 100 + 纯转账 50 → 只计支出 100
        let d3 = NaiveDate::from_ymd_opt(2024, 3, 3).unwrap();
        insert_tx(&db, member_id, d3, &[(acc.bank, "-100"), (acc.food, "100")]).await;
        insert_tx(&db, member_id, d3, &[(acc.bank, "-50"), (acc.cash, "50")]).await;

        // 3-04 期初余额：资产 +1000 / Equity -1000 → income 1000，权益侧分录不计入
        let d4 = NaiveDate::from_ymd_opt(2024, 3, 4).unwrap();
        insert_tx(
            &db,
            member_id,
            d4,
            &[(acc.bank, "1000"), (opening_id, "-1000")],
        )
        .await;

        let items = service.daily_summary(d1, d4).await.unwrap();

        // 3-01 纯转账日不出现；其余三天按日升序
        assert_eq!(items.len(), 3);

        assert_eq!(items[0].date, d2);
        assert_eq!(items[0].income, Decimal::from_str("0").unwrap());
        assert_eq!(items[0].expense, Decimal::from_str("10").unwrap());

        assert_eq!(items[1].date, d3);
        assert_eq!(items[1].income, Decimal::from_str("0").unwrap());
        assert_eq!(items[1].expense, Decimal::from_str("100").unwrap());

        assert_eq!(items[2].date, d4);
        assert_eq!(items[2].income, Decimal::from_str("1000").unwrap());
        assert_eq!(items[2].expense, Decimal::from_str("0").unwrap());
    }
}
