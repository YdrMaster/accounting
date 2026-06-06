use crate::id::ChannelId;

/// 支付渠道
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub description: Option<String>,
}
