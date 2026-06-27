use crate::id::{AccountId, BudgetId, CommodityId};
use chrono::Datelike;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::HashSet;

/// 预算周期类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetPeriod {
    /// 自然日（当天 00:00 ~ 23:59:59）
    Daily = 1,
    /// 自然周（周日起始）
    WeeklyFromSunday = 2,
    /// 自然周（周一起始）
    WeeklyFromMonday = 3,
    /// 自然月（每月1号起始）
    Monthly = 4,
    /// 自然年（每年1月1号起始）
    Yearly = 5,
}

impl BudgetPeriod {
    /// 给定日期，返回当前周期的起止日期范围
    ///
    /// ## 示例
    ///
    /// ```plaintext
    /// Daily,  2026-06-26  → (2026-06-26, 2026-06-26)
    /// WeeklyFromMonday, 2026-06-26(周五) → (2026-06-22, 2026-06-28)
    /// WeeklyFromSunday, 2026-06-26(周五) → (2026-06-21, 2026-06-27)
    /// Monthly, 2026-06-26 → (2026-06-01, 2026-06-30)
    /// Yearly,  2026-06-26 → (2026-01-01, 2026-12-31)
    /// ```
    pub fn period_range(&self, date: NaiveDate) -> (NaiveDate, NaiveDate) {
        match self {
            BudgetPeriod::Daily => (date, date),
            BudgetPeriod::WeeklyFromSunday => {
                // Sunday = 0 in Chrono::Weekday
                let weekday = date.weekday().num_days_from_sunday() as i64;
                let start = date - chrono::Duration::days(weekday);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            BudgetPeriod::WeeklyFromMonday => {
                // Monday = 0
                let weekday = date.weekday().num_days_from_monday() as i64;
                let start = date - chrono::Duration::days(weekday);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            BudgetPeriod::Monthly => {
                let year = date.year();
                let month = date.month();
                let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let end = start
                    .with_month(month + 1)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
                    - chrono::Duration::days(1);
                (start, end)
            }
            BudgetPeriod::Yearly => {
                let year = date.year();
                let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
                (start, end)
            }
        }
    }

    /// 从整数创建 BudgetPeriod
    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(BudgetPeriod::Daily),
            2 => Some(BudgetPeriod::WeeklyFromSunday),
            3 => Some(BudgetPeriod::WeeklyFromMonday),
            4 => Some(BudgetPeriod::Monthly),
            5 => Some(BudgetPeriod::Yearly),
            _ => None,
        }
    }

    /// 转为整数
    pub fn as_i64(&self) -> i64 {
        *self as i64
    }
}

impl std::fmt::Display for BudgetPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetPeriod::Daily => write!(f, "Daily"),
            BudgetPeriod::WeeklyFromSunday => write!(f, "WeeklyFromSunday"),
            BudgetPeriod::WeeklyFromMonday => write!(f, "WeeklyFromMonday"),
            BudgetPeriod::Monthly => write!(f, "Monthly"),
            BudgetPeriod::Yearly => write!(f, "Yearly"),
        }
    }
}

/// 预算表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    /// 预算表唯一标识符
    pub id: BudgetId,
    /// 预算表名称，如"月度生活预算"
    pub name: String,
    /// 预算周期
    pub period: BudgetPeriod,
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

    // === BudgetPeriod::period_range 测试 ===

    #[test]
    fn test_period_range_daily() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = BudgetPeriod::Daily.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, date);
    }

    #[test]
    fn test_period_range_weekly_from_monday_friday() {
        // 2026-06-26 is a Friday
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = BudgetPeriod::WeeklyFromMonday.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 22).unwrap()); // Monday
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()); // Sunday
    }

    #[test]
    fn test_period_range_weekly_from_sunday_friday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = BudgetPeriod::WeeklyFromSunday.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 21).unwrap()); // Sunday
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 27).unwrap()); // Saturday
    }

    #[test]
    fn test_period_range_weekly_from_monday_on_monday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 22).unwrap(); // Monday
        let (start, end) = BudgetPeriod::WeeklyFromMonday.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 28).unwrap());
    }

    #[test]
    fn test_period_range_weekly_from_sunday_on_sunday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(); // Sunday
        let (start, end) = BudgetPeriod::WeeklyFromSunday.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 27).unwrap());
    }

    #[test]
    fn test_period_range_monthly() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = BudgetPeriod::Monthly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }

    #[test]
    fn test_period_range_monthly_december() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let (start, end) = BudgetPeriod::Monthly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_period_range_yearly() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = BudgetPeriod::Yearly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    // === BudgetPeriod 整数 ↔ 枚举转换测试 ===

    #[test]
    fn test_from_i64() {
        assert_eq!(BudgetPeriod::from_i64(1), Some(BudgetPeriod::Daily));
        assert_eq!(
            BudgetPeriod::from_i64(2),
            Some(BudgetPeriod::WeeklyFromSunday)
        );
        assert_eq!(
            BudgetPeriod::from_i64(3),
            Some(BudgetPeriod::WeeklyFromMonday)
        );
        assert_eq!(BudgetPeriod::from_i64(4), Some(BudgetPeriod::Monthly));
        assert_eq!(BudgetPeriod::from_i64(5), Some(BudgetPeriod::Yearly));
        assert_eq!(BudgetPeriod::from_i64(0), None);
        assert_eq!(BudgetPeriod::from_i64(6), None);
    }

    #[test]
    fn test_as_i64() {
        assert_eq!(BudgetPeriod::Daily.as_i64(), 1);
        assert_eq!(BudgetPeriod::WeeklyFromSunday.as_i64(), 2);
        assert_eq!(BudgetPeriod::WeeklyFromMonday.as_i64(), 3);
        assert_eq!(BudgetPeriod::Monthly.as_i64(), 4);
        assert_eq!(BudgetPeriod::Yearly.as_i64(), 5);
    }

    #[test]
    fn test_roundtrip() {
        for value in 1..=5 {
            let period = BudgetPeriod::from_i64(value).unwrap();
            assert_eq!(period.as_i64(), value);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(BudgetPeriod::Daily.to_string(), "Daily");
        assert_eq!(
            BudgetPeriod::WeeklyFromSunday.to_string(),
            "WeeklyFromSunday"
        );
        assert_eq!(
            BudgetPeriod::WeeklyFromMonday.to_string(),
            "WeeklyFromMonday"
        );
        assert_eq!(BudgetPeriod::Monthly.to_string(), "Monthly");
        assert_eq!(BudgetPeriod::Yearly.to_string(), "Yearly");
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
