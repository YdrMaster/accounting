use crate::id::{ChannelId, ChannelPathId, TransactionId};

/// 渠道链路节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPathStatus {
    /// 默认状态（无特殊校验含义）
    Default = 0,
    /// 待人工校验
    Pending = 1,
    /// 已校验
    Verified = 2,
}

impl ChannelPathStatus {
    /// 将数据库存储的整数转换为枚举
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => ChannelPathStatus::Pending,
            2 => ChannelPathStatus::Verified,
            _ => ChannelPathStatus::Default,
        }
    }

    /// 将枚举转换为数据库存储的整数
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 返回状态字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelPathStatus::Default => "default",
            ChannelPathStatus::Pending => "pending",
            ChannelPathStatus::Verified => "verified",
        }
    }
}

impl std::str::FromStr for ChannelPathStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(ChannelPathStatus::Default),
            "pending" => Ok(ChannelPathStatus::Pending),
            "verified" => Ok(ChannelPathStatus::Verified),
            _ => Err(format!("unknown channel path status: {}", s)),
        }
    }
}

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
    /// 链路节点状态
    pub status: ChannelPathStatus,
}

/// 链路节点值类型（API/Service 层传递用，不含 id 和 transaction_id）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPathNode {
    /// 在链路中的位置
    pub position: i32,
    /// 渠道 ID
    pub channel_id: ChannelId,
    /// 链路节点状态
    pub status: ChannelPathStatus,
}
