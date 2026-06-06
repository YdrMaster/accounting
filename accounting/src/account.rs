use crate::account_type::AccountType;
use crate::id::AccountId;
use chrono::NaiveDate;

/// 账户
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// 账户唯一标识符
    pub id: AccountId,
    /// 账户全名，如 Assets:Cash
    pub full_name: String,
    /// 账户类型
    pub account_type: AccountType,
    /// 父账户 ID，根账户为 None
    pub parent_id: Option<AccountId>,
    /// 关户日期，未关闭为 None
    pub closed_at: Option<NaiveDate>,
    /// 是否为系统内置账户
    pub is_system: bool,
    /// 账单日（信用卡等账户使用）
    pub billing_day: Option<u8>,
    /// 还款日（信用卡等账户使用）
    pub repayment_day: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_type::AccountType;
    use crate::id::AccountId;
    #[test]
    fn test_account_fields() {
        let a = Account {
            id: AccountId(1),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        assert_eq!(a.full_name, "Assets:Cash");
        assert!(!a.is_system);
    }
}
