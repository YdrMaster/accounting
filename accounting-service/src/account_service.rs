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

    /// 创建账户并维护闭包表
    ///
    /// `name` 为账户名字（按 `lang` 语言写入名字表）；实体本身不再携带名字。
    pub async fn create(
        &self,
        account: Account,
        name: &str,
        lang: &str,
    ) -> Result<AccountId, AccountingError> {
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
            AccountType::from_str(name).map_err(|_| {
                AccountingError::InvalidTransaction(
                    t!("unrecognized_account_prefix", prefix = name).to_string(),
                )
            })?;
        }

        // 同级 name 唯一性检查
        let existing = tx
            .account_get_by_parent_and_name(account.parent_id, name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if existing.is_some() {
            return Err(AccountingError::AccountAlreadyExists(name.to_string()));
        }

        // 创建账户并维护闭包表，名字按 lang 写入名字表
        let id = tx
            .account_create_with_name(&account, name, lang)
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
    /// 新创建账户的名字按 `lang` 语言写入名字表。
    /// 返回目标账户的 ID。
    pub async fn create_cascading(
        &self,
        path: &str,
        lang: &str,
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
                parent_id,
                closed_at: None,
                is_system: false,
                billing_day: if is_leaf { billing_day } else { None },
                repayment_day: if is_leaf { repayment_day } else { None },
            };

            let id = tx
                .account_create_with_name(&account, segment, lang)
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
    /// 自动创建的账户名来自外部账单数据，语言标注为 `und`（未确定）。
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
                parent_id,
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            };

            // 导入自动创建的账户名来自外部账单，语言未确定（und）
            let id = tx
                .account_create_with_name(&account, segment, accounting::name::lang::UND)
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
        // 根账户名用于内部账户类型判定，固定按 en 解析（系统根账户必有 en 系统名）
        let root_name = tx
            .account_find_root_name(account.id, accounting::name::lang::EN)
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

    /// 变更账户父节点（整棵子树跟随移动）
    ///
    /// 校验（按序）：被移动账户存在、非系统账户、非根账户；目标父账户存在；
    /// 目标非自身且不在被移动账户的后代中（防成环）；目标父账户下无同名账户。
    /// 不校验账户类型（允许跨类型移动）；已关闭账户允许移动，关闭状态不变。
    /// `lang` 用于解析被移动账户的显示名以做同级重名检查。
    pub async fn reparent(
        &self,
        id: AccountId,
        new_parent_id: AccountId,
        lang: &str,
    ) -> Result<(), AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let account = tx
            .account_get(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                AccountingError::AccountNotFound(t!("account_not_found_id", id = id).to_string())
            })?;
        if account.is_system {
            return Err(AccountingError::InvalidTransaction(
                t!("cannot_move_system_account").to_string(),
            ));
        }
        if account.parent_id.is_none() {
            return Err(AccountingError::InvalidTransaction(
                t!("cannot_move_root_account").to_string(),
            ));
        }

        let target = tx
            .account_get(new_parent_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if target.is_none() {
            return Err(AccountingError::AccountNotFound(format!(
                "{}",
                t!("parent_account_not_found", id = new_parent_id)
            )));
        }

        // 防成环：目标是被移动账户自身或其后代
        if new_parent_id == id
            || tx
                .account_is_descendant_of(new_parent_id, id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
        {
            return Err(AccountingError::InvalidTransaction(
                t!("cannot_move_into_own_subtree").to_string(),
            ));
        }

        // 目标父账户下同级重名检查（名字按请求语言解析，与既有重名校验行为一致）
        if let Some(name) = tx
            .account_display_name(id, lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            && let Some(dup) = tx
                .account_get_by_parent_and_name(Some(new_parent_id), &name)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            && dup.id != id
        {
            return Err(AccountingError::AccountAlreadyExists(name));
        }

        tx.account_update_parent(id, new_parent_id)
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
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    fn bare_account(parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            parent_id,
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
        let id = service
            .create(bare_account(None), "Asset", "en")
            .await
            .unwrap();
        assert!(id.0 > 0);
    }

    #[tokio::test]
    async fn test_create_account_with_parent() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        // 种子数据中的系统根账户 Equity 作为父账户
        let parent_id = service
            .db
            .account_get_by_name("Equity")
            .await
            .unwrap()
            .unwrap()
            .id;

        let child_id = service
            .create(bare_account(Some(parent_id)), "OpeningSvc", "en")
            .await
            .unwrap();

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

        let parent_id = service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .unwrap()
            .id;

        let child = bare_account(Some(parent_id));
        service
            .create(child.clone(), "SalarySvc", "en")
            .await
            .unwrap();
        let result = service.create(child, "SalarySvc", "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::AccountAlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn test_create_with_nonexistent_parent_fails() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let account = bare_account(Some(AccountId(99999)));
        let result = service.create(account, "Whatever", "en").await;
        assert!(matches!(result, Err(AccountingError::AccountNotFound(_))));
    }

    #[tokio::test]
    async fn test_close_and_reopen_account() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        let parent_id = service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;

        let id = service
            .create(bare_account(Some(parent_id)), "FoodSvc", "en")
            .await
            .unwrap();

        service.close(id).await.unwrap();

        let closed = service.get(id).await.unwrap().unwrap();
        assert!(closed.closed_at.is_some());

        service.reopen(id).await.unwrap();

        let reopened = service.get(id).await.unwrap().unwrap();
        assert!(reopened.closed_at.is_none());
    }

    #[tokio::test]
    async fn test_ensure_cascading_under_import_root() {
        let db = setup_db().await;
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
        let db = setup_db().await;
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
        let db = setup_db().await;
        let service = AccountService::new(db);

        let result = service.ensure_cascading("Nonexistent:子账户").await;
        assert!(result.is_err(), "不存在的根账户应报错");
    }

    #[tokio::test]
    async fn test_ensure_cascading_rejects_non_system_root() {
        let db = setup_db().await;
        let service = AccountService::new(db);

        // 先在 Assets 下创建普通子账户
        let _bank_id = service
            .create_cascading("Assets:Bank", "en", None, None, &[])
            .await
            .unwrap();

        // Bank 不是系统根账户，ensure_cascading 应拒绝
        let result = service.ensure_cascading("Bank:SubAccount").await;
        assert!(result.is_err(), "非系统根账户应报错");
    }

    /// 在 Assets 下建 Bank→Sub，在 Expenses 下建 Food，返回 (assets, bank, sub, food)。
    async fn build_move_fixture(
        service: &AccountService,
    ) -> (AccountId, AccountId, AccountId, AccountId) {
        let assets = service
            .db
            .account_get_by_name("Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        let expenses = service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let bank = service
            .create(bare_account(Some(assets)), "Bank", "en")
            .await
            .unwrap();
        let sub = service
            .create(bare_account(Some(bank)), "Sub", "en")
            .await
            .unwrap();
        let food = service
            .create(bare_account(Some(expenses)), "Food", "en")
            .await
            .unwrap();
        (assets, bank, sub, food)
    }

    #[tokio::test]
    async fn test_reparent_success_moves_subtree() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (assets, bank, sub, _food) = build_move_fixture(&service).await;
        let broker = service
            .create(bare_account(Some(assets)), "Broker", "en")
            .await
            .unwrap();

        service.reparent(bank, broker, "en").await.unwrap();

        let moved = service.get(bank).await.unwrap().unwrap();
        assert_eq!(moved.parent_id, Some(broker));
        // 子账户跟随移动：Sub 以 Broker 为祖先，根仍是 Assets
        assert!(
            service
                .db
                .account_is_descendant_of(sub, broker)
                .await
                .unwrap()
        );
        assert_eq!(service.db.account_find_root_id(sub).await.unwrap(), assets);
        // 移动后可按新路径按名查找
        assert!(
            service
                .db
                .account_get_by_name("Assets:Broker:Bank:Sub")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_reparent_cross_type_allowed() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (_assets, bank, _sub, food) = build_move_fixture(&service).await;

        // 不校验账户类型：Assets 子树可移动到 Expenses 下
        service.reparent(bank, food, "en").await.unwrap();

        let expenses = service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        assert_eq!(
            service.db.account_find_root_id(bank).await.unwrap(),
            expenses
        );
    }

    #[tokio::test]
    async fn test_reparent_closed_account_allowed() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (assets, bank, _sub, _food) = build_move_fixture(&service).await;
        let broker = service
            .create(bare_account(Some(assets)), "Broker", "en")
            .await
            .unwrap();

        service.close(bank).await.unwrap();
        service.reparent(bank, broker, "en").await.unwrap();

        // 关闭状态不变
        let moved = service.get(bank).await.unwrap().unwrap();
        assert!(moved.closed_at.is_some());
        assert_eq!(moved.parent_id, Some(broker));
    }

    #[tokio::test]
    async fn test_reparent_rejects_root_account() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (_assets, bank, _sub, food) = build_move_fixture(&service).await;

        // 非系统根账户（repo 层直建，绕开 create 的根名校验）
        let custom_root = service
            .db
            .account_create_with_name(&bare_account(None), "CustomRoot", "en")
            .await
            .unwrap();
        let result = service.reparent(custom_root, food, "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));

        //  sanity：普通账户移动正常
        assert!(service.reparent(bank, food, "en").await.is_ok());
    }

    #[tokio::test]
    async fn test_reparent_rejects_system_account() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (_assets, _bank, _sub, food) = build_move_fixture(&service).await;

        // 种子系统子账户 Assets:Cash（is_system 且非根）
        let cash = service
            .db
            .account_get_by_name("Assets:Cash")
            .await
            .unwrap()
            .unwrap();
        assert!(cash.is_system);
        let result = service.reparent(cash.id, food, "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));
    }

    #[tokio::test]
    async fn test_reparent_rejects_missing_account_or_target() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (_assets, bank, _sub, food) = build_move_fixture(&service).await;

        let result = service.reparent(AccountId(99999), food, "en").await;
        assert!(matches!(result, Err(AccountingError::AccountNotFound(_))));

        let result = service.reparent(bank, AccountId(99999), "en").await;
        assert!(matches!(result, Err(AccountingError::AccountNotFound(_))));
    }

    #[tokio::test]
    async fn test_reparent_rejects_self_and_descendant() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (_assets, bank, sub, _food) = build_move_fixture(&service).await;

        // 移动到自身
        let result = service.reparent(bank, bank, "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));

        // 移动到后代 → 成环
        let result = service.reparent(bank, sub, "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));

        // 校验失败后原结构不变
        let unchanged = service.get(bank).await.unwrap().unwrap();
        assert_eq!(
            unchanged.parent_id,
            Some(
                service
                    .db
                    .account_get_by_name("Assets")
                    .await
                    .unwrap()
                    .unwrap()
                    .id
            )
        );
    }

    #[tokio::test]
    async fn test_reparent_rejects_duplicate_name_under_target() {
        let db = setup_db().await;
        let service = AccountService::new(db);
        let (assets, bank, _sub, _food) = build_move_fixture(&service).await;

        // 目标父 Assets 下已有名为 Sub 的账户时，把另一个 Sub 移入 → 拒绝
        let expenses = service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;
        let other_sub = service
            .create(bare_account(Some(expenses)), "Sub", "en")
            .await
            .unwrap();
        let result = service.reparent(other_sub, bank, "en").await;
        assert!(matches!(
            result,
            Err(AccountingError::AccountAlreadyExists(_))
        ));

        // 同名但目标父下无冲突 → 允许（bank 在 Assets 下，Assets 下无 Sub 之外的 Bank）
        let food_bank = service
            .create(bare_account(Some(expenses)), "Bank2", "en")
            .await
            .unwrap();
        assert!(service.reparent(food_bank, assets, "en").await.is_ok());
        let _ = bank;
    }
}
