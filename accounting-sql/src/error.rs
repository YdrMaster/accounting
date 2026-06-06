use thiserror::Error;

/// 数据库错误
#[derive(Error, Debug)]
pub enum DbError {
    /// SQLite 底层错误
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// 其他业务错误
    #[error("{0}")]
    Other(String),
}
