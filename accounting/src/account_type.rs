/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// 资产类账户
    Asset = 1,
    /// 负债类账户
    Liability = 2,
    /// 权益类账户
    Equity = 3,
    /// 收入类账户
    Income = 4,
    /// 支出类账户
    Expense = 5,
}

impl AccountType {
    /// 是否为永久账户（Asset / Liability）
    pub fn is_permanent(self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Liability)
    }

    /// 返回该类型账户的关闭条件说明
    pub fn close_conditions(self) -> &'static str {
        match self {
            AccountType::Asset | AccountType::Liability => "余额为零",
            AccountType::Income => "无限制",
            AccountType::Expense | AccountType::Equity => "余额为零",
        }
    }

    /// 返回本地化的显示名称
    pub fn display_name(self) -> String {
        let key = match self {
            AccountType::Asset => "account_type_asset",
            AccountType::Liability => "account_type_liability",
            AccountType::Equity => "account_type_equity",
            AccountType::Income => "account_type_income",
            AccountType::Expense => "account_type_expense",
        };
        rust_i18n::t!(key).to_string()
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
    fn test_account_type_is_permanent() {
        assert!(AccountType::Asset.is_permanent());
        assert!(AccountType::Liability.is_permanent());
        assert!(!AccountType::Income.is_permanent());
        assert!(!AccountType::Expense.is_permanent());
    }

    #[test]
    fn test_account_type_close_conditions() {
        assert_eq!(AccountType::Asset.close_conditions(), "余额为零");
        assert_eq!(AccountType::Income.close_conditions(), "无限制");
    }
}
