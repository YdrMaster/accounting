use thiserror::Error;

/// 核心库错误类型
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AccountingError {
    /// 交易校验失败
    #[error("无效交易: {0}")]
    InvalidTransaction(String),
    /// 指定账户不存在
    #[error("账户不存在: {0}")]
    AccountNotFound(String),
    /// 账户余额非零导致无法关闭
    #[error("账户余额非零，无法关闭: {0}")]
    AccountNotEmpty(String),
    /// 指定商品不存在
    #[error("商品不存在: {0}")]
    CommodityNotFound(String),
    /// 账户已存在
    #[error("账户已存在: {0}")]
    AccountAlreadyExists(String),
    /// 日期解析失败
    #[error("日期格式错误: {0}")]
    InvalidDate(String),
    /// 底层数据库异常
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    /// 数据库文件不存在
    #[error("数据库文件不存在，请先运行 initialize 命令")]
    DbNotInitialized,
    /// 数据库文件已存在
    #[error("数据库文件已存在")]
    DbAlreadyExists,
    /// 其他未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AccountingError::InvalidTransaction("unbalanced".to_string());
        assert!(err.to_string().contains("unbalanced"));
    }
}
