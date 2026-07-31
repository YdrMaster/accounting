//! 攒钱计划状态表

use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId, SavingPlanId};
use accounting::saving_plan::{SavingPlan, SavingPlanError, validate_saving_plan};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// 攒钱计划详情（含账户集合）
#[derive(Debug, Clone)]
pub struct SavingPlanDetail {
    /// 攒钱计划信息
    pub plan: SavingPlan,
    /// 关联账户 ID 列表
    pub account_ids: Vec<AccountId>,
}

/// 攒钱计划状态
#[derive(Debug, Clone)]
pub struct SavingPlanStatus {
    /// 攒钱计划信息
    pub plan: SavingPlan,
    /// 是否已失效（查询日 > deadline）
    pub expired: bool,
    /// 当前周期起始日期（period 为空时为 None）
    pub period_start: Option<NaiveDate>,
    /// 当前周期结束日期（period 为空时为 None）
    pub period_end: Option<NaiveDate>,
    /// 目标金额
    pub target_amount: Decimal,
    /// 账户集合（含后代）截至查询日的余额合计
    pub current_balance: Decimal,
    /// 缺口（target_amount - current_balance）
    pub gap: Decimal,
    /// 是否达标（current_balance >= target_amount）
    pub met: bool,
}

/// 攒钱计划服务
pub struct SavingPlanService {
    db: SqliteDatabase,
}

