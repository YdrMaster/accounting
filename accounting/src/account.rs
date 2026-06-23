use crate::account_type::AccountType;
use crate::id::AccountId;
use chrono::NaiveDate;
use std::collections::HashMap;

/// 账户
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// 账户唯一标识符
    pub id: AccountId,
    /// 账户名（本级名称），如 Cash
    pub name: String,
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

impl Account {
    /// 根据全量账户映射拼装完整路径，如 Assets:Bank:Checking
    pub fn display_path(&self, accounts_by_id: &HashMap<AccountId, Account>) -> String {
        let mut parts = vec![self.name.clone()];
        let mut current = self.parent_id;
        while let Some(pid) = current {
            if let Some(parent) = accounts_by_id.get(&pid) {
                parts.push(parent.name.clone());
                current = parent.parent_id;
            } else {
                break;
            }
        }
        parts.reverse();
        parts.join(":")
    }
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
            name: "Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        assert_eq!(a.name, "Cash");
        assert!(!a.is_system);
    }
}
