//! 预算执行表

use accounting::budget::{Budget, BudgetError, BudgetLimit, validate_budget};
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, BudgetId, CommodityId, TagId};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

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
    /// 是否已失效（查询日 > deadline）
    pub expired: bool,
    /// 当前周期的起始日期（一次性预算为 None）
    pub period_start: Option<NaiveDate>,
    /// 当前周期的结束日期（一次性预算为 None）
    pub period_end: Option<NaiveDate>,
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
    ///
    /// 预算名按 `lang` 语言写入名字表；period 为 None 时创建一次性预算。
    pub async fn create_budget(
        &self,
        name: &str,
        period: Option<FinancePeriod>,
        deadline: Option<NaiveDate>,
        commodity_id: CommodityId,
        limits: &[(AccountId, Decimal)],
        lang: &str,
    ) -> Result<BudgetId, AccountingError> {
        let accounts = super::load_accounts(&self.db).await?;
        let account_types = super::load_account_types(&self.db).await?;
        let commodity_ids = super::load_commodity_ids(&self.db).await?;
        validate_budget(name, limits, &accounts, &account_types, &commodity_ids)
            .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        if !commodity_ids.contains(&commodity_id) {
            return Err(AccountingError::CommodityNotFound(commodity_id.to_string()));
        }

        self.db
            .budget_create(name, period, deadline, commodity_id, limits, lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 更新预算表
    ///
    /// 预算名按 `lang` 语言更新名字表；period 置空后变为一次性预算。
    #[allow(clippy::too_many_arguments)]
    pub async fn update_budget(
        &self,
        budget_id: BudgetId,
        name: &str,
        period: Option<FinancePeriod>,
        deadline: Option<NaiveDate>,
        commodity_id: CommodityId,
        limits: &[(AccountId, Decimal)],
        lang: &str,
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

        let accounts = super::load_accounts(&self.db).await?;
        let account_types = super::load_account_types(&self.db).await?;
        let commodity_ids = super::load_commodity_ids(&self.db).await?;
        validate_budget(name, limits, &accounts, &account_types, &commodity_ids)
            .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        self.db
            .budget_update(
                budget_id,
                name,
                period,
                deadline,
                commodity_id,
                limits,
                lang,
            )
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
    ///
    /// period 非空时按查询日所在周期计量；period 为空（一次性预算）时，
    /// 计量窗口为不限下界到 min(查询日, deadline)，period_start/period_end 为 None。
    /// 查询日晚于 deadline 时返回 expired=true（其余字段仍正常计算）。
    pub async fn get_budget_status(
        &self,
        budget_id: BudgetId,
        date: NaiveDate,
    ) -> Result<BudgetStatus, AccountingError> {
        let detail = self.get_budget_detail(budget_id).await?;
        let budget = detail.budget;
        let limits = detail.limits;

        let expired = budget.deadline.is_some_and(|d| date > d);

        let exclude_tag_ids = self.get_exclude_budget_tag_ids().await?;

        let account_ids: Vec<AccountId> = limits.iter().map(|l| l.account_id).collect();

        let (period_start, period_end, actuals) = match budget.period {
            Some(period) => {
                let (start, end) = period.period_range(date);
                let actuals = self
                    .db
                    .posting_sum_by_period(
                        &account_ids,
                        start,
                        end,
                        &exclude_tag_ids,
                        budget.commodity_id,
                    )
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                (Some(start), Some(end), actuals)
            }
            None => {
                // 一次性预算：不限下界累计到 min(查询日, deadline)
                let end = match budget.deadline {
                    Some(deadline) if deadline < date => deadline,
                    _ => date,
                };
                let actuals = self
                    .db
                    .posting_sum_before(&account_ids, end, &exclude_tag_ids, budget.commodity_id)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                (None, None, actuals)
            }
        };

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
            expired,
            period_start,
            period_end,
            items,
        })
    }

    /// 列出所有预算表的执行情况
    ///
    /// 按预算 id 升序（`budget_list` 即按 id 排序）；逐项调用 `get_budget_status`，
    /// 保证与单条查询口径一致。
    pub async fn list_budget_statuses(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<BudgetStatus>, AccountingError> {
        let budgets = self.list_budgets().await?;
        let mut statuses = Vec::with_capacity(budgets.len());
        for budget in budgets {
            statuses.push(self.get_budget_status(budget.id, date).await?);
        }
        Ok(statuses)
    }

    /// 查找"不计预算"系统标签 ID（按系统名单次查询；该标签双语名字挂在同一实体上）
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
    use accounting::account::Account;
    use accounting::id::{PostingId, TagId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting_sql::SqliteDatabase;
    use chrono::NaiveDateTime;
    use std::str::FromStr;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    async fn setup() -> BudgetService {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let bare = |parent| Account {
            id: AccountId(0),
            parent_id: Some(parent),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        db.account_create_with_name(&bare(expenses_id), "Food", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(expenses_id), "Transport", "en")
            .await
            .unwrap();

        BudgetService::new(db)
    }

    async fn account_id_by_path(service: &BudgetService, path: &str) -> AccountId {
        service
            .db
            .account_get_by_name(path)
            .await
            .unwrap()
            .unwrap()
            .id
    }

    /// 插入一笔单分录交易（仅用于构造实际支出）
    async fn add_posting(
        service: &BudgetService,
        date: &str,
        account_id: AccountId,
        amount: &str,
        tag_ids: &[TagId],
    ) {
        let db = &service.db;
        let member_id = db.member_get_or_create_by_name("Test", "en").await.unwrap();
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDateTime::parse_from_str(
                &format!("{} 00:00:00", date),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            description: "test".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let tx_id = db.transaction_insert(&tx, tag_ids).await.unwrap();
        let posting = Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id,
            commodity_id: CommodityId(1),
            amount: dec(amount),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        db.posting_insert(&posting).await.unwrap();
    }

    /// 创建一次性预算（period=None, deadline=2026-09-30, Food 限额 2000）
    async fn create_one_off_budget(service: &BudgetService) -> BudgetId {
        let food_id = account_id_by_path(service, "Expenses:Food").await;
        service
            .create_budget(
                "旅行预算",
                None,
                Some(d(2026, 9, 30)),
                CommodityId(1),
                &[(food_id, dec("2000"))],
                "zh",
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_list_budget() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let transport_id = account_id_by_path(&service, "Expenses:Transport").await;

        let id = service
            .create_budget(
                "Monthly Life",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("2000")), (transport_id, dec("500"))],
                "en",
            )
            .await
            .unwrap();

        let list = service.list_budgets().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
    }

    #[tokio::test]
    async fn test_create_one_off_budget() {
        let service = setup().await;
        let budget_id = create_one_off_budget(&service).await;

        let detail = service.get_budget_detail(budget_id).await.unwrap();
        assert_eq!(detail.budget.period, None);
        assert_eq!(detail.budget.deadline, Some(d(2026, 9, 30)));
        assert_eq!(detail.limits.len(), 1);
    }

    #[tokio::test]
    async fn test_create_budget_asset_account_rejected() {
        let service = setup().await;
        let assets_id = account_id_by_path(&service, "Assets").await;
        let err = service
            .create_budget(
                "Bad",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(assets_id, dec("1000"))],
                "en",
            )
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&BudgetError::AccountNotExpense(assets_id).to_string())
        );
        assert!(service.list_budgets().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_budget_status() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let transport_id = account_id_by_path(&service, "Expenses:Transport").await;

        let budget_id = service
            .create_budget(
                "Monthly Life",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("2000")), (transport_id, dec("500"))],
                "en",
            )
            .await
            .unwrap();

        add_posting(&service, "2024-06-15", food_id, "-800", &[]).await;

        let status = service
            .get_budget_status(budget_id, d(2024, 6, 15))
            .await
            .unwrap();

        assert!(!status.expired);
        assert_eq!(status.period_start, Some(d(2024, 6, 1)));
        assert_eq!(status.period_end, Some(d(2024, 6, 30)));
        assert_eq!(status.items.len(), 2);

        let food_item = status
            .items
            .iter()
            .find(|i| i.account_id == food_id)
            .unwrap();
        assert_eq!(food_item.limit_amount, dec("2000"));
        assert_eq!(food_item.actual_amount, dec("800"));
        assert_eq!(food_item.remaining, dec("1200"));
    }

    #[tokio::test]
    async fn test_one_off_budget_status_full_history() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let budget_id = create_one_off_budget(&service).await;

        // 跨年份的历史支出全部累计；查询日之后的不计入
        add_posting(&service, "2025-01-10", food_id, "-300", &[]).await;
        add_posting(&service, "2026-09-10", food_id, "-500", &[]).await;
        add_posting(&service, "2026-09-20", food_id, "-200", &[]).await;

        let status = service
            .get_budget_status(budget_id, d(2026, 9, 15))
            .await
            .unwrap();

        assert!(!status.expired);
        assert_eq!(status.period_start, None);
        assert_eq!(status.period_end, None);
        let food_item = &status.items[0];
        assert_eq!(food_item.actual_amount, dec("800"));
        assert_eq!(food_item.remaining, dec("1200"));
    }

    #[tokio::test]
    async fn test_one_off_budget_status_capped_at_deadline() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let budget_id = create_one_off_budget(&service).await;

        add_posting(&service, "2026-09-10", food_id, "-500", &[]).await;
        add_posting(&service, "2026-09-20", food_id, "-200", &[]).await;
        // deadline 之后的支出不计入（窗口上界为 min(查询日, deadline)）
        add_posting(&service, "2026-10-05", food_id, "-50", &[]).await;

        let status = service
            .get_budget_status(budget_id, d(2026, 10, 15))
            .await
            .unwrap();

        assert!(status.expired);
        assert_eq!(status.period_start, None);
        assert_eq!(status.period_end, None);
        assert_eq!(status.items[0].actual_amount, dec("700"));
    }

    #[tokio::test]
    async fn test_one_off_budget_excludes_tagged_transactions() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let budget_id = create_one_off_budget(&service).await;
        let exclude_tag = service
            .db
            .tag_get_by_name("exclude-from-budget")
            .await
            .unwrap()
            .unwrap();

        add_posting(&service, "2026-09-01", food_id, "-400", &[]).await;
        // "不计预算"标签的支出对预算仍然豁免
        add_posting(&service, "2026-09-02", food_id, "-200", &[exclude_tag.id]).await;

        let status = service
            .get_budget_status(budget_id, d(2026, 9, 15))
            .await
            .unwrap();
        assert_eq!(status.items[0].actual_amount, dec("400"));
    }

    #[tokio::test]
    async fn test_period_budget_expired_after_deadline() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let budget_id = service
            .create_budget(
                "限期月度",
                Some(FinancePeriod::Monthly),
                Some(d(2026, 6, 30)),
                CommodityId(1),
                &[(food_id, dec("2000"))],
                "zh",
            )
            .await
            .unwrap();

        // deadline 当天仍有效
        let status = service
            .get_budget_status(budget_id, d(2026, 6, 30))
            .await
            .unwrap();
        assert!(!status.expired);
        assert_eq!(status.period_start, Some(d(2026, 6, 1)));
        assert_eq!(status.period_end, Some(d(2026, 6, 30)));

        // deadline 之后已失效，周期窗口仍正常计算
        let status = service
            .get_budget_status(budget_id, d(2026, 7, 1))
            .await
            .unwrap();
        assert!(status.expired);
        assert_eq!(status.period_start, Some(d(2026, 7, 1)));
        assert_eq!(status.period_end, Some(d(2026, 7, 31)));
    }

    #[tokio::test]
    async fn test_update_budget_to_one_off() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let budget_id = service
            .create_budget(
                "Monthly Life",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("2000"))],
                "en",
            )
            .await
            .unwrap();

        // 置空周期 + 设置 deadline → 一次性预算
        service
            .update_budget(
                budget_id,
                "Monthly Life",
                None,
                Some(d(2026, 12, 31)),
                CommodityId(1),
                &[(food_id, dec("2500"))],
                "en",
            )
            .await
            .unwrap();

        let detail = service.get_budget_detail(budget_id).await.unwrap();
        assert_eq!(detail.budget.period, None);
        assert_eq!(detail.budget.deadline, Some(d(2026, 12, 31)));
        assert_eq!(detail.limits.len(), 1);
        assert_eq!(detail.limits[0].amount, dec("2500"));
    }

    #[tokio::test]
    async fn test_delete_budget() {
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;

        let id = service
            .create_budget(
                "ToDelete",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("1000"))],
                "en",
            )
            .await
            .unwrap();

        service.delete_budget(id).await.unwrap();

        let list = service.list_budgets().await.unwrap();
        assert!(list.is_empty());
    }

    // === 批量执行情况 ===

    #[tokio::test]
    async fn test_list_budget_statuses_sorted_by_id() {
        // spec: 批量执行情况按预算 id 升序
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let b1 = service
            .create_budget(
                "月度生活",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("2000"))],
                "zh",
            )
            .await
            .unwrap();
        let b2 = create_one_off_budget(&service).await;

        let list = service.list_budget_statuses(d(2026, 6, 15)).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].budget.id, b1);
        assert_eq!(list[1].budget.id, b2);
    }

    #[tokio::test]
    async fn test_list_budget_statuses_matches_single() {
        // spec: 批量与单条口径一致（actual_amount/remaining/percentage）
        let service = setup().await;
        let food_id = account_id_by_path(&service, "Expenses:Food").await;
        let b1 = service
            .create_budget(
                "月度生活",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                &[(food_id, dec("2000"))],
                "zh",
            )
            .await
            .unwrap();
        let b2 = create_one_off_budget(&service).await;
        add_posting(&service, "2026-06-15", food_id, "-800", &[]).await;

        let date = d(2026, 6, 15);
        let list = service.list_budget_statuses(date).await.unwrap();
        assert_eq!(list.len(), 2);
        for status in &list {
            assert!(b1 == status.budget.id || b2 == status.budget.id);
            let single = service
                .get_budget_status(status.budget.id, date)
                .await
                .unwrap();
            assert_eq!(status.expired, single.expired);
            assert_eq!(status.items.len(), single.items.len());
            for (item, single_item) in status.items.iter().zip(single.items.iter()) {
                assert_eq!(item.account_id, single_item.account_id);
                assert_eq!(item.actual_amount, single_item.actual_amount);
                assert_eq!(item.remaining, single_item.remaining);
                assert_eq!(item.percentage, single_item.percentage);
            }
        }
    }

    #[tokio::test]
    async fn test_list_budget_statuses_empty() {
        // spec: 无预算时返回空数组
        let service = setup().await;
        let list = service.list_budget_statuses(d(2026, 6, 15)).await.unwrap();
        assert!(list.is_empty());
    }
}
