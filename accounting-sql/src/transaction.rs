use sqlx::{Sqlite, Transaction};

use crate::error::DbError;

/// SQLite 事务
pub struct SqliteTransaction<'a> {
    pub(crate) tx: Transaction<'a, Sqlite>,
}

impl<'a> SqliteTransaction<'a> {
    pub(crate) fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    /// 提交事务
    pub async fn commit(self) -> Result<(), DbError> {
        self.tx
            .commit()
            .await
            .map_err(|e| DbError::Database(e.to_string()))
    }

    // === Account ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Commodity ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Member ===
    // 委托方法已由 repo/facade.rs 的清单宏生成
    // 委托方法已由 repo/facade.rs 的清单宏生成

    /// 账户改名：先校验改名合法性再执行（复合流程，不入清单宏）
    pub async fn account_rename(
        &mut self,
        id: accounting::id::AccountId,
        new_name: &str,
        lang: &str,
    ) -> Result<(), DbError> {
        crate::repo::account::account_ensure_rename_allowed(&mut self.tx, id).await?;
        crate::repo::account::account_rename(&mut self.tx, id, new_name, lang).await
    }

    // === Channel ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === ChannelPath ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Tag ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Attachment ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Transaction ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Posting ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Settings ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Account Mapping ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Budget ===
    // 委托方法已由 repo/facade.rs 的清单宏生成
}
