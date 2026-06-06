use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId};
use accounting::validation::validate_account_close;
use accounting_sql::database::Database;
use accounting_sql::transaction::Transaction;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 账户服务
pub struct AccountService<D: Database> {
    db: D,
}

impl<D: Database> AccountService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 创建账户并维护闭包表
    pub async fn create(&self, account: Account) -> Result<AccountId, AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证父账户存在
        if let Some(parent_id) = account.parent_id {
            let parent = tx
                .account_repo()
                .get(&tx.conn(), parent_id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if parent.is_none() {
                return Err(AccountingError::AccountNotFound(format!(
                    "父账户 {} 不存在",
                    parent_id
                )));
            }
        }

        // full_name 唯一性检查
        let existing = tx
            .account_repo()
            .get_by_name(&tx.conn(), &account.full_name)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if existing.is_some() {
            return Err(AccountingError::AccountAlreadyExists(
                account.full_name.clone(),
            ));
        }

        // 创建账户并维护闭包表
        let id = tx
            .account_repo()
            .create_with_closure(&tx.conn(), &account)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(id)
    }

    /// 关闭账户（含余额验证 + 级联关闭子账户）
    pub async fn close(&self, id: AccountId) -> Result<(), AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let account = tx
            .account_repo()
            .get(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let account = account
            .ok_or_else(|| AccountingError::AccountNotFound(format!("账户 {} 不存在", id)))?;

        // 验证余额
        let balances = tx
            .posting_repo()
            .sum_by_account(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        validate_account_close(account.account_type, &balances)?;

        // 关闭目标账户
        tx.account_repo()
            .close(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 级联关闭子账户
        let children = tx
            .account_repo()
            .list_children(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        for child in children {
            tx.account_repo()
                .close(&tx.conn(), child.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 重新开启账户（级联恢复子账户）
    pub async fn reopen(&self, id: AccountId) -> Result<(), AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 级联恢复子账户
        let children = tx
            .account_repo()
            .list_children(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        for child in children {
            tx.account_repo()
                .reopen(&tx.conn(), child.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.account_repo()
            .reopen(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 列出账户
    pub async fn list(
        &self,
        account_type: Option<AccountType>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Account>, AccountingError> {
        let conn = self.db.connection();
        let mut accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if let Some(ty) = account_type {
            accounts.retain(|a| a.account_type == ty);
        }
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize).unwrap_or(accounts.len());
        if offset >= accounts.len() {
            accounts.clear();
        } else {
            let end = (offset + limit).min(accounts.len());
            accounts = accounts[offset..end].to_vec();
        }
        Ok(accounts)
    }

    /// 根据 ID 查询账户
    pub async fn get(&self, id: AccountId) -> Result<Option<Account>, AccountingError> {
        let conn = self.db.connection();
        self.db
            .account_repo()
            .get(&conn, id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 查询账户余额（含子账户聚合）
    pub async fn balance(
        &self,
        id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let totals = tx
            .posting_repo()
            .sum_with_ancestors(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(totals.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::AccountId;
    use accounting_sql::impls::sqlite::SqliteDatabase;

    fn sample_account(name: &str, account_type: AccountType) -> Account {
        Account {
            id: AccountId(0),
            full_name: name.to_string(),
            account_type,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    #[tokio::test]
    async fn test_create_account() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = AccountService::new(db);
        let account = sample_account("Assets:Cash", AccountType::Asset);
        let id = service.create(account).await.unwrap();
        assert!(id.0 > 0);
    }

    #[tokio::test]
    async fn test_create_account_with_parent() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = AccountService::new(db);

        let parent = sample_account("Assets", AccountType::Asset);
        let parent_id = service.create(parent).await.unwrap();

        let mut child = sample_account("Assets:Cash", AccountType::Asset);
        child.parent_id = Some(parent_id);
        let child_id = service.create(child).await.unwrap();

        // 验证闭包表
        let conn = service.db.connection();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM account_ancestors WHERE account_id = ?1",
                rusqlite::params![child_id.0],
                |row| row.get(0),
            )
            .unwrap();
        // child_id -> child_id (depth 0) + child_id -> parent_id (depth 1) = 2
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_create_duplicate_name_fails() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = AccountService::new(db);
        let account = sample_account("Assets:Cash", AccountType::Asset);
        service.create(account.clone()).await.unwrap();
        let result = service.create(account).await;
        assert!(matches!(
            result,
            Err(AccountingError::AccountAlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn test_create_with_nonexistent_parent_fails() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = AccountService::new(db);
        let mut account = sample_account("Assets:Cash", AccountType::Asset);
        account.parent_id = Some(AccountId(99999));
        let result = service.create(account).await;
        assert!(matches!(result, Err(AccountingError::AccountNotFound(_))));
    }

    #[tokio::test]
    async fn test_close_and_reopen_account() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = AccountService::new(db);
        let account = sample_account("Assets:Cash", AccountType::Asset);
        let id = service.create(account).await.unwrap();

        service.close(id).await.unwrap();

        {
            let conn = service.db.connection();
            let closed: Option<String> = conn
                .query_row(
                    "SELECT closed_at FROM accounts WHERE id = ?1",
                    rusqlite::params![id.0],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(closed.is_some());
        }

        service.reopen(id).await.unwrap();

        {
            let conn = service.db.connection();
            let reopened: Option<String> = conn
                .query_row(
                    "SELECT closed_at FROM accounts WHERE id = ?1",
                    rusqlite::params![id.0],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(reopened.is_none());
        }
    }
}
