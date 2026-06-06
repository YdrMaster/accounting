use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::error::DbError;

/// SQLite 数据库连接句柄（包装单个 Connection 的同步访问）
#[derive(Clone)]
pub struct ConnectionHandle {
    conn: Arc<Mutex<Connection>>,
}

impl ConnectionHandle {
    /// 打开文件数据库
    pub fn open(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 打开内存数据库
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取连接锁
    pub fn get(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}
