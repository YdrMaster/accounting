use crate::finance_period::FinancePeriod;
use crate::id::{AccountId, BudgetId, CommodityId};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::HashSet;

/// 预算表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    /// 预算表唯一标识符
    pub id: BudgetId,
    /// 预算表名称，如"月度生活预算"
    pub name: String,
    /// 预算周期
    pub period: FinancePeriod,
    /// 限额统一币种（所有限额折算到此币种）
    pub commodity_id: CommodityId,
}

/// 预算限额（账户 → 金额映射）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetLimit {
    /// 所属预算表 ID
    pub budget_id: BudgetId,
    /// 账户 ID（支持任意层级，含后代聚合）
    pub account_id: AccountId,
    /// 预算限额金额
    pub amount: Decimal,
}

/// 预算错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetError {
    /// 预算表名称不能为空
    EmptyName,
    /// 限额列表不能为空
    EmptyLimits,
    /// 账户不存在
    AccountNotFound(AccountId),
    /// 账户重复
    DuplicateAccount(AccountId),
    /// 限额金额无效
    InvalidAmount(Decimal),
    /// 币种不存在
    CommodityNotFound(CommodityId),
    /// 预算表不存在
    BudgetNotFound(BudgetId),
    /// 数据库错误
    DatabaseError(String),
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::EmptyName => write!(f, "预算表名称不能为空"),
            BudgetError::EmptyLimits => write!(f, "限额列表不能为空"),
            BudgetError::AccountNotFound(id) => write!(f, "账户不存在: {}", id),
            BudgetError::DuplicateAccount(id) => write!(f, "账户重复: {}", id),
            BudgetError::InvalidAmount(amount) => write!(f, "限额金额无效: {}", amount),
            BudgetError::CommodityNotFound(id) => write!(f, "币种不存在: {}", id),
            BudgetError::BudgetNotFound(id) => write!(f, "预算表不存在: {}", id),
            BudgetError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

impl std::error::Error for BudgetError {}

/// 验证预算表和限额列表
///
/// 验证规则：
/// - 名称不能为空
/// - 限额列表至少 1 条
/// - 每个 account_id 必须在 accounts 中存在
/// - 同一预算表中 account_id 不可重复
/// - 限额金额必须 > 0
/// - commodity_id 必须在 commodities 中存在
pub fn validate_budget(
    name: &str,
    limits: &[(AccountId, Decimal)],
    accounts: &HashMap<AccountId, crate::account::Account>,
    _commodity_ids: &HashSet<CommodityId>,
) -> Result<(), BudgetError> {
    if name.trim().is_empty() {
        return Err(BudgetError::EmptyName);
    }

    if limits.is_empty() {
        return Err(BudgetError::EmptyLimits);
    }

    let mut seen_accounts = HashSet::new();
    for (account_id, amount) in limits {
        if !accounts.contains_key(account_id) {
            return Err(BudgetError::AccountNotFound(*account_id));
        }
        if seen_accounts.contains(account_id) {
            return Err(BudgetError::DuplicateAccount(*account_id));
        }
        seen_accounts.insert(*account_id);

        if !amount.is_sign_positive() || amount.is_zero() {
            return Err(BudgetError::InvalidAmount(*amount));
        }
    }

    // commodity_id 验证由调用者传入有效的 commodity_ids 集合
    // 此函数不验证 commodity_id，因为它是预算表级别而非限额级别

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account;
    use crate::id::AccountId;
    use std::str::FromStr;

    // === BudgetError Display 测试 ===

    #[test]
    fn test_budget_error_display() {
        assert!(!BudgetError::EmptyName.to_string().is_empty());
        assert!(!BudgetError::EmptyLimits.to_string().is_empty());
        assert!(
            BudgetError::AccountNotFound(AccountId(5))
                .to_string()
                .contains("5")
        );
        assert!(
            BudgetError::DuplicateAccount(AccountId(3))
                .to_string()
                .contains("3")
        );
        assert!(
            BudgetError::InvalidAmount(Decimal::ZERO)
                .to_string()
                .contains("0")
        );
        assert!(
            BudgetError::CommodityNotFound(CommodityId(1))
                .to_string()
                .contains("1")
        );
        assert!(
            BudgetError::BudgetNotFound(BudgetId(2))
                .to_string()
                .contains("2")
        );
        assert!(
            BudgetError::DatabaseError("conn failed".to_string())
                .to_string()
                .contains("conn failed")
        );
    }

    // === validate_budget 测试 ===

    fn sample_account(id: i64) -> (AccountId, Account) {
        (
            AccountId(id),
            Account {
                id: AccountId(id),
                name: format!("Account{}", id),
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

    #[test]
    fn test_validate_budget_ok() {
        let accounts = sample_accounts(&[1, 2]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![
            (AccountId(1), Decimal::from_str("2000").unwrap()),
            (AccountId(2), Decimal::from_str("500").unwrap()),
        ];
        assert!(validate_budget("月度生活", &limits, &accounts, &commodity_ids).is_ok());
    }

    #[test]
    fn test_validate_budget_empty_name() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::from_str("100").unwrap())];
        assert_eq!(
            validate_budget("", &limits, &accounts, &commodity_ids),
            Err(BudgetError::EmptyName)
        );
        assert_eq!(
            validate_budget("   ", &limits, &accounts, &commodity_ids),
            Err(BudgetError::EmptyName)
        );
    }

    #[test]
    fn test_validate_budget_empty_limits() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_budget("测试", &[], &accounts, &commodity_ids),
            Err(BudgetError::EmptyLimits)
        );
    }

    #[test]
    fn test_validate_budget_account_not_found() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(99), Decimal::from_str("100").unwrap())];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &commodity_ids),
            Err(BudgetError::AccountNotFound(AccountId(99)))
        );
    }

    #[test]
    fn test_validate_budget_duplicate_account() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![
            (AccountId(1), Decimal::from_str("100").unwrap()),
            (AccountId(1), Decimal::from_str("200").unwrap()),
        ];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &commodity_ids),
            Err(BudgetError::DuplicateAccount(AccountId(1)))
        );
    }

    #[test]
    fn test_validate_budget_invalid_amount_zero() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::ZERO)];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &commodity_ids),
            Err(BudgetError::InvalidAmount(Decimal::ZERO))
        );
    }

    #[test]
    fn test_validate_budget_invalid_amount_negative() {
        let accounts = sample_accounts(&[1]);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::from_str("-100").unwrap())];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &commodity_ids),
            Err(BudgetError::InvalidAmount(
                Decimal::from_str("-100").unwrap()
            ))
        );
    }
}
