use crate::id::{MemberId, TransactionId};
use chrono::NaiveDate;

/// 交易
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub date: NaiveDate,
    pub description: String,
    pub member_id: Option<MemberId>,
    pub is_template: bool,
}
