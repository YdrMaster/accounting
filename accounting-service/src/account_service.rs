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

    /// 根据路径级联查找/创建账户（支持系统根账户）
    ///
    /// 与 `create_cascading` 类似，但允许路径中间段为任意名称。
    /// 只要首段是已存在的系统根账户（如 "Assets" / "Income" / "Expenses" / "Equity"）即可。
    /// 用于导入场景：在标准根账户下自动创建 `Import` 子树。
    pub async fn ensure_cascading(&self, path: &str) -> Result<AccountId, AccountingError> {
        let segments: Vec<&str> = path.split(':').collect();
        if segments.is_empty() {
            return Err(AccountingError::InvalidTransaction(
                t!("account_name_empty").to_string(),
            ));
        }

        // 校验首段：必须是已有的系统根账户
        let root = self
            .db
            .account_get_by_parent_and_name(None, segments[0])
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if let Some(root) = &root {
            if !root.is_system {
                return Err(AccountingError::InvalidTransaction(format!(
                    "根账户 '{}' 不是系统账户",
                    segments[0]
                )));
            }
        } else {
            return Err(AccountingError::AccountNotFound(format!(
                "根账户 '{}' 不存在",
                segments[0]
            )));
        }

        // 首段已存在，从第二段开始查找/创建
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut parent_id: Option<AccountId> = Some(root.as_ref().unwrap().id);
        let mut last_id: Option<AccountId> = Some(root.as_ref().unwrap().id);

        for segment in segments.iter().skip(1) {
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

            let account = Account {
                id: AccountId(0),
                name: segment.to_string(),
                parent_id,
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            };

            let id = tx
                .account_create_with_closure(&account)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            parent_id = Some(id);
            last_id = Some(id);
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

    #[tokio::test]
    async fn test_ensure_cascading_under_import_root() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let service = AccountService::new(db);

        // Expenses 系统根账户下创建 Import 子树
        let leaf_id = service
            .ensure_cascading("Expenses:Import:支付宝:餐饮美食")
            .await
            .unwrap();
        assert!(leaf_id.0 > 0);

        // 验证路径上的账户都存在
        let root = service
            .db
            .account_get_by_parent_and_name(None, "Expenses")
            .await
            .unwrap()
            .unwrap();
        assert!(root.is_system);

        let import_node = service
            .db
            .account_get_by_parent_and_name(Some(root.id), "Import")
            .await
            .unwrap()
            .unwrap();
        assert!(!import_node.is_system);

        let alipay = service
            .db
            .account_get_by_parent_and_name(Some(import_node.id), "支付宝")
            .await
            .unwrap()
            .unwrap();
        assert!(!alipay.is_system);

        let food = service
            .db
            .account_get_by_parent_and_name(Some(alipay.id), "餐饮美食")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(food.id, leaf_id);
        assert!(!food.is_system);
    }

    #[tokio::test]
    async fn test_ensure_cascading_idempotent() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let service = AccountService::new(db);

        let id1 = service
            .ensure_cascading("Expenses:Import:支付宝:餐饮美食")
            .await
            .unwrap();
        let id2 = service
            .ensure_cascading("Expenses:Import:支付宝:餐饮美食")
            .await
            .unwrap();
        assert_eq!(id1, id2, "重复调用应返回相同 AccountId");
    }

    #[tokio::test]
    async fn test_ensure_cascading_rejects_nonexistent_root() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let service = AccountService::new(db);

        let result = service.ensure_cascading("Nonexistent:子账户").await;
        assert!(result.is_err(), "不存在的根账户应报错");
    }

    #[tokio::test]
    async fn test_ensure_cascading_rejects_non_system_root() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let service = AccountService::new(db);

        // 先在 Assets 下创建普通子账户
        let _bank_id = service
            .create_cascading("Assets:Bank", None, None, &[])
            .await
            .unwrap();

        // Bank 不是系统根账户，ensure_cascading 应拒绝
        let result = service.ensure_cascading("Bank:SubAccount").await;
        assert!(result.is_err(), "非系统根账户应报错");
    }
}
