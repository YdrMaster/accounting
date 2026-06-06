use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AccountingError {
    #[error("无效交易: {0}")]
    InvalidTransaction(String),
    #[error("账户不存在: {0}")]
    AccountNotFound(String),
    #[error("账户余额非零，无法关闭: {0}")]
    AccountNotEmpty(String),
    #[error("商品不存在: {0}")]
    CommodityNotFound(String),
    #[error("账户已存在: {0}")]
    AccountAlreadyExists(String),
    #[error("日期格式错误: {0}")]
    InvalidDate(String),
    #[error("数据库错误: {0}")]
    DatabaseError(String),
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
