use crate::id::{AccountId, ChannelId, MemberId, TagId};
use chrono::NaiveDate;

/// 交易查询条件
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub account_id: Option<AccountId>,
    pub member_id: Option<MemberId>,
    pub channel_id: Option<ChannelId>,
    pub tag_id: Option<TagId>,
    pub keyword: Option<String>,
    pub has_installment: Option<bool>,
    pub is_template: Option<bool>,
}
