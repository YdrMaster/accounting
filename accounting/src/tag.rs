use crate::id::TagId;

/// 标签
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}
