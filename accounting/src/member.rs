use crate::id::MemberId;

/// 成员
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// 成员唯一标识符
    pub id: MemberId,
    /// 成员名称
    pub name: String,
    /// 成员描述
    pub description: Option<String>,
}
