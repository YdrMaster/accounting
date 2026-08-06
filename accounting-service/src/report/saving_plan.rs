//! 攒钱计划状态表

use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId, SavingPlanId};
use accounting::saving_plan::{SavingPlan, SavingPlanError, validate_saving_plan};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// 攒钱计划详情（含账户集合）
#[derive(Debug, Clone)]
pub struct SavingPlanDetail {
    /// 攒钱计划信息
    pub plan: SavingPlan,
    /// 关联账户 ID 列表
    pub account_ids: Vec<AccountId>,
}

/// 攒钱计划单账户分配明细
#[derive(Debug, Clone)]
pub struct SavingPlanAccountAllocation {
    /// 账户 ID
    pub account_id: AccountId,
    /// 该账户（含后代）截至查询日的余额
    pub balance: Decimal,
    /// 被更早检查点的计划占用的金额
    pub occupied_by_earlier: Decimal,
    /// 本计划分配到的金额
    pub allocated: Decimal,
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
    /// 全局分配口径下本计划分配到的金额（Σ 各账户 allocated）
    pub allocated: Decimal,
    /// 满足率（allocated / target_amount * 100）
    pub satisfaction: Decimal,
    /// 各账户分配明细
    pub accounts: Vec<SavingPlanAccountAllocation>,
}

/// 攒钱计划服务
pub struct SavingPlanService {
    db: SqliteDatabase,
}

/// 全局分配遍历中的单计划中间结果
struct PlanAllocationWork {
    plan: SavingPlan,
    account_ids: Vec<AccountId>,
    expired: bool,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    /// 检查点（None 表示不参与全局分配：永久/过期计划）
    checkpoint: Option<NaiveDate>,
    current_balance: Decimal,
    allocated: Decimal,
    accounts: Vec<SavingPlanAccountAllocation>,
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
    /// 分配字段（allocated/satisfaction/accounts）基于一次全局分配遍历，
    /// 与 `list_saving_plan_statuses` 口径一致。
    pub async fn get_saving_plan_status(
        &self,
        plan_id: SavingPlanId,
        date: NaiveDate,
    ) -> Result<SavingPlanStatus, AccountingError> {
        self.compute_allocations(date)
            .await?
            .into_iter()
            .find(|s| s.plan.id == plan_id)
            .ok_or_else(|| {
                AccountingError::InvalidTransaction(
                    SavingPlanError::PlanNotFound(plan_id).to_string(),
                )
            })
    }

