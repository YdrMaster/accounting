use thiserror::Error;

/// 数据库错误
#[derive(Error, Debug)]
pub enum DbError {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}
