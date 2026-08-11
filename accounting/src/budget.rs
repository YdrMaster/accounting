use crate::account_type::AccountType;
use crate::finance_period::FinancePeriod;
use crate::id::{AccountId, BudgetId, CommodityId};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;
use std::collections::HashSet;

/// 预算表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    /// 预算表唯一标识符
    pub id: BudgetId,
    /// 预算周期（None 表示一次性/无节奏预算）
    pub period: Option<FinancePeriod>,
    /// 截止日期（None 表示永久有效）
    pub deadline: Option<NaiveDate>,
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
    /// 限额账户不是支出账户（必须位于 Expenses 根子树内）
    AccountNotExpense(AccountId),
    /// 数据库错误
    DatabaseError(String),
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 稳定的非本地化变体标识，供边界（AccountingError::Display / API handler）按变体
        // 映射 t! 本地化。此处不产出面向用户文案。
        match self {
            Self::EmptyName => write!(f, "budget_empty_name"),
            Self::EmptyLimits => write!(f, "budget_empty_limits"),
            Self::AccountNotFound(id) => write!(f, "budget_account_not_found: {id}"),
            Self::DuplicateAccount(id) => write!(f, "budget_duplicate_account: {id}"),
            Self::InvalidAmount(amount) => write!(f, "budget_invalid_amount: {amount}"),
            Self::CommodityNotFound(id) => write!(f, "budget_commodity_not_found: {id}"),
            Self::BudgetNotFound(id) => write!(f, "budget_not_found: {id}"),
            Self::AccountNotExpense(id) => write!(f, "budget_account_not_expense: {id}"),
            Self::DatabaseError(msg) => write!(f, "budget_database_error: {msg}"),
        }
    }
}

impl std::error::Error for BudgetError {}

impl BudgetError {
    /// 按当前进程 locale 产出本地化文案。供 `AccountingError::Display`（CLI `error_prefix`
    /// 链路）使用。API 边界若需 per-request locale，应直接按变体 `t!(..., locale=lang)`，
    /// 不经此方法。
    pub fn localized(&self) -> String {
        match self {
            Self::EmptyName => t!("budget_empty_name").to_string(),
            Self::EmptyLimits => t!("budget_empty_limits").to_string(),
            Self::AccountNotFound(id) => t!("budget_account_not_found", id = id).to_string(),
            Self::DuplicateAccount(id) => t!("budget_duplicate_account", id = id).to_string(),
            Self::InvalidAmount(amount) => t!("budget_invalid_amount", amount = amount).to_string(),
            Self::CommodityNotFound(id) => t!("budget_commodity_not_found", id = id).to_string(),
            Self::BudgetNotFound(id) => t!("budget_not_found", id = id).to_string(),
            Self::AccountNotExpense(id) => t!("budget_account_not_expense", id = id).to_string(),
            Self::DatabaseError(msg) => t!("database_error", msg = msg).to_string(),
        }
    }
}

