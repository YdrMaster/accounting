use rust_decimal::Decimal;

/// BillPosting 的角色，用于区分收支侧和资产侧
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostingRole {
    /// 收支侧（收入/支出分类）
    IncomeExpense,
    /// 资产侧（付款方式/资金来源）
    Asset,
}

impl PostingRole {
    /// 判断给定分类是否是退款
    pub fn is_refund_category(category: &str) -> bool {
        category == "退款" || category == "Refund"
    }

    /// 生成映射 key
    ///
    /// - Asset 角色使用 `Assets:<category>`
    /// - IncomeExpense 角色：退款或金额为正则使用 `Expenses:<category>`，金额为负则使用 `Income:<category>`
    pub fn to_key(&self, category: &str, amount: Decimal) -> String {
        format!("{}:{}", self.fallback_root(category, amount), category)
    }

    /// 从映射 key 解析出 (PostingRole, 原始分类名)
    pub fn from_key(key: &str) -> Option<(PostingRole, &str)> {
        if let Some(cat) = key.strip_prefix("Assets:") {
            Some((PostingRole::Asset, cat))
        } else if let Some(cat) = key.strip_prefix("Income:") {
            Some((PostingRole::IncomeExpense, cat))
        } else if let Some(cat) = key.strip_prefix("Expenses:") {
            Some((PostingRole::IncomeExpense, cat))
        } else {
            None
        }
    }

    /// 返回 Import fallback 账户路径应使用的 beancount 根账户名
    ///
    /// - Asset 角色返回 `Assets`
    /// - IncomeExpense 角色：退款或金额为正返回 `Expenses`，金额为负返回 `Income`
    pub fn fallback_root(&self, category: &str, amount: Decimal) -> &'static str {
        match self {
            PostingRole::Asset => "Assets",
            PostingRole::IncomeExpense => {
                if Self::is_refund_category(category) || amount > Decimal::ZERO {
                    "Expenses"
                } else {
                    "Income"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_to_key_asset() {
        assert_eq!(
            PostingRole::Asset.to_key("蚂蚁宝藏信用卡", Decimal::from_str("-4.80").unwrap()),
            "Assets:蚂蚁宝藏信用卡"
        );
    }

    #[test]
    fn test_to_key_expense() {
        assert_eq!(
            PostingRole::IncomeExpense.to_key("餐饮美食", Decimal::from_str("4.80").unwrap()),
            "Expenses:餐饮美食"
        );
    }

    #[test]
    fn test_to_key_income() {
        assert_eq!(
            PostingRole::IncomeExpense.to_key("工资", Decimal::from_str("-100.00").unwrap()),
            "Income:工资"
        );
    }

    #[test]
    fn test_to_key_refund_as_expense() {
        assert_eq!(
            PostingRole::IncomeExpense.to_key("退款", Decimal::from_str("-36.75").unwrap()),
            "Expenses:退款"
        );
    }

    #[test]
    fn test_from_key_asset() {
        let (role, cat) = PostingRole::from_key("Assets:蚂蚁宝藏信用卡").unwrap();
        assert_eq!(role, PostingRole::Asset);
        assert_eq!(cat, "蚂蚁宝藏信用卡");
    }

    #[test]
    fn test_from_key_income() {
        let (role, cat) = PostingRole::from_key("Income:工资").unwrap();
        assert_eq!(role, PostingRole::IncomeExpense);
        assert_eq!(cat, "工资");
    }

    #[test]
    fn test_from_key_expenses() {
        let (role, cat) = PostingRole::from_key("Expenses:餐饮美食").unwrap();
        assert_eq!(role, PostingRole::IncomeExpense);
        assert_eq!(cat, "餐饮美食");
    }

    #[test]
    fn test_from_key_invalid() {
        assert!(PostingRole::from_key("invalid").is_none());
        assert!(PostingRole::from_key("Assets").is_none()); // 缺少冒号后的分类
    }

    #[test]
    fn test_fallback_root() {
        assert_eq!(
            PostingRole::Asset.fallback_root("蚂蚁宝藏信用卡", Decimal::from_str("-4.80").unwrap()),
            "Assets"
        );
        assert_eq!(
            PostingRole::IncomeExpense
                .fallback_root("餐饮美食", Decimal::from_str("4.80").unwrap()),
            "Expenses"
        );
        assert_eq!(
            PostingRole::IncomeExpense.fallback_root("工资", Decimal::from_str("-100.00").unwrap()),
            "Income"
        );
        assert_eq!(
            PostingRole::IncomeExpense.fallback_root("退款", Decimal::from_str("-36.75").unwrap()),
            "Expenses"
        );
    }
}
