use crate::id::{AttachmentId, TransactionId};

/// 附件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: AttachmentId,
    pub transaction_id: TransactionId,
    pub filename: String,
    pub data: Vec<u8>,
}
