use crate::id::{AccountId, ChannelId};

/// 支付渠道
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    /// 渠道唯一标识符
    pub id: ChannelId,
    /// 渠道名称
    pub name: String,
    /// 渠道描述
    pub description: Option<String>,
    /// 关联资产账户 ID（可选一对一）
    pub account_id: Option<AccountId>,
}
