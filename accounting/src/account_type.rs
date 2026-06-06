/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    Asset = 1,
    Liability = 2,
    Equity = 3,
    Income = 4,
    Expense = 5,
}

impl AccountType {
    pub fn is_permanent(self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Liability)
    }

    pub fn close_conditions(self) -> &'static str {
        match self {
            AccountType::Asset | AccountType::Liability => "余额为零",
            AccountType::Income => "无限制",
            AccountType::Expense | AccountType::Equity => "余额为零",
        }
    }
}

/// 分期方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallmentMethod {
    EqualPrincipalAndInterest = 1,
    EqualPrincipal = 2,
    InterestFree = 3,
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
