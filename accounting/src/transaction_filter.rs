use crate::id::{AccountId, ChannelId, MemberId, TagId};
use chrono::NaiveDate;

/// 交易查询条件
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    /// 起始日期
    pub start_date: Option<NaiveDate>,
    /// 结束日期
    pub end_date: Option<NaiveDate>,
    /// 指定账户 ID
    pub account_id: Option<AccountId>,
    /// 指定成员 ID
    pub member_id: Option<MemberId>,
    /// 指定支付渠道 ID
    pub channel_id: Option<ChannelId>,
    /// 指定标签 ID
    pub tag_id: Option<TagId>,
    /// 关键词模糊匹配
    pub keyword: Option<String>,
    /// 是否包含分期
    pub has_installment: Option<bool>,
    /// 只包含可报销分录的交易
    pub has_reimbursable: Option<bool>,
}
