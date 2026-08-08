//! 错误定义：crate 内部统一用 AuthError，HTTP 映射在 api 层完成

/// 认证子系统统一错误类型。
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    /// 数据库访问错误。
    #[error("数据库错误: {0}")]
    Db(#[from] sqlx::Error),

    /// schema 迁移失败。
    #[error("数据库迁移失败: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// 密码哈希计算失败（argon2 内部错误）。
    #[error("密码哈希错误: {0}")]
    PasswordHash(String),

    /// 用户名已存在（唯一约束冲突）。
    #[error("用户名已存在: {0}")]
    DuplicateUsername(String),

    /// 用户不存在。
    #[error("用户不存在: {0}")]
    UserNotFound(String),

    /// 配置/参数错误。
    #[error("配置错误: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;