    /// 列出所有攒钱计划的状态
    ///
    /// 与 `get_saving_plan_status` 复用同一全局分配计算，保证两种入口口径一致。
    /// 排序：参与全局分配的计划按（检查点, plan_id）升序在前，
    /// 不参与分配的计划（过期/永久）在后，按 plan_id 升序。
    pub async fn list_saving_plan_statuses(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<SavingPlanStatus>, AccountingError> {
        self.compute_allocations(date).await
    }

    /// 一次全局分配遍历，返回所有计划的状态（内部计算入口）
    ///
    /// 参与计划（未过期且有检查点）按 commodity_id 分组，组内按（检查点, plan_id）
    /// 升序顺序占用资金（`allocated = min(target, available)`，欠费占光）；
    /// 永久/过期计划不参与，按无竞争退化口径计算分配字段。
    async fn compute_allocations(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<SavingPlanStatus>, AccountingError> {
        let plans = self
            .db
            .saving_plan_list_all_with_accounts()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 基础信息：过期判定、周期区间、检查点、独立口径余额
        let mut works = Vec::with_capacity(plans.len());
        for (plan, accounts) in plans {
            let expired = plan.deadline.is_some_and(|d| date > d);
            let (period_start, period_end) = match plan.period {
                Some(period) => {
                    let (start, end) = period.period_range(date);
                    (Some(start), Some(end))
                }
                None => (None, None),
            };
            // 检查点：一次性计划为 deadline；周期计划为 min(当前周期末, deadline)；
            // 永久计划（period/deadline 皆空）与过期计划不参与分配
            let checkpoint = if expired {
                None
            } else {
                match (plan.period, plan.deadline) {
                    (None, None) => None,
                    (None, Some(deadline)) => Some(deadline),
                    (Some(_), deadline) => {
                        let end = period_end.expect("周期计划必有周期末");
                        Some(deadline.map_or(end, |d| d.min(end)))
                    }
                }
            };
            let account_ids: Vec<AccountId> = accounts.iter().map(|a| a.account_id).collect();
            let current_balance = self
                .db
                .account_balance_by_ids(&account_ids, plan.commodity_id, date)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            works.push(PlanAllocationWork {
                plan,
                account_ids,
                expired,
                period_start,
                period_end,
                checkpoint,
                current_balance,
                allocated: Decimal::ZERO,
                accounts: Vec::new(),
            });
        }

        // 按 commodity 分组（跨币种互不争钱），保持首次出现顺序
        let mut commodity_ids: Vec<CommodityId> = Vec::new();
        for w in &works {
            if !commodity_ids.contains(&w.plan.commodity_id) {
                commodity_ids.push(w.plan.commodity_id);
            }
        }

        for commodity_id in commodity_ids {
            // 本币种相关账户的分组余额（含后代、仅本币、截至查询日）
            let mut all_ids: Vec<AccountId> = Vec::new();
            for w in &works {
                if w.plan.commodity_id == commodity_id {
                    for id in &w.account_ids {
                        if !all_ids.contains(id) {
                            all_ids.push(*id);
                        }
                    }
                }
            }
            let balances: HashMap<AccountId, Decimal> = self
                .db
                .account_balances_by_ids(&all_ids, commodity_id, date)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .into_iter()
                .collect();
            let balance_of = |id: &AccountId| balances.get(id).copied().unwrap_or_default();

            // 参与者按（检查点, plan_id）升序，顺序执行占用
            let mut order: Vec<usize> = works
                .iter()
                .enumerate()
                .filter(|(_, w)| w.plan.commodity_id == commodity_id && w.checkpoint.is_some())
                .map(|(i, _)| i)
                .collect();
            order.sort_by_key(|&i| {
                (
                    works[i].checkpoint.expect("参与者必有检查点"),
                    works[i].plan.id,
                )
            });

            let mut occupied: HashMap<AccountId, Decimal> = HashMap::new();
            for (pos, &i) in order.iter().enumerate() {
                let current: HashSet<AccountId> = works[i].account_ids.iter().copied().collect();
                // 排序在后第一个账户集合有交集的计划 j（账户内分配偏好）
                let next_intersection: Option<HashSet<AccountId>> = order[pos + 1..]
                    .iter()
                    .map(|&j| works[j].account_ids.iter().copied().collect::<HashSet<_>>())
                    .find(|s| !s.is_disjoint(&current));

                let available: Decimal = works[i]
                    .account_ids
                    .iter()
                    .map(|id| balance_of(id) - occupied.get(id).copied().unwrap_or_default())
                    .sum();
                // 欠费（available < target）占光全部可用
                let allocated = works[i]
                    .plan
                    .target_amount
                    .min(available)
                    .max(Decimal::ZERO);

                // 扣减顺序：优先 Sᵢ \ Sⱼ（与 j 无关的账户），再 Sᵢ ∩ Sⱼ；同类按账户 id 升序
                let mut deduct_order = works[i].account_ids.clone();
                deduct_order.sort();
                if let Some(ref s) = next_intersection {
                    deduct_order.sort_by_key(|id| s.contains(id));
                }

                let occupied_before: HashMap<AccountId, Decimal> = works[i]
                    .account_ids
                    .iter()
                    .map(|id| (*id, occupied.get(id).copied().unwrap_or_default()))
                    .collect();

                let mut remaining = allocated;
                let mut taken: HashMap<AccountId, Decimal> = HashMap::new();
                for id in &deduct_order {
                    let avail = balance_of(id) - occupied.get(id).copied().unwrap_or_default();
                    let take = avail.min(remaining).max(Decimal::ZERO);
                    if !take.is_zero() {
                        *occupied.entry(*id).or_default() += take;
                    }
                    taken.insert(*id, take);
                    remaining -= take;
                }

                works[i].allocated = allocated;
                works[i].accounts = works[i]
                    .account_ids
                    .iter()
                    .map(|id| SavingPlanAccountAllocation {
                        account_id: *id,
                        balance: balance_of(id),
                        occupied_by_earlier: occupied_before.get(id).copied().unwrap_or_default(),
                        allocated: taken.get(id).copied().unwrap_or_default(),
                    })
                    .collect();
                works[i].accounts.sort_by_key(|a| a.account_id);
            }

            // 非参与计划（永久/过期）：无竞争退化口径，不占用资金也不视为竞争者
            for w in &mut works {
                if w.plan.commodity_id != commodity_id || w.checkpoint.is_some() {
                    continue;
                }
                let available: Decimal = w.account_ids.iter().map(balance_of).sum();
                let allocated = w.plan.target_amount.min(available).max(Decimal::ZERO);

                let mut ids_asc = w.account_ids.clone();
                ids_asc.sort();
                let mut remaining = allocated;
                let mut account_allocs = Vec::with_capacity(ids_asc.len());
                for id in ids_asc {
                    let balance = balance_of(&id);
                    let take = balance.min(remaining).max(Decimal::ZERO);
                    remaining -= take;
                    account_allocs.push(SavingPlanAccountAllocation {
                        account_id: id,
                        balance,
                        occupied_by_earlier: Decimal::ZERO,
                        allocated: take,
                    });
                }
                w.allocated = allocated;
                w.accounts = account_allocs;
            }
        }

        // 输出顺序：参与分配的计划按（检查点, plan_id）升序在前，
        // 不参与的计划（过期/永久）在后，按 plan_id 升序
        works.sort_by_key(|w| (w.checkpoint.is_none(), w.checkpoint, w.plan.id));

        Ok(works
            .into_iter()
            .map(|w| {
                let gap = w.plan.target_amount - w.current_balance;
                let met = w.current_balance >= w.plan.target_amount;
                let satisfaction = if w.plan.target_amount.is_zero() {
                    Decimal::ZERO
                } else {
                    w.allocated / w.plan.target_amount * Decimal::from(100)
                };
                SavingPlanStatus {
                    target_amount: w.plan.target_amount,
                    plan: w.plan,
                    expired: w.expired,
                    period_start: w.period_start,
                    period_end: w.period_end,
                    current_balance: w.current_balance,
                    gap,
                    met,
                    allocated: w.allocated,
                    satisfaction,
                    accounts: w.accounts,
                }
            })
            .collect())
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

    /// 在 Assets 下新建一个资产账户
    async fn create_asset_account(service: &SavingPlanService, name: &str) -> AccountId {
        let assets_id = service
            .db
            .account_get_by_name("Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        service
            .db
            .account_create_with_name(
                &Account {
                    id: AccountId(0),
                    parent_id: Some(assets_id),
                    closed_at: None,
                    is_system: false,
                    billing_day: None,
                    repayment_day: None,
                },
                name,
                "en",
            )
            .await
            .unwrap()
    }

    /// 快捷创建攒钱计划
    async fn create_plan(
        service: &SavingPlanService,
        name: &str,
        period: Option<FinancePeriod>,
        deadline: Option<NaiveDate>,
        commodity_id: CommodityId,
        target: &str,
        account_ids: &[AccountId],
    ) -> SavingPlanId {
        service
            .create_saving_plan(
                name,
                period,
                deadline,
                commodity_id,
                dec(target),
                account_ids,
                "zh",
            )
            .await
            .unwrap()
    }

    /// 取某账户的分配明细
    fn alloc_of(status: &SavingPlanStatus, account_id: AccountId) -> SavingPlanAccountAllocation {
        status
            .accounts
            .iter()
            .find(|a| a.account_id == account_id)
            .unwrap()
            .clone()
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

    // === 全局资金分配 ===

    #[tokio::test]
    async fn test_allocation_first_come_first_served() {
        // spec: 按检查点顺序先到先得
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        let b = create_asset_account(&service, "B").await;
        let c = create_asset_account(&service, "C").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "3000", &[]).await;
        add_posting(&service, "2026-06-01", b, CommodityId(1), "1000", &[]).await;
        add_posting(&service, "2026-06-01", c, CommodityId(1), "500", &[]).await;

        let plan1 = create_plan(
            &service,
            "计划1",
            None,
            Some(d(2026, 8, 31)),
            CommodityId(1),
            "3000",
            &[a, b],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "计划2",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "2000",
            &[a, c],
        )
        .await;

        let s1 = service
            .get_saving_plan_status(plan1, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s1.allocated, dec("3000"));
        assert_eq!(s1.satisfaction, dec("100"));

        // 计划 1 按 D3 偏好先扣非交集账户 B（1000），再扣交集账户 A（2000），A 剩 1000。
        // 计划 2 可用 = A 剩 1000 + C 500 = 1500。
        // 注：spec「按检查点顺序先到先得」场景曾误写"A 剩 0、satisfaction=25"（按无偏好的
        // id 顺序扣减），与 D3 矛盾，已修正为 D3 口径（A 剩 1000、satisfaction=75）。
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("1500"));
        assert_eq!(s2.satisfaction, dec("75"));
        let alloc_a = alloc_of(&s2, a);
        assert_eq!(alloc_a.occupied_by_earlier, dec("2000"));
        assert_eq!(alloc_a.allocated, dec("1000"));
        assert_eq!(alloc_of(&s2, c).allocated, dec("500"));
    }

    #[tokio::test]
    async fn test_allocation_shortfall_takes_all_available() {
        // spec: 欠费计划占光可用 + 满足率计算 + 无后续交集计划时按账户顺序分配
        let service = setup().await;
        let c = create_asset_account(&service, "C").await;
        let dd = create_asset_account(&service, "D").await;
        add_posting(&service, "2026-06-01", c, CommodityId(1), "2000", &[]).await;
        add_posting(&service, "2026-06-01", dd, CommodityId(1), "1000", &[]).await;

        let plan1 = create_plan(
            &service,
            "计划1",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "4000",
            &[c, dd],
        )
        .await;

        let status = service
            .get_saving_plan_status(plan1, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(status.allocated, dec("3000"));
        assert_eq!(status.satisfaction, dec("75"));
        // 无后续交集计划：按账户 id 升序占光
        assert_eq!(alloc_of(&status, c).allocated, dec("2000"));
        assert_eq!(alloc_of(&status, dd).allocated, dec("1000"));
    }

    #[tokio::test]
    async fn test_allocation_cross_commodity_isolated() {
        // spec: 跨币种不争钱
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
        let a = create_asset_account(&service, "A").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "3000", &[]).await;
        add_posting(&service, "2026-06-01", a, usd, "50", &[]).await;

        create_plan(
            &service,
            "人民币计划",
            None,
            Some(d(2026, 8, 31)),
            CommodityId(1),
            "3000",
            &[a],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "美元计划",
            None,
            Some(d(2026, 9, 30)),
            usd,
            "100",
            &[a],
        )
        .await;

        // 计划 1 的 CNY 占用不影响计划 2 的 USD 可用
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("50"));
        assert_eq!(s2.satisfaction, dec("50"));
        assert_eq!(alloc_of(&s2, a).occupied_by_earlier, dec("0"));
    }

    #[tokio::test]
    async fn test_allocation_periodic_checkpoint_is_period_end() {
        // spec: 周期计划检查点为当前周期末
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "1200", &[]).await;

        let plan1 = create_plan(
            &service,
            "一次性计划",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "月度计划",
            Some(FinancePeriod::Monthly),
            None,
            CommodityId(1),
            "500",
            &[a],
        )
        .await;

        // 查询日 2026-07-15：计划 2 检查点 2026-07-31 早于计划 1 的 2026-09-30，先占用
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("500"));
        assert_eq!(s2.satisfaction, dec("100"));

        let s1 = service
            .get_saving_plan_status(plan1, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s1.allocated, dec("700"));
        assert_eq!(s1.satisfaction, dec("70"));
        assert_eq!(alloc_of(&s1, a).occupied_by_earlier, dec("500"));
    }

