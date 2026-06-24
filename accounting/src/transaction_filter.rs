use crate::id::{AccountId, ChannelId, MemberId, TagId};
use chrono::NaiveDate;

/// 交易查询条件
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    /// 起始日期
    pub start_date: Option<NaiveDate>,
    /// 结束日期
    pub end_date: Option<NaiveDate>,
    /// 指定账户 ID（多选，空 = 不筛选）
    pub account_ids: Vec<AccountId>,
    /// 指定成员 ID（多选，空 = 不筛选）
    pub member_ids: Vec<MemberId>,
    /// 指定支付渠道 ID（多选，空 = 不筛选）
    pub channel_ids: Vec<ChannelId>,
    /// 指定标签 ID（多选，空 = 不筛选）
    pub tag_ids: Vec<TagId>,
    /// 关键词模糊匹配
    pub keyword: Option<String>,
    /// 只包含可报销分录的交易
    pub has_reimbursable: Option<bool>,
}
