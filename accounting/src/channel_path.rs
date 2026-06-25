use crate::id::{ChannelId, ChannelPathId, TransactionId};

/// 交易链路记录（数据库行）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPath {
    /// 链路记录唯一标识符
    pub id: ChannelPathId,
    /// 所属交易 ID
    pub transaction_id: TransactionId,
    /// 在链路中的位置（从 0 开始递增）
    pub position: i32,
    /// 渠道 ID
    pub channel_id: ChannelId,
    /// 是否已对账
    pub reconciled: bool,
}

/// 链路节点值类型（API/Service 层传递用，不含 id 和 transaction_id）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPathNode {
    /// 在链路中的位置
    pub position: i32,
    /// 渠道 ID
    pub channel_id: ChannelId,
    /// 是否已对账
    pub reconciled: bool,
}
