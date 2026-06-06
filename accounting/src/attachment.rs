use crate::id::{AttachmentId, TransactionId};

/// 附件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// 附件唯一标识符
    pub id: AttachmentId,
    /// 所属交易 ID
    pub transaction_id: TransactionId,
    /// 文件名
    pub filename: String,
    /// 二进制数据
    pub data: Vec<u8>,
}
