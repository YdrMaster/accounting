use crate::account::Account;
use crate::account_type::AccountType;
use crate::finance_period::FinancePeriod;
use crate::id::{AccountId, CommodityId, SavingPlanId};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::HashSet;

/// 攒钱计划表（资产存量的下限：一组账户共享一个目标金额）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavingPlan {
    /// 攒钱计划唯一标识符
    pub id: SavingPlanId,
    /// 攒钱周期（None 表示一次性/无节奏计划）
    pub period: Option<FinancePeriod>,
    /// 截止日期（None 表示永久有效）
    pub deadline: Option<NaiveDate>,
    /// 目标金额统一币种
    pub commodity_id: CommodityId,
    /// 目标金额（账户集合余额合计的下限）
    pub target_amount: Decimal,
}

/// 攒钱计划账户关联（账户集合共享一个目标金额，判定口径为集合余额合计）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavingPlanAccount {
    /// 所属攒钱计划 ID
    pub plan_id: SavingPlanId,
    /// 账户 ID（支持任意层级，含后代聚合）
    pub account_id: AccountId,
}

/// 攒钱计划错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum SavingPlanError {
    /// 攒钱计划名称不能为空
    EmptyName,
    /// 账户集合不能为空
    EmptyAccounts,
    /// 账户不存在
    AccountNotFound(AccountId),
    /// 账户重复
    DuplicateAccount(AccountId),
    /// 目标金额无效
    InvalidAmount(Decimal),
    /// 币种不存在
    CommodityNotFound(CommodityId),
    /// 攒钱计划不存在
    PlanNotFound(SavingPlanId),
    /// 账户不是资产账户（必须位于 Assets 根子树内）
    AccountNotAsset(AccountId),
    /// 数据库错误
    DatabaseError(String),
}

impl std::fmt::Display for SavingPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(f, "攒钱计划名称不能为空"),
            Self::EmptyAccounts => write!(f, "账户集合不能为空"),
            Self::AccountNotFound(id) => write!(f, "账户不存在: {id}"),
            Self::DuplicateAccount(id) => write!(f, "账户重复: {id}"),
            Self::InvalidAmount(amount) => write!(f, "目标金额无效: {amount}"),
            Self::CommodityNotFound(id) => write!(f, "币种不存在: {id}"),
            Self::PlanNotFound(id) => write!(f, "攒钱计划不存在: {id}"),
            Self::AccountNotAsset(id) => write!(f, "账户必须位于资产根账户子树内: {id}"),
            Self::DatabaseError(msg) => write!(f, "数据库错误: {msg}"),
        }
    }
}

impl std::error::Error for SavingPlanError {}

