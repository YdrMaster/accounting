use crate::id::TagId;

/// 标签
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// 标签唯一标识符
    pub id: TagId,
    /// 标签描述
    pub description: Option<String>,
    /// 是否为系统内置标签
    pub is_system: bool,
}
