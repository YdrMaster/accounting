use crate::id::{AccountId, ChannelId, MemberId};

/// 账户映射 — 绑定在 (成员, 渠道) 上的分类字符串→账户编号映射
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountMapping {
    /// 成员 ID
    pub member_id: MemberId,
    /// 渠道 ID
    pub channel_id: ChannelId,
    /// 映射 key，格式为 "收支:<分类>" 或 "资产:<付款方式>"
    pub category: String,
    /// 目标账户 ID
    pub account_id: AccountId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_mapping_fields() {
        let m = AccountMapping {
            member_id: MemberId(1),
            channel_id: ChannelId(2),
            category: "收支:餐饮美食".to_string(),
            account_id: AccountId(42),
        };
        assert_eq!(m.member_id, MemberId(1));
        assert_eq!(m.channel_id, ChannelId(2));
        assert_eq!(m.category, "收支:餐饮美食");
        assert_eq!(m.account_id, AccountId(42));
    }
}
