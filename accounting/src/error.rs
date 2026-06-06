use rust_i18n::t;

/// 核心库错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum AccountingError {
    /// 交易校验失败
    InvalidTransaction(String),
    /// 指定账户不存在
    AccountNotFound(String),
    /// 账户余额非零导致无法关闭
    AccountNotEmpty(String),
    /// 指定商品不存在
    CommodityNotFound(String),
    /// 账户已存在
    AccountAlreadyExists(String),
    /// 日期解析失败
    InvalidDate(String),
    /// 底层数据库异常
    DatabaseError(String),
    /// 数据库文件不存在
    DbNotInitialized,
    /// 数据库文件已存在
    DbAlreadyExists,
    /// 其他未知错误
    Unknown(String),
}

impl std::fmt::Display for AccountingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransaction(msg) => {
                write!(f, "{}", t!("invalid_transaction", msg = msg))
            }
            Self::AccountNotFound(name) => {
                write!(f, "{}", t!("account_not_found", name = name))
            }
            Self::AccountNotEmpty(name) => {
                write!(f, "{}", t!("account_not_empty", name = name))
            }
            Self::CommodityNotFound(name) => {
                write!(f, "{}", t!("commodity_not_found", name = name))
            }
            Self::AccountAlreadyExists(name) => {
                write!(f, "{}", t!("account_already_exists", name = name))
            }
            Self::InvalidDate(msg) => {
                write!(f, "{}", t!("invalid_date", msg = msg))
            }
            Self::DatabaseError(msg) => {
                write!(f, "{}", t!("database_error", msg = msg))
            }
            Self::DbNotInitialized => {
                write!(f, "{}", t!("db_not_initialized"))
            }
            Self::DbAlreadyExists => {
                write!(f, "{}", t!("db_already_exists"))
            }
            Self::Unknown(msg) => {
                write!(f, "{}", t!("unknown_error", msg = msg))
            }
        }
    }
}

impl std::error::Error for AccountingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AccountingError::InvalidTransaction("unbalanced".to_string());
        assert!(err.to_string().contains("unbalanced"));
    }

    #[test]
    fn test_error_display_en() {
        rust_i18n::set_locale("en");
        let err = AccountingError::DbNotInitialized;
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_error_display_zh() {
        rust_i18n::set_locale("zh-CN");
        let err = AccountingError::DbNotInitialized;
        assert!(err.to_string().contains("不存在"));
    }
}
