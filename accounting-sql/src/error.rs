use thiserror::Error;

/// 数据库错误
#[derive(Error, Debug)]
pub enum DbError {
    /// 底层数据库错误
    #[error("database error: {0}")]
    Database(String),
}
