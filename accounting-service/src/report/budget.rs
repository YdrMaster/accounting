//! 预算执行表

use accounting::budget::{Budget, BudgetError, BudgetLimit, validate_budget};
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, BudgetId, CommodityId, TagId};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// 预算表详情（含限额）
#[derive(Debug, Clone)]
pub struct BudgetDetail {
    /// 预算表信息
    pub budget: Budget,
    /// 限额列表
    pub limits: Vec<BudgetLimit>,
}

/// 预算执行情况
#[derive(Debug, Clone)]
pub struct BudgetStatus {
    /// 预算表信息
    pub budget: Budget,
    /// 当前周期的起始日期
    pub period_start: NaiveDate,
    /// 当前周期的结束日期
    pub period_end: NaiveDate,
    /// 各账户限额执行情况
    pub items: Vec<BudgetItemStatus>,
}

/// 单个账户的预算执行情况
#[derive(Debug, Clone)]
pub struct BudgetItemStatus {
    /// 账户 ID
    pub account_id: AccountId,
    /// 预算限额
    pub limit_amount: Decimal,
    /// 实际支出/收入金额
    pub actual_amount: Decimal,
    /// 剩余/超支金额（正=剩余，负=超支）
    pub remaining: Decimal,
    /// 执行百分比 (actual / limit * 100)
    pub percentage: Decimal,
}

/// 预算服务
pub struct BudgetService {
    db: SqliteDatabase,
}

