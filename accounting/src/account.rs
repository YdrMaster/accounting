use crate::account_type::AccountType;
use crate::id::AccountId;
use chrono::NaiveDate;

/// 账户
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub full_name: String,
    pub account_type: AccountType,
    pub parent_id: Option<AccountId>,
    pub opened_at: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub is_system: bool,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_type::AccountType;
    use crate::id::AccountId;
    use chrono::NaiveDate;

    #[test]
    fn test_account_fields() {
        let a = Account {
            id: AccountId(1),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            opened_at: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        assert_eq!(a.full_name, "Assets:Cash");
        assert!(!a.is_system);
    }
}