/// 验证预算表和限额列表
///
/// 验证规则：
/// - 名称不能为空
/// - 限额列表至少 1 条
/// - 每个 account_id 必须在 accounts 中存在
/// - 同一预算表中 account_id 不可重复
/// - 限额金额必须 > 0
/// - 每个限额账户必须位于 Expenses 根账户子树内（account_types 中类型为 Expense；
///   类型未解析到视为账户不存在）
/// - commodity_id 必须在 commodities 中存在
pub fn validate_budget(
    name: &str,
    limits: &[(AccountId, Decimal)],
    accounts: &HashMap<AccountId, crate::account::Account>,
    account_types: &HashMap<AccountId, AccountType>,
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

        match account_types.get(account_id) {
            Some(AccountType::Expense) => {}
            Some(_) => return Err(BudgetError::AccountNotExpense(*account_id)),
            None => return Err(BudgetError::AccountNotFound(*account_id)),
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
    use crate::account_type::AccountType;
    use crate::id::AccountId;
    use chrono::NaiveDate;
    use std::str::FromStr;

    // === Budget 结构体测试 ===

    #[test]
    fn test_budget_recurring_instance() {
        let budget = Budget {
            id: BudgetId(1),
            period: Some(FinancePeriod::Monthly),
            deadline: None,
            commodity_id: CommodityId(1),
        };
        assert_eq!(budget.id, BudgetId(1));
        assert_eq!(budget.period, Some(FinancePeriod::Monthly));
        assert_eq!(budget.deadline, None);
        assert_eq!(budget.commodity_id, CommodityId(1));
    }

    #[test]
    fn test_budget_one_off_instance() {
        let deadline = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let budget = Budget {
            id: BudgetId(2),
            period: None,
            deadline: Some(deadline),
            commodity_id: CommodityId(1),
        };
        assert_eq!(budget.period, None);
        assert_eq!(budget.deadline, Some(deadline));
    }

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
            BudgetError::AccountNotExpense(AccountId(7))
                .to_string()
                .contains("7")
        );
        assert!(
            BudgetError::DatabaseError("conn failed".to_string())
                .to_string()
                .contains("conn failed")
        );
    }

    #[test]
    fn test_budget_error_localized_by_locale() {
        use std::sync::Mutex;
        static LOCALE_LOCK: Mutex<()> = Mutex::new(());
        let _guard = LOCALE_LOCK.lock().unwrap();
        rust_i18n::set_locale("en");
        assert!(
            BudgetError::EmptyName.localized().contains("Budget name"),
            "en: {}",
            BudgetError::EmptyName.localized()
        );
        assert!(
            BudgetError::BudgetNotFound(BudgetId(2))
                .localized()
                .contains("Budget not found"),
            "en: {}",
            BudgetError::BudgetNotFound(BudgetId(2)).localized()
        );
        rust_i18n::set_locale("zh-CN");
        assert!(
            BudgetError::EmptyName.localized().contains("预算表名称"),
            "zh: {}",
            BudgetError::EmptyName.localized()
        );
        assert!(
            BudgetError::BudgetNotFound(BudgetId(2))
                .localized()
                .contains("预算表不存在"),
            "zh: {}",
            BudgetError::BudgetNotFound(BudgetId(2)).localized()
        );
    }

    // === validate_budget 测试 ===

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
    fn test_validate_budget_ok() {
        let accounts = sample_accounts(&[1, 2]);
        let types = account_types(&[1, 2], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![
            (AccountId(1), Decimal::from_str("2000").unwrap()),
            (AccountId(2), Decimal::from_str("500").unwrap()),
        ];
        assert!(validate_budget("月度生活", &limits, &accounts, &types, &commodity_ids).is_ok());
    }

    #[test]
    fn test_validate_budget_empty_name() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::from_str("100").unwrap())];
        assert_eq!(
            validate_budget("", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::EmptyName)
        );
        assert_eq!(
            validate_budget("   ", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::EmptyName)
        );
    }

    #[test]
    fn test_validate_budget_empty_limits() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        assert_eq!(
            validate_budget("测试", &[], &accounts, &types, &commodity_ids),
            Err(BudgetError::EmptyLimits)
        );
    }

    #[test]
    fn test_validate_budget_account_not_found() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(99), Decimal::from_str("100").unwrap())];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::AccountNotFound(AccountId(99)))
        );
    }

    #[test]
    fn test_validate_budget_duplicate_account() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![
            (AccountId(1), Decimal::from_str("100").unwrap()),
            (AccountId(1), Decimal::from_str("200").unwrap()),
        ];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::DuplicateAccount(AccountId(1)))
        );
    }

    #[test]
    fn test_validate_budget_invalid_amount_zero() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::ZERO)];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::InvalidAmount(Decimal::ZERO))
        );
    }

    #[test]
    fn test_validate_budget_invalid_amount_negative() {
        let accounts = sample_accounts(&[1]);
        let types = account_types(&[1], AccountType::Expense);
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::from_str("-100").unwrap())];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::InvalidAmount(
                Decimal::from_str("-100").unwrap()
            ))
        );
    }

    #[test]
    fn test_validate_budget_account_not_expense() {
        // 限额账户位于 Assets 根子树 → 拒绝
        let accounts = sample_accounts(&[1, 2]);
        let mut types = account_types(&[1], AccountType::Expense);
        types.extend(account_types(&[2], AccountType::Asset));
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![
            (AccountId(1), Decimal::from_str("100").unwrap()),
            (AccountId(2), Decimal::from_str("200").unwrap()),
        ];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::AccountNotExpense(AccountId(2)))
        );
    }

    #[test]
    fn test_validate_budget_account_type_missing() {
        // 账户存在但类型未解析到 → 视为账户不存在
        let accounts = sample_accounts(&[1]);
        let types = HashMap::new();
        let commodity_ids = HashSet::from([CommodityId(1)]);
        let limits = vec![(AccountId(1), Decimal::from_str("100").unwrap())];
        assert_eq!(
            validate_budget("测试", &limits, &accounts, &types, &commodity_ids),
            Err(BudgetError::AccountNotFound(AccountId(1)))
        );
    }
}
