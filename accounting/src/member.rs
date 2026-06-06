use crate::id::MemberId;

/// 成员
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: MemberId,
    pub name: String,
    pub description: Option<String>,
}
