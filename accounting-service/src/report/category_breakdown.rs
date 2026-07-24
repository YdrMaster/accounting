//! 收支分类明细

use accounting::account::Account;
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId, TagId};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 收支分类明细项
#[derive(Debug, Clone)]
pub struct CategoryBreakdownItem {
    /// 账户信息（含每一层祖先的汇总）
    pub account: Account,
    /// 周期内汇总金额（绝对值）
    pub amount: Decimal,
}

/// 收支分类明细报表
#[derive(Debug, Clone)]
pub struct CategoryBreakdownReport {
    /// 周期起始日期
    pub period_start: NaiveDate,
    /// 周期结束日期
    pub period_end: NaiveDate,
    /// 收入分类明细（Income 根下各层级汇总）
    pub income: Vec<CategoryBreakdownItem>,
    /// 支出分类明细（Expenses 根下各层级汇总）
    pub expense: Vec<CategoryBreakdownItem>,
}

/// 收支分类明细服务
pub struct CategoryBreakdownService {
    db: SqliteDatabase,
}

impl CategoryBreakdownService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 生成收支分类明细
    ///
    /// 分别汇总 Income 与 Expenses 根下每个账户（含各层祖先）在周期内的金额，
    /// 金额取绝对值，排除"不计预算"标签的分录。
    pub async fn category_breakdown(
        &self,
        date: NaiveDate,
        period: FinancePeriod,
        commodity_id: CommodityId,
    ) -> Result<CategoryBreakdownReport, AccountingError> {
        let (period_start, period_end) = period.period_range(date);
        let exclude_tag_ids = self.get_exclude_budget_tag_ids().await?;

        let income = self
            .breakdown_root("Income", period_start, period_end, &exclude_tag_ids, commodity_id)
            .await?;
        let expense = self
            .breakdown_root("Expenses", period_start, period_end, &exclude_tag_ids, commodity_id)
            .await?;

        Ok(CategoryBreakdownReport {
            period_start,
            period_end,
            income,
            expense,
        })
    }

    async fn breakdown_root(
        &self,
        root_name: &str,
        start: NaiveDate,
        end: NaiveDate,
        exclude_tag_ids: &[TagId],
        commodity_id: CommodityId,
    ) -> Result<Vec<CategoryBreakdownItem>, AccountingError> {
        // 收集该根下的所有账户（含根本身）；聚合查询按传入的祖先 ID 分组，
        // 必须传入全部层级才能拿到每一层的汇总
        let accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let mut root_accounts: Vec<Account> = Vec::new();
        for account in accounts {
            // 根账户类型判定为内部逻辑，固定按 en 解析
            let found = self
                .db
                .account_find_root_name(account.id, accounting::name::lang::EN)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if found == root_name {
                root_accounts.push(account);
            }
        }
        if root_accounts.is_empty() {
            return Ok(Vec::new());
        }

        let account_ids: Vec<AccountId> = root_accounts.iter().map(|a| a.id).collect();
        let sums = self
            .db
            .sum_by_account_with_descendants(
                &account_ids,
                start,
                end,
                exclude_tag_ids,
                commodity_id,
            )
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let account_map: HashMap<AccountId, Account> =
            root_accounts.into_iter().map(|a| (a.id, a)).collect();

        let mut items: Vec<CategoryBreakdownItem> = sums
            .into_iter()
            .filter_map(|(account_id, amount)| {
                account_map.get(&account_id).map(|account| CategoryBreakdownItem {
                    account: account.clone(),
                    amount: amount.abs(),
                })
            })
            .collect();
        items.sort_by_key(|i| i.account.id);
        Ok(items)
    }

    /// 查找"不计预算"系统标签 ID（与资金流量表口径一致）
    async fn get_exclude_budget_tag_ids(&self) -> Result<Vec<TagId>, AccountingError> {
        let tag = self
            .db
            .tag_get_by_name("exclude-from-budget")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(tag.map(|t| vec![t.id]).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{MemberId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
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

    async fn insert_tx_with_postings(
        db: &SqliteDatabase,
        member_id: MemberId,
        date: NaiveDate,
        postings: &[(AccountId, &str)],
        tags: &[TagId],
    ) {
        let tx = Transaction {
            id: TransactionId(0),
            date_time: date.and_hms_opt(0, 0, 0).unwrap(),
            description: "test".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let tx_id = db.transaction_insert(&tx, tags).await.unwrap();
        for (account_id, amount) in postings {
            let mut p = sample_posting(*account_id, amount);
            p.transaction_id = tx_id;
            db.posting_insert(&p).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_multi_level_aggregation() {
        let db = setup_db().await;
        let service = CategoryBreakdownService::new(db.clone());
        let member_id = create_test_member(&db).await;

        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let food_id = db
            .account_create_with_name(&bare_account(Some(expenses_id)), "餐饮", "en")
            .await
            .unwrap();
        let takeout_id = db
            .account_create_with_name(&bare_account(Some(food_id)), "外卖", "en")
            .await
            .unwrap();

        insert_tx_with_postings(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            &[(takeout_id, "500")],
            &[],
        )
        .await;

        let report = service
            .category_breakdown(
                NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                FinancePeriod::Monthly,
                CommodityId(1),
            )
            .await
            .unwrap();

        // 外卖、餐饮、Expenses 三层各一行
        assert_eq!(report.expense.len(), 3);
        let amounts: HashMap<AccountId, Decimal> = report
            .expense
            .iter()
            .map(|i| (i.account.id, i.amount))
            .collect();
        let expected = Decimal::from_str("500").unwrap();
        assert_eq!(amounts[&takeout_id], expected);
        assert_eq!(amounts[&food_id], expected);
        assert_eq!(amounts[&expenses_id], expected);
        assert!(report.income.is_empty());
    }

    #[tokio::test]
    async fn test_income_negative_normalized() {
        let db = setup_db().await;
        let service = CategoryBreakdownService::new(db.clone());
        let member_id = create_test_member(&db).await;

        let income_id = db.account_get_by_name("Income").await.unwrap().unwrap().id;
        let salary_id = db
            .account_create_with_name(&bare_account(Some(income_id)), "工资", "en")
            .await
            .unwrap();

        insert_tx_with_postings(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            &[(salary_id, "-15000")],
            &[],
        )
        .await;

        let report = service
            .category_breakdown(
                NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                FinancePeriod::Monthly,
                CommodityId(1),
            )
            .await
            .unwrap();

        let salary = report
            .income
            .iter()
            .find(|i| i.account.id == salary_id)
            .unwrap();
        assert_eq!(salary.amount, Decimal::from_str("15000").unwrap());
    }

    #[tokio::test]
    async fn test_exclude_budget_tag() {
        let db = setup_db().await;
        let service = CategoryBreakdownService::new(db.clone());
        let member_id = create_test_member(&db).await;

        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let food_id = db
            .account_create_with_name(&bare_account(Some(expenses_id)), "餐饮", "en")
            .await
            .unwrap();

        let exclude_tag = db
            .tag_get_by_name("exclude-from-budget")
            .await
            .unwrap()
            .unwrap()
            .id;

        // 带"不计预算"标签的分录应被排除
        insert_tx_with_postings(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 6, 10).unwrap(),
            &[(food_id, "999")],
            &[exclude_tag],
        )
        .await;
        insert_tx_with_postings(
            &db,
            member_id,
            NaiveDate::from_ymd_opt(2024, 6, 11).unwrap(),
            &[(food_id, "100")],
            &[],
        )
        .await;

        let report = service
            .category_breakdown(
                NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                FinancePeriod::Monthly,
                CommodityId(1),
            )
            .await
            .unwrap();

        let food = report
            .expense
            .iter()
            .find(|i| i.account.id == food_id)
            .unwrap();
        assert_eq!(food.amount, Decimal::from_str("100").unwrap());
    }
}