impl SavingPlanService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 创建攒钱计划
    ///
    /// 攒钱计划名按 `lang` 语言写入名字表。
    #[allow(clippy::too_many_arguments)]
    pub async fn create_saving_plan(
        &self,
        name: &str,
        period: Option<FinancePeriod>,
        deadline: Option<NaiveDate>,
        commodity_id: CommodityId,
        target_amount: Decimal,
        account_ids: &[AccountId],
        lang: &str,
    ) -> Result<SavingPlanId, AccountingError> {
        let accounts = super::load_accounts(&self.db).await?;
        let account_types = super::load_account_types(&self.db).await?;
        let commodity_ids = super::load_commodity_ids(&self.db).await?;
        validate_saving_plan(
            name,
            target_amount,
            account_ids,
            &accounts,
            &account_types,
            commodity_id,
            &commodity_ids,
        )
        .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        self.db
            .saving_plan_create(
                name,
                period,
                deadline,
                commodity_id,
                target_amount,
                account_ids,
                lang,
            )
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 更新攒钱计划（整集合替换账户关联）
    ///
    /// 攒钱计划名按 `lang` 语言更新名字表。
    #[allow(clippy::too_many_arguments)]
    pub async fn update_saving_plan(
        &self,
        plan_id: SavingPlanId,
        name: &str,
        period: Option<FinancePeriod>,
        deadline: Option<NaiveDate>,
        commodity_id: CommodityId,
        target_amount: Decimal,
        account_ids: &[AccountId],
        lang: &str,
    ) -> Result<(), AccountingError> {
        let existing = self
            .db
            .saving_plan_get(plan_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if existing.is_none() {
            return Err(AccountingError::InvalidTransaction(
                SavingPlanError::PlanNotFound(plan_id).to_string(),
            ));
        }

        let accounts = super::load_accounts(&self.db).await?;
        let account_types = super::load_account_types(&self.db).await?;
        let commodity_ids = super::load_commodity_ids(&self.db).await?;
        validate_saving_plan(
            name,
            target_amount,
            account_ids,
            &accounts,
            &account_types,
            commodity_id,
            &commodity_ids,
        )
        .map_err(|e| AccountingError::InvalidTransaction(e.to_string()))?;

        self.db
            .saving_plan_update(
                plan_id,
                name,
                period,
                deadline,
                commodity_id,
                target_amount,
                account_ids,
                lang,
            )
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除攒钱计划（级联删除账户关联）
    pub async fn delete_saving_plan(&self, plan_id: SavingPlanId) -> Result<(), AccountingError> {
        self.db
            .saving_plan_delete(plan_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 列出所有攒钱计划
    pub async fn list_saving_plans(&self) -> Result<Vec<SavingPlan>, AccountingError> {
        self.db
            .saving_plan_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 获取攒钱计划详情（含账户 ID 列表）
    pub async fn get_saving_plan_detail(
        &self,
        plan_id: SavingPlanId,
    ) -> Result<SavingPlanDetail, AccountingError> {
        let plan = self
            .db
            .saving_plan_get(plan_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                AccountingError::InvalidTransaction(
                    SavingPlanError::PlanNotFound(plan_id).to_string(),
                )
            })?;

        let account_ids = self
            .db
            .saving_plan_get_accounts(plan_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(|a| a.account_id)
            .collect();

        Ok(SavingPlanDetail { plan, account_ids })
    }

    /// 查询攒钱计划状态
    ///
    /// 余额为账户集合（含后代子账户）截至查询日（含当日）的合计，
    /// 仅统计与计划币种匹配的分录；不计预算标签不豁免余额。
    pub async fn get_saving_plan_status(
        &self,
        plan_id: SavingPlanId,
        date: NaiveDate,
    ) -> Result<SavingPlanStatus, AccountingError> {
        let detail = self.get_saving_plan_detail(plan_id).await?;
        let plan = detail.plan;

        let expired = plan.deadline.is_some_and(|d| date > d);
        let (period_start, period_end) = match plan.period {
            Some(period) => {
                let (start, end) = period.period_range(date);
                (Some(start), Some(end))
            }
            None => (None, None),
        };

        let current_balance = self
            .db
            .account_balance_by_ids(&detail.account_ids, plan.commodity_id, date)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let gap = plan.target_amount - current_balance;
        let met = current_balance >= plan.target_amount;

        Ok(SavingPlanStatus {
            target_amount: plan.target_amount,
            plan,
            expired,
            period_start,
            period_end,
            current_balance,
            gap,
            met,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{PostingId, TagId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use chrono::NaiveDateTime;
    use std::str::FromStr;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    /// 建库并在 Assets 下建 Alipay/WeChat/Bank/Bank:Checking，Expenses 下建 Food，Income 下建 Salary
    async fn setup() -> SavingPlanService {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let income_id = db.account_get_by_name("Income").await.unwrap().unwrap().id;
        let bare = |parent| Account {
            id: AccountId(0),
            parent_id: Some(parent),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        db.account_create_with_name(&bare(assets_id), "Alipay", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(assets_id), "WeChat", "en")
            .await
            .unwrap();
        let bank_id = db
            .account_create_with_name(&bare(assets_id), "Bank", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(bank_id), "Checking", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(expenses_id), "Food", "en")
            .await
            .unwrap();
        db.account_create_with_name(&bare(income_id), "Salary", "en")
            .await
            .unwrap();

        SavingPlanService::new(db)
    }

    async fn account_id(service: &SavingPlanService, path: &str) -> AccountId {
        service
            .db
            .account_get_by_name(path)
            .await
            .unwrap()
            .unwrap()
            .id
    }

    /// 插入一笔单分录交易（仅用于构造余额）
    async fn add_posting(
        service: &SavingPlanService,
        date: &str,
        account_id: AccountId,
        commodity_id: CommodityId,
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
            commodity_id,
            amount: dec(amount),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        db.posting_insert(&posting).await.unwrap();
    }

    /// 快捷创建一次性攒钱计划（target=5000 CNY，账户为 Alipay+WeChat）
    async fn create_travel_fund(service: &SavingPlanService) -> SavingPlanId {
        let alipay = account_id(service, "Assets:Alipay").await;
        let wechat = account_id(service, "Assets:WeChat").await;
        service
            .create_saving_plan(
                "旅行基金",
                None,
                Some(d(2026, 9, 30)),
                CommodityId(1),
                dec("5000"),
                &[alipay, wechat],
                "zh",
            )
            .await
            .unwrap()
    }

    // === CRUD ===

    #[tokio::test]
    async fn test_create_saving_plan_success() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;

        let plans = service.list_saving_plans().await.unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].id, plan_id);
        assert_eq!(plans[0].period, None);
        assert_eq!(plans[0].deadline, Some(d(2026, 9, 30)));
        assert_eq!(plans[0].target_amount, dec("5000"));

        let accounts = service.db.saving_plan_get_accounts(plan_id).await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn test_create_saving_plan_empty_name_rejected() {
        let service = setup().await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        let result = service
            .create_saving_plan("", None, None, CommodityId(1), dec("100"), &[alipay], "zh")
            .await;
        assert!(result.is_err());
        assert!(service.list_saving_plans().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_create_saving_plan_empty_accounts_rejected() {
        let service = setup().await;
        let err = service
            .create_saving_plan("测试", None, None, CommodityId(1), dec("100"), &[], "zh")
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::EmptyAccounts.to_string())
        );
    }

    #[tokio::test]
    async fn test_create_saving_plan_expense_account_rejected() {
        let service = setup().await;
        let food = account_id(&service, "Expenses:Food").await;
        let err = service
            .create_saving_plan(
                "测试",
                None,
                None,
                CommodityId(1),
                dec("100"),
                &[food],
                "zh",
            )
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::AccountNotAsset(food).to_string())
        );
        assert!(service.list_saving_plans().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_update_saving_plan_replaces_accounts() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let bank = account_id(&service, "Assets:Bank").await;

        service
            .update_saving_plan(
                plan_id,
                "欧洲旅行基金",
                None,
                Some(d(2026, 9, 30)),
                CommodityId(1),
                dec("5000"),
                &[bank],
                "zh",
            )
            .await
            .unwrap();

        // 名称已更新
        let renamed = service
            .db
            .saving_plan_get_by_name("欧洲旅行基金")
            .await
            .unwrap();
        assert_eq!(renamed.unwrap().id, plan_id);
        assert!(
            service
                .db
                .saving_plan_get_by_name("旅行基金")
                .await
                .unwrap()
                .is_none()
        );

        // 旧账户关联全部删除，新关联已插入
        let accounts = service.db.saving_plan_get_accounts(plan_id).await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_id, bank);
    }

    #[tokio::test]
    async fn test_update_saving_plan_not_found() {
        let service = setup().await;
        let bank = account_id(&service, "Assets:Bank").await;
        let err = service
            .update_saving_plan(
                SavingPlanId(999),
                "x",
                None,
                None,
                CommodityId(1),
                dec("100"),
                &[bank],
                "zh",
            )
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::PlanNotFound(SavingPlanId(999)).to_string())
        );
    }

    #[tokio::test]
    async fn test_update_saving_plan_income_account_rejected() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let salary = account_id(&service, "Income:Salary").await;
        let err = service
            .update_saving_plan(
                plan_id,
                "旅行基金",
                None,
                None,
                CommodityId(1),
                dec("5000"),
                &[salary],
                "zh",
            )
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::AccountNotAsset(salary).to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_saving_plan_cascades() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;

        service.delete_saving_plan(plan_id).await.unwrap();

        assert!(service.list_saving_plans().await.unwrap().is_empty());
        assert!(
            service
                .db
                .saving_plan_get_accounts(plan_id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_list_saving_plans() {
        let service = setup().await;
        create_travel_fund(&service).await;
        let bank = account_id(&service, "Assets:Bank").await;
        service
            .create_saving_plan(
                "房租备用金",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                dec("6000"),
                &[bank],
                "zh",
            )
            .await
            .unwrap();

        let plans = service.list_saving_plans().await.unwrap();
        assert_eq!(plans.len(), 2);
    }

    #[tokio::test]
    async fn test_get_saving_plan_detail() {
        let service = setup().await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        let wechat = account_id(&service, "Assets:WeChat").await;
        let bank = account_id(&service, "Assets:Bank").await;
        let plan_id = service
            .create_saving_plan(
                "三账户计划",
                None,
                None,
                CommodityId(1),
                dec("1000"),
                &[alipay, wechat, bank],
                "zh",
            )
            .await
            .unwrap();

        let detail = service.get_saving_plan_detail(plan_id).await.unwrap();
        assert_eq!(detail.plan.id, plan_id);
        assert_eq!(detail.account_ids.len(), 3);
        assert!(detail.account_ids.contains(&alipay));
        assert!(detail.account_ids.contains(&wechat));
        assert!(detail.account_ids.contains(&bank));
    }

    #[tokio::test]
    async fn test_get_saving_plan_detail_not_found() {
        let service = setup().await;
        let err = service
            .get_saving_plan_detail(SavingPlanId(999))
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::PlanNotFound(SavingPlanId(999)).to_string())
        );
    }

    // === 状态计算 ===

    #[tokio::test]
    async fn test_status_multi_account_merged() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        let wechat = account_id(&service, "Assets:WeChat").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "3000", &[]).await;
        add_posting(&service, "2026-06-01", wechat, CommodityId(1), "2000", &[]).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("5000"));
    }

    #[tokio::test]
    async fn test_status_includes_descendants() {
        let service = setup().await;
        let bank = account_id(&service, "Assets:Bank").await;
        let checking = account_id(&service, "Assets:Bank:Checking").await;
        let plan_id = service
            .create_saving_plan(
                "银行存款计划",
                None,
                None,
                CommodityId(1),
                dec("5000"),
                &[bank],
                "zh",
            )
            .await
            .unwrap();
        add_posting(&service, "2026-06-01", bank, CommodityId(1), "1000", &[]).await;
        add_posting(
            &service,
            "2026-06-01",
            checking,
            CommodityId(1),
            "2500",
            &[],
        )
        .await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("3500"));
    }

    #[tokio::test]
    async fn test_status_ignores_other_commodity() {
        let service = setup().await;
        let usd = service
            .db
            .commodity_create(&accounting::commodity::Commodity {
                id: CommodityId(0),
                symbol: "USD".to_string(),
                precision: 2,
                created_at: None,
            })
            .await
            .unwrap();
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "4000", &[]).await;
        add_posting(&service, "2026-06-01", alipay, usd, "100", &[]).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("4000"));
    }

    #[tokio::test]
    async fn test_status_balance_as_of_query_date() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "4500", &[]).await;
        // 查询日之后的入账不计入
        add_posting(&service, "2026-06-30", alipay, CommodityId(1), "500", &[]).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("4500"));
    }

    #[tokio::test]
    async fn test_status_gap_positive_when_unmet() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "3200", &[]).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.target_amount, dec("5000"));
        assert_eq!(status.current_balance, dec("3200"));
        assert_eq!(status.gap, dec("1800"));
        assert!(!status.met);
    }

    #[tokio::test]
    async fn test_status_met_when_exceeded() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "5300", &[]).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("5300"));
        assert_eq!(status.gap, dec("-300"));
        assert!(status.met);
    }

    #[tokio::test]
    async fn test_status_expired_returns_full_status() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "5200", &[]).await;

        // deadline=2026-09-30，查询 2026-10-15：已失效但其余字段正常返回
        let status = service
            .get_saving_plan_status(plan_id, d(2026, 10, 15))
            .await
            .unwrap();
        assert!(status.expired);
        assert_eq!(status.current_balance, dec("5200"));
        assert_eq!(status.gap, dec("-200"));
        assert!(status.met);
    }

    #[tokio::test]
    async fn test_status_valid_on_deadline_day() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 9, 30))
            .await
            .unwrap();
        assert!(!status.expired);
    }

    #[tokio::test]
    async fn test_status_no_deadline_never_expires() {
        let service = setup().await;
        let bank = account_id(&service, "Assets:Bank").await;
        let plan_id = service
            .create_saving_plan(
                "应急金",
                None,
                None,
                CommodityId(1),
                dec("1000"),
                &[bank],
                "zh",
            )
            .await
            .unwrap();

        let status = service
            .get_saving_plan_status(plan_id, d(2099, 1, 1))
            .await
            .unwrap();
        assert!(!status.expired);
    }

    #[tokio::test]
    async fn test_status_monthly_period_range() {
        let service = setup().await;
        let bank = account_id(&service, "Assets:Bank").await;
        let plan_id = service
            .create_saving_plan(
                "房租备用金",
                Some(FinancePeriod::Monthly),
                None,
                CommodityId(1),
                dec("6000"),
                &[bank],
                "zh",
            )
            .await
            .unwrap();

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.period_start, Some(d(2026, 6, 1)));
        assert_eq!(status.period_end, Some(d(2026, 6, 30)));
    }

    #[tokio::test]
    async fn test_status_one_off_no_period_range() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.period_start, None);
        assert_eq!(status.period_end, None);
    }

    #[tokio::test]
    async fn test_status_exclude_from_budget_tag_still_counted() {
        let service = setup().await;
        let plan_id = create_travel_fund(&service).await;
        let alipay = account_id(&service, "Assets:Alipay").await;
        let exclude_tag = service
            .db
            .tag_get_by_name("exclude-from-budget")
            .await
            .unwrap()
            .unwrap();

        add_posting(&service, "2026-06-01", alipay, CommodityId(1), "4000", &[]).await;
        add_posting(&service, "2026-06-02", alipay, CommodityId(1), "800", &[]).await;
        // 带"不计预算"标签的 200 仍计入余额（余额是事实，不可豁免）
        add_posting(
            &service,
            "2026-06-03",
            alipay,
            CommodityId(1),
            "200",
            &[exclude_tag.id],
        )
        .await;

        let status = service
            .get_saving_plan_status(plan_id, d(2026, 6, 26))
            .await
            .unwrap();
        assert_eq!(status.current_balance, dec("5000"));
    }

    #[tokio::test]
    async fn test_status_plan_not_found() {
        let service = setup().await;
        let err = service
            .get_saving_plan_status(SavingPlanId(999), d(2026, 6, 26))
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains(&SavingPlanError::PlanNotFound(SavingPlanId(999)).to_string())
        );
    }
}
