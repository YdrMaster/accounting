use crate::id::{MemberId, TransactionId};
use chrono::NaiveDateTime;

/// 交易
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// 交易唯一标识符
    pub id: TransactionId,
    /// 交易时间
    pub date_time: NaiveDateTime,
    /// 交易描述
    pub description: String,
    /// 关联成员 ID
    pub member_id: Option<MemberId>,
    /// 支付渠道 ID
    pub channel_id: Option<crate::id::ChannelId>,
    /// 是否为模板交易
    pub is_template: bool,
}
