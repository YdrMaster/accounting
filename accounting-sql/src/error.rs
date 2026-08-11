use accounting::id::AccountId;
use thiserror::Error;

/// 数据库错误
#[derive(Error, Debug)]
pub enum DbError {
    /// 底层数据库错误
    #[error("database error: {0}")]
    Database(String),
    /// 系统根账户改名被拒绝（调用方按变体判定状态码，不依赖本地化字面量）
    #[error("cannot rename system root account {0}")]
    SystemRootRenameProtected(AccountId),
}
