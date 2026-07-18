//! 数据库层：sqlx SQLite 实现

/// 数据库入口
pub mod database;
/// 错误类型
pub mod error;
/// 名字按语言管理：六张名字表共享的显示名解析与写路径校验
pub mod names;
/// Repository 模块集合
pub mod repo;
/// 数据库 schema 与种子数据
pub mod schema;
/// 事务
pub mod transaction;

pub use database::SqliteDatabase;
pub use error::DbError;
pub use transaction::SqliteTransaction;