/// 验证攒钱计划和账户集合
///
/// 验证规则：
/// - 名称不能为空
/// - 账户集合至少 1 个
/// - 每个 account_id 必须在 accounts 中存在
/// - 账户集合中 account_id 不可重复
/// - 每个账户必须位于 Assets 根账户子树内（account_types 中类型为 Asset；
///   类型未解析到视为账户不存在）
/// - target_amount 必须 > 0
/// - commodity_id 必须在 commodity_ids 中存在
#[allow(clippy::too_many_arguments)]
pub fn validate_saving_plan(
    name: &str,
    target_amount: Decimal,
    account_ids: &[AccountId],
    accounts: &HashMap<AccountId, Account>,
    account_types: &HashMap<AccountId, AccountType>,
    commodity_id: CommodityId,
    commodity_ids: &HashSet<CommodityId>,
) -> Result<(), SavingPlanError> {
    if name.trim().is_empty() {
        return Err(SavingPlanError::EmptyName);
    }

    if account_ids.is_empty() {
        return Err(SavingPlanError::EmptyAccounts);
    }

    let mut seen_accounts = HashSet::new();
    for account_id in account_ids {
        if !accounts.contains_key(account_id) {
            return Err(SavingPlanError::AccountNotFound(*account_id));
        }
        if seen_accounts.contains(account_id) {
            return Err(SavingPlanError::DuplicateAccount(*account_id));
        }
        seen_accounts.insert(*account_id);

        match account_types.get(account_id) {
            Some(AccountType::Asset) => {}
            Some(_) => return Err(SavingPlanError::AccountNotAsset(*account_id)),
            None => return Err(SavingPlanError::AccountNotFound(*account_id)),
        }
    }

    if !target_amount.is_sign_positive() || target_amount.is_zero() {
        return Err(SavingPlanError::InvalidAmount(target_amount));
    }

    if !commodity_ids.contains(&commodity_id) {
        return Err(SavingPlanError::CommodityNotFound(commodity_id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account;
    use crate::account_type::AccountType;
    use crate::finance_period::FinancePeriod;
    use crate::id::{AccountId, CommodityId};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

    // === SavingPlanId 测试 ===

    #[test]
    fn test_saving_plan_id_equality() {
        assert_eq!(SavingPlanId(1), SavingPlanId(1));
        assert_ne!(SavingPlanId(1), SavingPlanId(2));
    }

    // === SavingPlan 结构体测试 ===

    #[test]
    fn test_saving_plan_recurring_instance() {
        let plan = SavingPlan {
            id: SavingPlanId(1),
            period: Some(FinancePeriod::Monthly),
            deadline: None,
            commodity_id: CommodityId(1),
            target_amount: Decimal::from_str("6000").unwrap(),
        };
        assert_eq!(plan.id, SavingPlanId(1));
        assert_eq!(plan.period, Some(FinancePeriod::Monthly));
        assert_eq!(plan.deadline, None);
        assert_eq!(plan.commodity_id, CommodityId(1));
        assert_eq!(plan.target_amount, Decimal::from_str("6000").unwrap());
    }

    #[test]
    fn test_saving_plan_one_off_instance() {
        let deadline = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let plan = SavingPlan {
            id: SavingPlanId(2),
            period: None,
            deadline: Some(deadline),
            commodity_id: CommodityId(1),
            target_amount: Decimal::from_str("5000").unwrap(),
        };
        assert_eq!(plan.period, None);
        assert_eq!(plan.deadline, Some(deadline));
    }

    // === SavingPlanAccount 结构体测试 ===

    #[test]
    fn test_saving_plan_account_instance() {
        let link = SavingPlanAccount {
            plan_id: SavingPlanId(1),
            account_id: AccountId(5),
        };
        assert_eq!(link.plan_id, SavingPlanId(1));
        assert_eq!(link.account_id, AccountId(5));
    }

    // === SavingPlanError Display 测试 ===

    #[test]
    fn test_saving_plan_error_display() {
        assert!(!SavingPlanError::EmptyName.to_string().is_empty());
        assert!(!SavingPlanError::EmptyAccounts.to_string().is_empty());
        assert!(
            SavingPlanError::AccountNotFound(AccountId(5))
                .to_string()
                .contains("5")
        );
        assert!(
            SavingPlanError::DuplicateAccount(AccountId(3))
                .to_string()
                .contains("3")
        );
        assert!(
            SavingPlanError::InvalidAmount(Decimal::ZERO)
                .to_string()
                .contains("0")
        );
        assert!(
            SavingPlanError::CommodityNotFound(CommodityId(1))
                .to_string()
                .contains("1")
        );
        assert!(
            SavingPlanError::PlanNotFound(SavingPlanId(2))
                .to_string()
                .contains("2")
        );
        assert!(
            SavingPlanError::AccountNotAsset(AccountId(7))
                .to_string()
                .contains("7")
        );
        assert!(
            SavingPlanError::DatabaseError("conn failed".to_string())
                .to_string()
                .contains("conn failed")
        );
    }

    // === validate_saving_plan 测试 ===

    fn sample_account(id: i64) -> (AccountId, Account) {
        (
            AccountId(id),
            Account {
                id: AccountId(id),
                parent_id: None,
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            },
        )
    }

    fn sample_accounts(ids: &[i64]) -> HashMap<AccountId, Account> {
        ids.iter().map(|&id| sample_account(id)).collect()
    }

    fn account_types(ids: &[i64], account_type: AccountType) -> HashMap<AccountId, AccountType> {
        ids.iter()
            .map(|&id| (AccountId(id), account_type))
            .collect()
    }

    #[test]
    fn test_validate_saving_plan_ok() {
        let accounts = sample_accounts(&[1, 2]);
        let types = account_types(&[1, 2], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert!(
            validate_saving_plan(
                "房租备用金",
                Decimal::from_str("6000").unwrap(),
                &[AccountId(1), AccountId(2)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_saving_plan_empty_name() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::EmptyName)
        );
        assert_eq!(
            validate_saving_plan(
                "   ",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::EmptyName)
        );
    }

    #[test]
    fn test_validate_saving_plan_empty_accounts() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::EmptyAccounts)
        );
    }

    #[test]
    fn test_validate_saving_plan_account_not_found() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[AccountId(99)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::AccountNotFound(AccountId(99)))
        );
    }

    #[test]
    fn test_validate_saving_plan_duplicate_account() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1), AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::DuplicateAccount(AccountId(1)))
        );
    }

    #[test]
    fn test_validate_saving_plan_invalid_target_zero() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::ZERO,
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::InvalidAmount(Decimal::ZERO))
        );
    }

    #[test]
    fn test_validate_saving_plan_invalid_target_negative() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("-100").unwrap(),
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::InvalidAmount(
                Decimal::from_str("-100").unwrap()
            ))
        );
    }

    #[test]
    fn test_validate_saving_plan_account_not_asset() {
        // 账户集合包含 Expenses 根子树账户 → 拒绝
        let accounts = sample_accounts(&[1, 2]);
        let mut types = account_types(&[1], AccountType::Asset);
        types.extend(account_types(&[2], AccountType::Expense));
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1), AccountId(2)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::AccountNotAsset(AccountId(2)))
        );
    }

    #[test]
    fn test_validate_saving_plan_account_type_missing() {
        // 账户存在但类型未解析到 → 视为账户不存在
        let accounts = sample_accounts(&[1]);
        let types = HashMap::new();
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(1),
                &commodity_ids,
            ),
            Err(SavingPlanError::AccountNotFound(AccountId(1)))
        );
    }

    #[test]
    fn test_validate_saving_plan_commodity_not_found() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Asset);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_saving_plan(
                "测试",
                Decimal::from_str("100").unwrap(),
                &[AccountId(1)],
                &accounts,
                &types,
                CommodityId(99),
                &commodity_ids,
            ),
            Err(SavingPlanError::CommodityNotFound(CommodityId(99)))
        );
    }
}
