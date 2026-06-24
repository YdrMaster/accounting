use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId, MemberId};
use accounting::validation::validate_account_close;
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;
use std::str::FromStr;

/// 账户服务
pub struct AccountService {
    db: SqliteDatabase,
}

impl AccountService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 创建账户并维护闭包表（保留原始接口）
    pub async fn create(&self, account: Account) -> Result<AccountId, AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证父账户存在；根账户则校验名称是否为有效根节点名
        if let Some(parent_id) = account.parent_id {
            let parent = tx
                .account_get(parent_id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if parent.is_none() {
                return Err(AccountingError::AccountNotFound(format!(
                    "{}",
                    t!("parent_account_not_found", id = parent_id)
                )));
            }
        } else {
            AccountType::from_str(&account.name).map_err(|_| {
                AccountingError::InvalidTransaction(
                    t!("unrecognized_account_prefix", prefix = account.name).to_string(),
                )
            })?;
        }

        // 同级 name 唯一性检查
        let existing = tx
            .account_get_by_parent_and_name(account.parent_id, &account.name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if existing.is_some() {
            return Err(AccountingError::AccountAlreadyExists(account.name.clone()));
        }

        // 创建账户并维护闭包表
        let id = tx
            .account_create_with_closure(&account)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(id)
    }

    /// 根据路径级联创建账户
    ///
    /// 解析 `:` 分隔的层级结构，校验第一段为有效根节点名，
    /// 逐级按 name + parent_id 查找/创建父账户，最后创建目标账户。
    /// 返回目标账户的 ID。
    pub async fn create_cascading(
        &self,
        path: &str,
        billing_day: Option<u8>,
        repayment_day: Option<u8>,
        owner_ids: &[MemberId],
    ) -> Result<AccountId, AccountingError> {
        let segments: Vec<&str> = path.split(':').collect();
        if segments.is_empty() {
            return Err(AccountingError::InvalidTransaction(
                t!("account_name_empty").to_string(),
            ));
        }

        // 校验第一段为有效根节点名
        AccountType::from_str(segments[0]).map_err(|_| {
            AccountingError::InvalidTransaction(
                t!("unrecognized_account_prefix", prefix = segments[0]).to_string(),
            )
        })?;

        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut parent_id: Option<AccountId> = None;
        let mut last_id: Option<AccountId> = None;

        for (i, segment) in segments.iter().enumerate() {
            // 检查是否已存在
            if let Some(existing) = tx
                .account_get_by_parent_and_name(parent_id, segment)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            {
                parent_id = Some(existing.id);
                last_id = Some(existing.id);
                continue;
            }

            let is_leaf = i == segments.len() - 1;
            let account = Account {
                id: AccountId(0),
                name: segment.to_string(),
                parent_id,
                closed_at: None,
                is_system: false,
                billing_day: if is_leaf { billing_day } else { None },
                repayment_day: if is_leaf { repayment_day } else { None },
            };

            let id = tx
                .account_create_with_closure(&account)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            parent_id = Some(id);
            last_id = Some(id);
        }

        // 设置账户所有者
        if !owner_ids.is_empty()
            && let Some(account_id) = last_id
        {
            tx.account_set_owners(account_id, owner_ids)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        last_id.ok_or_else(|| {
            AccountingError::InvalidTransaction(t!("cascade_create_failed").to_string())
        })
    }

    /// 关闭账户（含余额验证 + 级联关闭子账户）
    pub async fn close(&self, id: AccountId) -> Result<(), AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let account = tx
            .account_get(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let account = account.ok_or_else(|| {
            AccountingError::AccountNotFound(t!("account_not_found_id", id = id).to_string())
        })?;

        // 验证余额
        let balances = tx
            .posting_sum_by_account(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let root_name = tx
            .account_find_root_name(account.id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let account_type =
            AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)?;
        validate_account_close(account_type, &balances)?;

        // 关闭目标账户
        tx.account_close(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 级联关闭子账户
        let children = tx
            .account_list_children(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        for child in children {
            tx.account_close(child.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 重新开启账户（级联恢复子账户）
    pub async fn reopen(&self, id: AccountId) -> Result<(), AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 级联恢复子账户
        let children = tx
            .account_list_children(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        for child in children {
            tx.account_reopen(child.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.account_reopen(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 列出账户
    pub async fn list(
        &self,
        root_id: Option<AccountId>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Account>, AccountingError> {
        let mut accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if let Some(root_id) = root_id {
            let mut filtered = Vec::new();
            for a in accounts {
                let rid = self
                    .db
                    .account_find_root_id(a.id)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                if rid == root_id {
                    filtered.push(a);
                }
            }
            accounts = filtered;
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
        self.db
            .account_get(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 查询账户余额（含子账户聚合）
    pub async fn balance(
        &self,
        id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let totals = tx
            .posting_sum_with_ancestors(id)
            .await
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
    use accounting::id::AccountId;
    use accounting_sql::SqliteDatabase;

    async fn setup_db() -> SqliteDatabase {
        SqliteDatabase::open_in_memory().await.unwrap()
    }

    fn sample_account(name: &str) -> Account {
        Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    #[tokio::test]
    async fn test_create_account() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let account = sample_account("Asset");
        let id = service.create(account).await.unwrap();
        assert!(id.0 > 0);
    }

    #[tokio::test]
    async fn test_create_account_with_parent() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        let parent = sample_account("Equity");
        let parent_id = service.create(parent).await.unwrap();

        let mut child = sample_account("OpeningSvc");
        child.parent_id = Some(parent_id);
        let child_id = service.create(child).await.unwrap();

        // 验证闭包关系：子账户的根应为父账户
        let root_id = service.db.account_find_root_id(child_id).await.unwrap();
        assert_eq!(root_id, parent_id);

        let child = service.get(child_id).await.unwrap().unwrap();
        assert_eq!(child.parent_id, Some(parent_id));
    }

    #[tokio::test]
    async fn test_create_duplicate_name_fails() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        let parent = sample_account("Income");
        let parent_id = service.create(parent).await.unwrap();

        let mut child = sample_account("SalarySvc");
        child.parent_id = Some(parent_id);
        service.create(child.clone()).await.unwrap();
        let result = service.create(child).await;
        assert!(matches!(
            result,
            Err(AccountingError::AccountAlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn test_create_with_nonexistent_parent_fails() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let mut account = sample_account("Expense");
        account.parent_id = Some(AccountId(99999));
        let result = service.create(account).await;
        assert!(matches!(result, Err(AccountingError::AccountNotFound(_))));
    }

    #[tokio::test]
    async fn test_close_and_reopen_account() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        let parent = sample_account("Expenses");
        let parent_id = service.create(parent).await.unwrap();

        let mut child = sample_account("FoodSvc");
        child.parent_id = Some(parent_id);
        let id = service.create(child).await.unwrap();

        service.close(id).await.unwrap();

        let closed = service.get(id).await.unwrap().unwrap();
        assert!(closed.closed_at.is_some());

        service.reopen(id).await.unwrap();

        let reopened = service.get(id).await.unwrap().unwrap();
        assert!(reopened.closed_at.is_none());
    }
}
