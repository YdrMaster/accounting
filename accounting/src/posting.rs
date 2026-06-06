use crate::id::{AccountId, ChannelId, CommodityId, MemberId, PostingId, TransactionId};
use rust_decimal::Decimal;

/// 分录（Posting / 端点）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub id: PostingId,
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub commodity_id: CommodityId,
    pub amount: Decimal,
    /// 总价格（双边 Cost），单币种交易为 None
    pub cost: Option<Decimal>,
    /// cost 对应的商品 ID
    pub cost_commodity_id: Option<CommodityId>,
    pub description: Option<String>,
    pub member_id: Option<MemberId>,
    pub channel_id: Option<ChannelId>,
}