    #[tokio::test]
    async fn test_allocation_permanent_plan_excluded() {
        // spec: 永久计划不参与分配
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "2500", &[]).await;

        let plan1 = create_plan(
            &service,
            "永久计划",
            None,
            None,
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "一次性计划",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "2000",
            &[a],
        )
        .await;

        // 计划 1 未占用资金，计划 2 无竞争者
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("2000"));
        assert_eq!(s2.satisfaction, dec("100"));
        assert_eq!(alloc_of(&s2, a).occupied_by_earlier, dec("0"));

        // 永久计划按无竞争退化口径返回分配字段
        let s1 = service
            .get_saving_plan_status(plan1, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s1.allocated, dec("1000"));
        assert_eq!(s1.satisfaction, dec("100"));
        let alloc_a = alloc_of(&s1, a);
        assert_eq!(alloc_a.occupied_by_earlier, dec("0"));
        assert_eq!(alloc_a.allocated, dec("1000"));
    }

    #[tokio::test]
    async fn test_allocation_expired_plan_excluded() {
        // spec: 过期计划不占用资金
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "3000", &[]).await;

        let plan1 = create_plan(
            &service,
            "过期计划",
            None,
            Some(d(2026, 8, 31)),
            CommodityId(1),
            "3000",
            &[a],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "有效计划",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "3000",
            &[a],
        )
        .await;

        // 查询日 2026-09-15：计划 1 已过期，不参与分配
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 9, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("3000"));
        assert_eq!(s2.satisfaction, dec("100"));
        assert_eq!(alloc_of(&s2, a).occupied_by_earlier, dec("0"));

        // 过期计划仍返回完整状态，分配字段按无竞争退化口径
        let s1 = service
            .get_saving_plan_status(plan1, d(2026, 9, 15))
            .await
            .unwrap();
        assert!(s1.expired);
        assert_eq!(s1.allocated, dec("3000"));
        assert_eq!(s1.satisfaction, dec("100"));
        assert_eq!(alloc_of(&s1, a).occupied_by_earlier, dec("0"));
    }

    /// 经典例：计划1{A,B}目标3000、计划2{C,D}目标4000、计划3{A,E}目标2000，
    /// 余额 A3000/B1000/C2000/D1000/E500
    async fn setup_classic() -> (SavingPlanService, [AccountId; 5], [SavingPlanId; 3]) {
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        let b = create_asset_account(&service, "B").await;
        let c = create_asset_account(&service, "C").await;
        let dd = create_asset_account(&service, "D").await;
        let e = create_asset_account(&service, "E").await;
        add_posting(&service, "2026-06-01", a, CommodityId(1), "3000", &[]).await;
        add_posting(&service, "2026-06-01", b, CommodityId(1), "1000", &[]).await;
        add_posting(&service, "2026-06-01", c, CommodityId(1), "2000", &[]).await;
        add_posting(&service, "2026-06-01", dd, CommodityId(1), "1000", &[]).await;
        add_posting(&service, "2026-06-01", e, CommodityId(1), "500", &[]).await;

        let plan1 = create_plan(
            &service,
            "计划1",
            None,
            Some(d(2026, 8, 31)),
            CommodityId(1),
            "3000",
            &[a, b],
        )
        .await;
        let plan2 = create_plan(
            &service,
            "计划2",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "4000",
            &[c, dd],
        )
        .await;
        let plan3 = create_plan(
            &service,
            "计划3",
            None,
            Some(d(2026, 10, 31)),
            CommodityId(1),
            "2000",
            &[a, e],
        )
        .await;
        (service, [a, b, c, dd, e], [plan1, plan2, plan3])
    }

    #[tokio::test]
    async fn test_allocation_prefers_non_intersecting_accounts() {
        // spec: 为下一交集计划保留交集账户
        let (service, [a, b, _, _, _], [plan1, _, _]) = setup_classic().await;

        let s1 = service
            .get_saving_plan_status(plan1, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s1.allocated, dec("3000"));
        assert_eq!(s1.satisfaction, dec("100"));
        // 尽量不动与计划 3 交集的 A：先扣 B 1000，再扣 A 2000
        assert_eq!(alloc_of(&s1, a).allocated, dec("2000"));
        assert_eq!(alloc_of(&s1, b).allocated, dec("1000"));
    }

    #[tokio::test]
    async fn test_allocation_cascade_after_intersection_reserved() {
        // spec: 交集账户被保留后的级联
        let (service, [a, _, c, dd, e], [_, plan2, plan3]) = setup_classic().await;

        // 计划 2 无后续交集计划，按账户 id 升序占光
        let s2 = service
            .get_saving_plan_status(plan2, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s2.allocated, dec("3000"));
        assert_eq!(s2.satisfaction, dec("75"));
        assert_eq!(alloc_of(&s2, c).allocated, dec("2000"));
        assert_eq!(alloc_of(&s2, dd).allocated, dec("1000"));

        // 计划 3：A 已被计划 1 占用 2000，可用 = A 剩 1000 + E 500 = 1500
        let s3 = service
            .get_saving_plan_status(plan3, d(2026, 7, 15))
            .await
            .unwrap();
        assert_eq!(s3.allocated, dec("1500"));
        assert_eq!(s3.satisfaction, dec("75"));
        let alloc_a = alloc_of(&s3, a);
        assert_eq!(alloc_a.balance, dec("3000"));
        assert_eq!(alloc_a.occupied_by_earlier, dec("2000"));
        assert_eq!(alloc_a.allocated, dec("1000"));
        assert_eq!(alloc_of(&s3, e).allocated, dec("500"));
    }

    #[tokio::test]
    async fn test_status_accounts_detail_includes_descendants() {
        // spec: accounts 明细中每个账户的 balance 含后代、仅本币、截至查询日
        let service = setup().await;
        let bank = account_id(&service, "Assets:Bank").await;
        let checking = account_id(&service, "Assets:Bank:Checking").await;
        let plan_id = create_plan(
            &service,
            "银行存款计划",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "5000",
            &[bank],
        )
        .await;
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
        assert_eq!(status.accounts.len(), 1);
        let alloc = alloc_of(&status, bank);
        assert_eq!(alloc.balance, dec("3500"));
        assert_eq!(alloc.occupied_by_earlier, dec("0"));
        assert_eq!(alloc.allocated, dec("3500"));
        assert_eq!(status.allocated, dec("3500"));
        assert_eq!(status.satisfaction, dec("70"));
    }

    #[tokio::test]
    async fn test_list_saving_plan_statuses_matches_single() {
        // spec: 批量状态与单条状态口径一致
        let (service, _, plan_ids) = setup_classic().await;
        let date = d(2026, 7, 15);

        let list = service.list_saving_plan_statuses(date).await.unwrap();
        assert_eq!(list.len(), 3);
        for status in &list {
            let single = service
                .get_saving_plan_status(status.plan.id, date)
                .await
                .unwrap();
            assert_eq!(status.allocated, single.allocated);
            assert_eq!(status.satisfaction, single.satisfaction);
            assert!(plan_ids.contains(&status.plan.id));
        }
    }

    #[tokio::test]
    async fn test_list_saving_plan_statuses_empty() {
        // spec: 空计划列表
        let service = setup().await;
        let list = service
            .list_saving_plan_statuses(d(2026, 7, 15))
            .await
            .unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_list_saving_plan_statuses_sorted_by_checkpoint() {
        // spec: 参与分配的计划按（检查点, plan_id）升序在前，过期/永久计划在后
        let service = setup().await;
        let a = create_asset_account(&service, "A").await;
        // 创建顺序与期望输出顺序不同，验证排序不是创建顺序
        let expired = create_plan(
            &service,
            "过期计划",
            None,
            Some(d(2026, 6, 30)),
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;
        let late = create_plan(
            &service,
            "晚检查点",
            None,
            Some(d(2026, 9, 30)),
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;
        let permanent = create_plan(
            &service,
            "永久计划",
            None,
            None,
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;
        let early = create_plan(
            &service,
            "早检查点",
            None,
            Some(d(2026, 7, 31)),
            CommodityId(1),
            "1000",
            &[a],
        )
        .await;

        // 查询日 2026-07-15：expired 已过期；early/late 参与分配
        let list = service
            .list_saving_plan_statuses(d(2026, 7, 15))
            .await
            .unwrap();
        let ids: Vec<SavingPlanId> = list.iter().map(|s| s.plan.id).collect();
        // 参与者按检查点升序（early 2026-07-31 < late 2026-09-30）；
        // 不参与者（expired、permanent）在后，按 plan_id 升序
        assert_eq!(ids, vec![early, late, expired, permanent]);
        assert!(!list[0].expired && !list[1].expired);
        assert!(list[2].expired);
    }
}
