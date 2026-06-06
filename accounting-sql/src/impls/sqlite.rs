use crate::database::Database;
use crate::error::DbError;
use crate::pool::ConnectionHandle;
use crate::repo::account::{AccountRepo, SqliteAccountRepo};
use crate::repo::attachment::{AttachmentRepo, SqliteAttachmentRepo};
use crate::repo::channel::{ChannelRepo, SqliteChannelRepo};
use crate::repo::commodity::{CommodityRepo, SqliteCommodityRepo};
use crate::repo::member::{MemberRepo, SqliteMemberRepo};
use crate::repo::posting::{PostingRepo, SqlitePostingRepo};
use crate::repo::tag::{SqliteTagRepo, TagRepo};
use crate::repo::transaction::{SqliteTransactionRepo, TransactionRepo};
use crate::transaction::Transaction;

/// SQLite 数据库实现
pub struct SqliteDatabase {
    pool: ConnectionHandle,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    member_repo: SqliteMemberRepo,
    channel_repo: SqliteChannelRepo,
    tag_repo: SqliteTagRepo,
    attachment_repo: SqliteAttachmentRepo,
    transaction_repo: SqliteTransactionRepo,
    posting_repo: SqlitePostingRepo,
}

impl SqliteDatabase {
    /// 打开文件数据库并自动初始化 schema
    pub fn open(path: &str) -> Result<Self, DbError> {
        let pool = ConnectionHandle::open(path)?;
        {
            let conn = pool.get();
            crate::schema::initialize_schema(&conn)?;
        }
        Ok(Self::new(pool))
    }

    /// 打开内存数据库并自动初始化 schema
    pub fn open_in_memory() -> Result<Self, DbError> {
        let pool = ConnectionHandle::open_in_memory()?;
        {
            let conn = pool.get();
            crate::schema::initialize_schema(&conn)?;
        }
        Ok(Self::new(pool))
    }

    /// 初始化 seed 数据，支持语言选择
    pub fn initialize(&self, lang: &str) -> Result<(), DbError> {
        let conn = self.pool.get();
        crate::schema::insert_seed_data(&conn, lang)?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('language', ?1)",
            [lang],
        )?;
        Ok(())
    }

    fn new(pool: ConnectionHandle) -> Self {
        Self {
            pool,
            account_repo: SqliteAccountRepo,
            commodity_repo: SqliteCommodityRepo,
            member_repo: SqliteMemberRepo,
            channel_repo: SqliteChannelRepo,
            tag_repo: SqliteTagRepo,
            attachment_repo: SqliteAttachmentRepo,
            transaction_repo: SqliteTransactionRepo,
            posting_repo: SqlitePostingRepo,
        }
    }
}

impl Database for SqliteDatabase {
    type Tx = SqliteTransaction;

    fn account_repo(&self) -> &dyn AccountRepo {
        &self.account_repo
    }
    fn commodity_repo(&self) -> &dyn CommodityRepo {
        &self.commodity_repo
    }
    fn member_repo(&self) -> &dyn MemberRepo {
        &self.member_repo
    }
    fn channel_repo(&self) -> &dyn ChannelRepo {
        &self.channel_repo
    }
    fn tag_repo(&self) -> &dyn TagRepo {
        &self.tag_repo
    }
    fn attachment_repo(&self) -> &dyn AttachmentRepo {
        &self.attachment_repo
    }
    fn transaction_repo(&self) -> &dyn TransactionRepo {
        &self.transaction_repo
    }
    fn posting_repo(&self) -> &dyn PostingRepo {
        &self.posting_repo
    }

    fn connection(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.pool.get()
    }

    async fn transaction(&self) -> Result<Self::Tx, DbError> {
        let conn = self.pool.get();
        conn.execute("BEGIN", [])?;
        Ok(SqliteTransaction {
            pool: self.pool.clone(),
            committed: false,
            account_repo: SqliteAccountRepo,
            commodity_repo: SqliteCommodityRepo,
            member_repo: SqliteMemberRepo,
            channel_repo: SqliteChannelRepo,
            tag_repo: SqliteTagRepo,
            attachment_repo: SqliteAttachmentRepo,
            transaction_repo: SqliteTransactionRepo,
            posting_repo: SqlitePostingRepo,
        })
    }
}

/// SQLite 事务实现
pub struct SqliteTransaction {
    pool: ConnectionHandle,
    committed: bool,
    account_repo: SqliteAccountRepo,
    commodity_repo: SqliteCommodityRepo,
    member_repo: SqliteMemberRepo,
    channel_repo: SqliteChannelRepo,
    tag_repo: SqliteTagRepo,
    attachment_repo: SqliteAttachmentRepo,
    transaction_repo: SqliteTransactionRepo,
    posting_repo: SqlitePostingRepo,
}

impl SqliteTransaction {
    /// 获取底层连接锁（供 Repository 方法使用）
    pub fn conn(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.pool.get()
    }
}

impl Transaction for SqliteTransaction {
    fn account_repo(&self) -> &dyn AccountRepo {
        &self.account_repo
    }
    fn commodity_repo(&self) -> &dyn CommodityRepo {
        &self.commodity_repo
    }
    fn member_repo(&self) -> &dyn MemberRepo {
        &self.member_repo
    }
    fn channel_repo(&self) -> &dyn ChannelRepo {
        &self.channel_repo
    }
    fn tag_repo(&self) -> &dyn TagRepo {
        &self.tag_repo
    }
    fn attachment_repo(&self) -> &dyn AttachmentRepo {
        &self.attachment_repo
    }
    fn transaction_repo(&self) -> &dyn TransactionRepo {
        &self.transaction_repo
    }
    fn posting_repo(&self) -> &dyn PostingRepo {
        &self.posting_repo
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
        self.pool.get()
    }

    async fn commit(mut self) -> Result<(), DbError> {
        let conn = self.pool.get();
        conn.execute("COMMIT", [])?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for SqliteTransaction {
    fn drop(&mut self) {
        if !self.committed {
            let conn = self.pool.get();
            let _ = conn.execute("ROLLBACK", []);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::AccountId;

    #[tokio::test]
    async fn test_open_in_memory_initializes_schema() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let conn = db.pool.get();
        let repo = SqliteAccountRepo;
        let found = repo.get_by_name(&conn, "Equity:OpeningBalances").unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let tx = db.transaction().await.unwrap();
        let account = Account {
            id: AccountId(0),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = tx.account_repo().create(&tx.conn(), &account).unwrap();
        tx.commit().await.unwrap();

        let conn = db.pool.get();
        let repo = SqliteAccountRepo;
        assert!(repo.get(&conn, id).unwrap().is_some());
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_drop() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let id = {
            let tx = db.transaction().await.unwrap();
            let account = Account {
                id: AccountId(0),
                full_name: "Assets:Cash".to_string(),
                account_type: AccountType::Asset,
                parent_id: None,
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            };
            let id = tx.account_repo().create(&tx.conn(), &account).unwrap();
            id // tx dropped here without commit
        };

        let conn = db.pool.get();
        let repo = SqliteAccountRepo;
        assert!(repo.get(&conn, id).unwrap().is_none());
    }
}
