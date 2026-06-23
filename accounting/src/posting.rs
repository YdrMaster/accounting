use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
use rust_decimal::Decimal;

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
    /// 可报销标记（仅 Expense 类账户可设置）
    pub is_reimbursable: bool,
    /// 关联原分录 ID（非空表示该分录是冲减分录）
    pub linked_posting_id: Option<PostingId>,
    /// 累计被冲减金额（由触发器自动维护）
    pub reversal_total: Decimal,
}
