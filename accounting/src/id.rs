use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[doc = concat!(stringify!($name), " 类型标识符")]
        pub struct $name(pub i64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(AccountId);
define_id!(TransactionId);
define_id!(PostingId);
define_id!(CommodityId);
define_id!(MemberId);
define_id!(ChannelId);
define_id!(ChannelPathId);
define_id!(TagId);
define_id!(AttachmentId);
define_id!(BudgetId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_equality() {
        let a = AccountId(1);
        let b = AccountId(1);
        let c = AccountId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_account_id_copy() {
        let a = AccountId(42);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_account_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AccountId(1));
        set.insert(AccountId(1));
        assert_eq!(set.len(), 1);
    }
}
