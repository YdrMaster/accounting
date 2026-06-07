use crate::id::{AccountId, ChannelId, CommodityId, MemberId, PostingId, TransactionId};
use rust_decimal::Decimal;

/// 分录类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingKind {
    /// 普通分录
    Normal = 1,
    /// 退款分录（冲减原支出）
    Refund = 2,
    /// 报销分录（冲减原支出）
    Reimbursement = 3,
}

impl PostingKind {
    /// 从数据库整数值解析
    pub fn from_db(value: i32) -> Option<Self> {
        match value {
            1 => Some(PostingKind::Normal),
            2 => Some(PostingKind::Refund),
            3 => Some(PostingKind::Reimbursement),
            _ => None,
        }
    }
}

/// 分录（Posting / 端点）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    /// 分录唯一标识符
    pub id: PostingId,
    /// 所属交易 ID
    pub transaction_id: TransactionId,
    /// 所属账户 ID
    pub account_id: AccountId,
    /// 商品/货币 ID
    pub commodity_id: CommodityId,
    /// 金额（正数表示借方，负数表示贷方）
    pub amount: Decimal,
    /// 总价格（双边 Cost），单币种交易为 None
    pub cost: Option<Decimal>,
    /// cost 对应的商品 ID
    pub cost_commodity_id: Option<CommodityId>,
    /// 分录描述
    pub description: Option<String>,
    /// 关联成员 ID
    pub member_id: Option<MemberId>,
    /// 关联支付渠道 ID
    pub channel_id: Option<ChannelId>,
    /// 分录类型
    pub kind: PostingKind,
    /// 关联原分录 ID（退款/报销时指向被冲减的分录）
    pub linked_posting_id: Option<PostingId>,
    /// 累计被冲减金额（由触发器自动维护）
    pub reversal_total: Decimal,
}
