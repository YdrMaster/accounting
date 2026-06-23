use crate::id::{ChannelId, MemberId, TransactionId};
use chrono::NaiveDateTime;

/// 交易类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionKind {
    /// 普通交易
    Normal = 1,
    /// 退款交易
    Refund = 2,
    /// 报销交易
    Reimbursement = 3,
}

impl TransactionKind {
    /// 从数据库整数值解析
    pub fn from_db(value: i32) -> Option<Self> {
        match value {
            1 => Some(TransactionKind::Normal),
            2 => Some(TransactionKind::Refund),
            3 => Some(TransactionKind::Reimbursement),
            _ => None,
        }
    }
}

/// 交易
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// 交易唯一标识符
    pub id: TransactionId,
    /// 交易时间
    pub date_time: NaiveDateTime,
    /// 交易描述
    pub description: String,
    /// 交易类型（普通/退款/报销）
    pub kind: TransactionKind,
    /// 关联成员 ID
    pub member_id: Option<MemberId>,
    /// 支付渠道 ID
    pub channel_id: Option<ChannelId>,
}
