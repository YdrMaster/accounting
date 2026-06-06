//! 数据库层：Repository traits + SQLite 实现

/// 数据库 trait 定义
pub mod database;
/// 错误类型
pub mod error;
/// 具体实现
pub mod impls;
/// 连接池
pub mod pool;
/// Repository 模块集合
pub mod repo;
/// 数据库 schema 与种子数据
pub mod schema;
/// 事务 trait
pub mod transaction;
