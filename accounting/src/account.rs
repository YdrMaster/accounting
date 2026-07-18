use crate::id::AccountId;
use chrono::NaiveDate;
use std::collections::HashMap;

/// 账户
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// 账户唯一标识符
    pub id: AccountId,
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
    /// 根据全量账户映射与显示名映射拼装完整路径，如 Assets:Bank:Checking
    pub fn display_path(
        &self,
        accounts_by_id: &HashMap<AccountId, Account>,
        display_names: &HashMap<AccountId, String>,
    ) -> String {
        let name = display_names
            .get(&self.id)
            .map(|s| s.as_str())
            .unwrap_or("?");
        let mut parts = vec![name.to_string()];
        let mut current = self.parent_id;
        while let Some(pid) = current {
            if let Some(parent) = accounts_by_id.get(&pid) {
                let pname = display_names.get(&pid).map(|s| s.as_str()).unwrap_or("?");
                parts.push(pname.to_string());
                current = parent.parent_id;
            } else {
                // 孤儿父 id（映射中不存在）：直接终止，不追加占位段
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
    use crate::id::AccountId;
    #[test]
    fn test_account_fields() {
        let a = Account {
            id: AccountId(1),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        assert!(!a.is_system);
    }

    fn account(id: i64, parent_id: Option<i64>) -> Account {
        Account {
            id: AccountId(id),
            parent_id: parent_id.map(AccountId),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    #[test]
    fn test_display_path_full_chain() {
        let accounts: HashMap<AccountId, Account> = [
            (AccountId(1), account(1, None)),
            (AccountId(2), account(2, Some(1))),
            (AccountId(3), account(3, Some(2))),
        ]
        .into_iter()
        .collect();
        let names: HashMap<AccountId, String> = [
            (AccountId(1), "Assets".to_string()),
            (AccountId(2), "Bank".to_string()),
            (AccountId(3), "Checking".to_string()),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            accounts[&AccountId(3)].display_path(&accounts, &names),
            "Assets:Bank:Checking"
        );
        assert_eq!(
            accounts[&AccountId(1)].display_path(&accounts, &names),
            "Assets"
        );
    }

    #[test]
    fn test_display_path_missing_name_uses_placeholder() {
        let accounts: HashMap<AccountId, Account> = [
            (AccountId(1), account(1, None)),
            (AccountId(2), account(2, Some(1))),
        ]
        .into_iter()
        .collect();
        let names: HashMap<AccountId, String> = HashMap::new();

        assert_eq!(
            accounts[&AccountId(2)].display_path(&accounts, &names),
            "?:?"
        );
    }

    #[test]
    fn test_display_path_orphan_parent_id_no_leading_placeholder() {
        // 父 id 在映射中不存在：不应产生多余的 "?:" 前导段
        let accounts: HashMap<AccountId, Account> =
            [(AccountId(2), account(2, Some(99)))].into_iter().collect();
        let names: HashMap<AccountId, String> = [(AccountId(2), "Checking".to_string())]
            .into_iter()
            .collect();

        assert_eq!(
            accounts[&AccountId(2)].display_path(&accounts, &names),
            "Checking"
        );
    }
}
