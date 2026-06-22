/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// 资产类账户
    Asset = 1,
    /// 权益类账户
    Equity = 2,
    /// 收入类账户
    Income = 3,
    /// 支出类账户
    Expense = 4,
}

impl AccountType {
    /// 返回该类型账户的关闭条件说明
    pub fn close_conditions(self) -> String {
        match self {
            AccountType::Asset => rust_i18n::t!("close_condition_balance_zero").to_string(),
            AccountType::Equity | AccountType::Income | AccountType::Expense => {
                rust_i18n::t!("close_condition_unlimited").to_string()
            }
        }
    }

    /// 返回本地化的显示名称
    pub fn display_name(self) -> String {
        let key = match self {
            AccountType::Asset => "account_type_asset",
            AccountType::Equity => "account_type_equity",
            AccountType::Income => "account_type_income",
            AccountType::Expense => "account_type_expense",
        };
        rust_i18n::t!(key).to_string()
    }

    /// 根据账户名前缀解析账户类型（支持中英文、单复数）
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        let lower = prefix.to_lowercase();
        match lower.as_str() {
            // 英文（单复数兼容）
            "asset" | "assets" => Some(Self::Asset),
            "equity" => Some(Self::Equity),
            "income" => Some(Self::Income),
            "expense" | "expenses" => Some(Self::Expense),
            // 中文（与 seed 数据和 display_name 一致）
            "资产" => Some(Self::Asset),
            "权益" => Some(Self::Equity),
            "收入" => Some(Self::Income),
            "支出" => Some(Self::Expense),
            _ => None,
        }
    }
}

/// 分期方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallmentMethod {
    /// 等额本息
    EqualPrincipalAndInterest = 1,
    /// 等额本金
    EqualPrincipal = 2,
    /// 免息分期
    InterestFree = 3,
    /// 自定义分期
    Custom = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_close_conditions() {
        assert_eq!(
            AccountType::Asset.close_conditions(),
            rust_i18n::t!("close_condition_balance_zero").to_string()
        );
        assert_eq!(
            AccountType::Equity.close_conditions(),
            rust_i18n::t!("close_condition_unlimited").to_string()
        );
        assert_eq!(
            AccountType::Income.close_conditions(),
            rust_i18n::t!("close_condition_unlimited").to_string()
        );
        assert_eq!(
            AccountType::Expense.close_conditions(),
            rust_i18n::t!("close_condition_unlimited").to_string()
        );
    }

    #[test]
    fn test_display_name() {
        assert_eq!(
            AccountType::Asset.display_name(),
            rust_i18n::t!("account_type_asset").to_string()
        );
        assert_eq!(
            AccountType::Equity.display_name(),
            rust_i18n::t!("account_type_equity").to_string()
        );
        assert_eq!(
            AccountType::Income.display_name(),
            rust_i18n::t!("account_type_income").to_string()
        );
        assert_eq!(
            AccountType::Expense.display_name(),
            rust_i18n::t!("account_type_expense").to_string()
        );
    }

    #[test]
    fn test_from_prefix() {
        // 英文单复数
        assert_eq!(AccountType::from_prefix("asset"), Some(AccountType::Asset));
        assert_eq!(AccountType::from_prefix("assets"), Some(AccountType::Asset));
        assert_eq!(
            AccountType::from_prefix("equity"),
            Some(AccountType::Equity)
        );
        assert_eq!(
            AccountType::from_prefix("income"),
            Some(AccountType::Income)
        );
        assert_eq!(
            AccountType::from_prefix("expense"),
            Some(AccountType::Expense)
        );
        assert_eq!(
            AccountType::from_prefix("expenses"),
            Some(AccountType::Expense)
        );

        // 中文
        assert_eq!(AccountType::from_prefix("资产"), Some(AccountType::Asset));
        assert_eq!(AccountType::from_prefix("权益"), Some(AccountType::Equity));
        assert_eq!(AccountType::from_prefix("收入"), Some(AccountType::Income));
        assert_eq!(AccountType::from_prefix("支出"), Some(AccountType::Expense));

        // 大小写不敏感
        assert_eq!(AccountType::from_prefix("ASSET"), Some(AccountType::Asset));

        // 无效前缀
        assert_eq!(AccountType::from_prefix("foo"), None);
        assert_eq!(AccountType::from_prefix("负债"), None);
    }
}