impl BudgetService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 创建预算表
    pub async fn create_budget(
        &self,
        name: &str,
        period: FinancePeriod,
        commodity_id: CommodityId,
        limits: &[(AccountId, Decimal)],
    ) -> Result<BudgetId, AccountingError> {
        let accounts = self.load_accounts().await?;
        let commodity_ids = self.load_commodity_ids().await?;
        validate_budget(name, limits, &accounts, &commodity_ids)
            .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        if !commodity_ids.contains(&commodity_id) {
            return Err(AccountingError::CommodityNotFound(commodity_id.to_string()));
        }

        self.db
            .budget_create(name, period, commodity_id, limits)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 更新预算表
    pub async fn update_budget(
        &self,
        budget_id: BudgetId,
        name: &str,
        period: FinancePeriod,
        commodity_id: CommodityId,
        limits: &[(AccountId, Decimal)],
    ) -> Result<(), AccountingError> {
        let existing = self
            .db
            .budget_get(budget_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if existing.is_none() {
            return Err(AccountingError::InvalidTransaction(
                BudgetError::BudgetNotFound(budget_id).to_string(),
            ));
        }

        let accounts = self.load_accounts().await?;
        let commodity_ids = self.load_commodity_ids().await?;
        validate_budget(name, limits, &accounts, &commodity_ids)
            .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        self.db
            .budget_update(budget_id, name, period, commodity_id, limits)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除预算表
    pub async fn delete_budget(&self, budget_id: BudgetId) -> Result<(), AccountingError> {
        self.db
            .budget_delete(budget_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 列出所有预算表
    pub async fn list_budgets(&self) -> Result<Vec<Budget>, AccountingError> {
        self.db
            .budget_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 按名称查找预算表
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Budget>, AccountingError> {
        self.db
            .budget_get_by_name(name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 获取预算表详情（含限额列表）
    pub async fn get_budget_detail(
        &self,
        budget_id: BudgetId,
    ) -> Result<BudgetDetail, AccountingError> {
        let budget = self
            .db
            .budget_get(budget_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                AccountingError::InvalidTransaction(
                    BudgetError::BudgetNotFound(budget_id).to_string(),
                )
            })?;

        let limits = self
            .db
            .budget_get_limits(budget_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        Ok(BudgetDetail { budget, limits })
    }

    /// 查询预算执行情况
    pub async fn get_budget_status(
        &self,
        budget_id: BudgetId,
        date: NaiveDate,
    ) -> Result<BudgetStatus, AccountingError> {
        let detail = self.get_budget_detail(budget_id).await?;
        let budget = detail.budget;
        let limits = detail.limits;

        let (period_start, period_end) = budget.period.period_range(date);

        let exclude_tag_ids = self.get_exclude_budget_tag_ids().await?;

        let account_ids: Vec<AccountId> = limits.iter().map(|l| l.account_id).collect();

        let actuals = self
            .db
            .posting_sum_by_period(
                &account_ids,
                period_start,
                period_end,
                &exclude_tag_ids,
                budget.commodity_id,
            )
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let actual_map: HashMap<AccountId, Decimal> = actuals.into_iter().collect();

        let items: Vec<BudgetItemStatus> = limits
            .iter()
            .map(|limit| {
                let actual = actual_map
                    .get(&limit.account_id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let remaining = limit.amount - actual.abs();
                let percentage = if limit.amount.is_zero() {
                    Decimal::ZERO
                } else {
                    actual.abs() / limit.amount * Decimal::from(100)
                };
                BudgetItemStatus {
                    account_id: limit.account_id,
                    limit_amount: limit.amount,
                    actual_amount: actual.abs(),
                    remaining,
                    percentage,
                }
            })
            .collect();

        Ok(BudgetStatus {
            budget,
            period_start,
            period_end,
            items,
        })
    }

    async fn get_exclude_budget_tag_ids(&self) -> Result<Vec<TagId>, AccountingError> {
        let tags = self
            .db
            .tag_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let exclude_names = ["exclude-from-budget", "不计预算"];
        Ok(tags
            .iter()
            .filter(|t| exclude_names.contains(&t.name.as_str()))
            .map(|t| t.id)
            .collect())
    }

    async fn load_accounts(
        &self,
    ) -> Result<HashMap<AccountId, accounting::account::Account>, AccountingError> {
        let list = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(list.into_iter().map(|a| (a.id, a)).collect())
    }

    async fn load_commodity_ids(&self) -> Result<HashSet<CommodityId>, AccountingError> {
        let list = self
            .db
            .commodity_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(list.into_iter().map(|c| c.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::MemberId;
    use accounting::member::Member;
    use accounting_sql::SqliteDatabase;
    use std::str::FromStr;

    async fn setup() -> BudgetService {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();

        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let food = Account {
            id: AccountId(0),
            name: "Food".to_string(),
            parent_id: Some(expenses_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        db.account_create_with_closure(&food).await.unwrap();

        let transport = Account {
            id: AccountId(0),
            name: "Transport".to_string(),
            parent_id: Some(expenses_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        db.account_create_with_closure(&transport).await.unwrap();

        BudgetService::new(db)
    }

    #[tokio::test]
    async fn test_create_and_list_budget() {
        let service = setup().await;
        let accounts = service.db.account_list().await.unwrap();
        let food = accounts.iter().find(|a| a.name == "Food").unwrap();
        let transport = accounts.iter().find(|a| a.name == "Transport").unwrap();

        let id = service
            .create_budget(
                "Monthly Life",
                FinancePeriod::Monthly,
                CommodityId(1),
                &[
                    (food.id, Decimal::from_str("2000").unwrap()),
                    (transport.id, Decimal::from_str("500").unwrap()),
                ],
            )
            .await
            .unwrap();

        let list = service.list_budgets().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
    }

    #[tokio::test]
    async fn test_get_budget_status() {
        let service = setup().await;
        let accounts = service.db.account_list().await.unwrap();
        let food = accounts.iter().find(|a| a.name == "Food").unwrap();
        let transport = accounts.iter().find(|a| a.name == "Transport").unwrap();
        let assets = accounts.iter().find(|a| a.name == "Assets").unwrap();

        let member_id = service
            .db
            .member_create(&Member {
                id: MemberId(0),
                name: "Test".to_string(),
            })
            .await
            .unwrap();

        let budget_id = service
            .create_budget(
                "Monthly Life",
                FinancePeriod::Monthly,
                CommodityId(1),
                &[
                    (food.id, Decimal::from_str("2000").unwrap()),
                    (transport.id, Decimal::from_str("500").unwrap()),
                ],
            )
            .await
            .unwrap();

        let tx = accounting::transaction::Transaction {
            id: accounting::id::TransactionId(0),
            date_time: chrono::NaiveDateTime::parse_from_str(
                "2024-06-15 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            description: "lunch".to_string(),
            kind: accounting::transaction::TransactionKind::Normal,
            member_id,
        };
        let tx_id = service.db.transaction_insert(&tx, &[]).await.unwrap();

        let p1 = accounting::posting::Posting {
            id: accounting::id::PostingId(0),
            transaction_id: tx_id,
            account_id: food.id,
            commodity_id: CommodityId(1),
            amount: Decimal::from_str("-800").unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        let p2 = accounting::posting::Posting {
            id: accounting::id::PostingId(0),
            transaction_id: tx_id,
            account_id: assets.id,
            commodity_id: CommodityId(1),
            amount: Decimal::from_str("800").unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        service.db.posting_insert(&p1).await.unwrap();
        service.db.posting_insert(&p2).await.unwrap();

        let status = service
            .get_budget_status(budget_id, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap())
            .await
            .unwrap();

        assert_eq!(
            status.period_start,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()
        );
        assert_eq!(
            status.period_end,
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()
        );
        assert_eq!(status.items.len(), 2);

        let food_item = status
            .items
            .iter()
            .find(|i| i.account_id == food.id)
            .unwrap();
        assert_eq!(food_item.limit_amount, Decimal::from_str("2000").unwrap());
        assert_eq!(food_item.actual_amount, Decimal::from_str("800").unwrap());
        assert_eq!(food_item.remaining, Decimal::from_str("1200").unwrap());
    }

    #[tokio::test]
    async fn test_delete_budget() {
        let service = setup().await;
        let accounts = service.db.account_list().await.unwrap();
        let food = accounts.iter().find(|a| a.name == "Food").unwrap();

        let id = service
            .create_budget(
                "ToDelete",
                FinancePeriod::Monthly,
                CommodityId(1),
                &[(food.id, Decimal::from_str("1000").unwrap())],
            )
            .await
            .unwrap();

        service.delete_budget(id).await.unwrap();

        let list = service.list_budgets().await.unwrap();
        assert!(list.is_empty());
    }
}
