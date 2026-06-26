/// BillPosting 的角色，用于区分收支侧和资产侧
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostingRole {
    /// 收支侧（收入/支出分类）
    IncomeExpense,
    /// 资产侧（付款方式/资金来源）
    Asset,
}

impl PostingRole {
    /// 映射 key 前缀
    pub fn prefix(&self) -> &'static str {
        match self {
            PostingRole::IncomeExpense => "收支",
            PostingRole::Asset => "资产",
        }
    }

    /// 从映射 key 解析出 (PostingRole, 原始分类名)
    pub fn from_key(key: &str) -> Option<(PostingRole, &str)> {
        if let Some(cat) = key.strip_prefix("收支:") {
            Some((PostingRole::IncomeExpense, cat))
        } else if let Some(cat) = key.strip_prefix("资产:") {
            Some((PostingRole::Asset, cat))
        } else {
            None
        }
    }

    /// 生成映射 key
    pub fn to_key(&self, category: &str) -> String {
        format!("{}:{}", self.prefix(), category)
    }

    /// Import fallback 路径段（与 prefix 相同）
    pub fn import_segment(&self) -> &'static str {
        self.prefix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix() {
        assert_eq!(PostingRole::IncomeExpense.prefix(), "收支");
        assert_eq!(PostingRole::Asset.prefix(), "资产");
    }

    #[test]
    fn test_from_key_income_expense() {
        let (role, cat) = PostingRole::from_key("收支:餐饮美食").unwrap();
        assert_eq!(role, PostingRole::IncomeExpense);
        assert_eq!(cat, "餐饮美食");
    }

    #[test]
    fn test_from_key_asset() {
        let (role, cat) = PostingRole::from_key("资产:蚂蚁宝藏信用卡").unwrap();
        assert_eq!(role, PostingRole::Asset);
        assert_eq!(cat, "蚂蚁宝藏信用卡");
    }

    #[test]
    fn test_from_key_invalid() {
        assert!(PostingRole::from_key("invalid").is_none());
        assert!(PostingRole::from_key("收支").is_none()); // 缺少冒号后的分类
    }

    #[test]
    fn test_to_key() {
        assert_eq!(
            PostingRole::IncomeExpense.to_key("餐饮美食"),
            "收支:餐饮美食"
        );
        assert_eq!(
            PostingRole::Asset.to_key("蚂蚁宝藏信用卡"),
            "资产:蚂蚁宝藏信用卡"
        );
    }

    #[test]
    fn test_import_segment() {
        assert_eq!(PostingRole::IncomeExpense.import_segment(), "收支");
        assert_eq!(PostingRole::Asset.import_segment(), "资产");
    }
}
