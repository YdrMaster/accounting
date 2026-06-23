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
}

impl std::str::FromStr for AccountType {
    type Err = String;
    fn from_str(root_name: &str) -> Result<Self, Self::Err> {
        let lower = root_name.to_lowercase();
        match lower.as_str() {
            "asset" | "assets" | "资产" => Ok(Self::Asset),
            "equity" | "权益" => Ok(Self::Equity),
            "income" | "收入" => Ok(Self::Income),
            "expense" | "expenses" | "支出" => Ok(Self::Expense),
            _ => Err(format!("unknown account root name: {}", root_name)),
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
    fn test_from_str() {
        use std::str::FromStr;

        // 英文单复数
        assert_eq!(AccountType::from_str("asset"), Ok(AccountType::Asset));
        assert_eq!(AccountType::from_str("assets"), Ok(AccountType::Asset));
        assert_eq!(AccountType::from_str("equity"), Ok(AccountType::Equity));
        assert_eq!(AccountType::from_str("income"), Ok(AccountType::Income));
        assert_eq!(AccountType::from_str("expense"), Ok(AccountType::Expense));
        assert_eq!(AccountType::from_str("expenses"), Ok(AccountType::Expense));

        // 中文
        assert_eq!(AccountType::from_str("资产"), Ok(AccountType::Asset));
        assert_eq!(AccountType::from_str("权益"), Ok(AccountType::Equity));
        assert_eq!(AccountType::from_str("收入"), Ok(AccountType::Income));
        assert_eq!(AccountType::from_str("支出"), Ok(AccountType::Expense));

        // 大小写不敏感
        assert_eq!(AccountType::from_str("ASSET"), Ok(AccountType::Asset));

        // 无效根节点名
        assert!(AccountType::from_str("foo").is_err());
        assert!(AccountType::from_str("负债").is_err());
    }
}
