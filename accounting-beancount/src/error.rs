use thiserror::Error;

#[derive(Debug, Error)]
pub enum BeancountError {
    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("database error: {0}")]
    DatabaseError(String),

    #[error("missing attachment file: {0}")]
    MissingAttachment(String),
}
