use accounting::account_mapping::AccountMapping;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId};
use accounting_sql::SqliteDatabase;
use rust_i18n::t;

/// 账户映射服务
pub struct MappingService {
    db: SqliteDatabase,
}

impl MappingService {
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 设置映射（upsert 语义）
    ///
    /// 接受账户路径字符串，查找对应 AccountId，不存在则报错。
    pub async fn set(
        &self,
        member_id: MemberId,
        channel_id: ChannelId,
        category: &str,
        account_path: &str,
    ) -> Result<(), AccountingError> {
        let account_id = self
            .find_account_by_path(account_path)
            .await?
            .ok_or_else(|| {
                AccountingError::AccountNotFound(format!(
                    "{}",
                    t!("account_not_found", name = account_path)
                ))
            })?;

        let mapping = AccountMapping {
            member_id,
            channel_id,
            category: category.to_string(),
            account_id,
        };

        self.db
            .account_mapping_upsert(&mapping)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 列出某个 (成员, 渠道) 的所有映射
    pub async fn list(
        &self,
        member_id: MemberId,
        channel_id: ChannelId,
    ) -> Result<Vec<AccountMapping>, AccountingError> {
        self.db
            .account_mapping_list(member_id, channel_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除单条映射
    pub async fn delete(
        &self,
        member_id: MemberId,
        channel_id: ChannelId,
        category: &str,
    ) -> Result<(), AccountingError> {
        let deleted = self
            .db
            .account_mapping_delete(member_id, channel_id, category)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        if deleted {
            Ok(())
        } else {
            Err(AccountingError::AccountNotFound(format!(
                "映射 ({}, {}, {}) 不存在",
                member_id.0, channel_id.0, category
            )))
        }
    }

    /// 根据账户路径查找 AccountId
    ///
    /// 路径格式："A:B:C"，沿账户树逐级查找。
    async fn find_account_by_path(&self, path: &str) -> Result<Option<AccountId>, AccountingError> {
        let segments: Vec<&str> = path.split(':').collect();
        if segments.is_empty() {
            return Ok(None);
        }

        // 查找根账户
        let root = self
            .db
            .account_get_by_parent_and_name(None, segments[0])
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut current = match root {
            Some(r) => r,
            None => return Ok(None),
        };

        // 逐级查找子账户
        for segment in segments.iter().skip(1) {
            match self
                .db
                .account_get_by_parent_and_name(Some(current.id), segment)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            {
                Some(child) => current = child,
                None => return Ok(None),
            };
        }

        Ok(Some(current.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::channel::Channel;
    use accounting::id::AccountId;
    use accounting::member::Member;
    use accounting_sql::SqliteDatabase;

    async fn setup_db() -> (SqliteDatabase, MemberId, ChannelId) {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize("en").await.unwrap();

        // 创建成员
        let member = Member {
            id: MemberId(0),
            name: "测试用户".to_string(),
        };
        let member_id = db.member_create(&member).await.unwrap();

        // 创建渠道
        let channel = Channel {
            id: ChannelId(0),
            name: "TestPay".to_string(),
            description: None,
            account_id: None,
            is_system: false,
        };
        let channel_id = db.channel_create(&channel).await.unwrap();

        (db, member_id, channel_id)
    }

    #[tokio::test]
    async fn test_mapping_set_and_list() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        // Expenses 账户在种子数据中存在
        service
            .set(member_id, channel_id, "收支:餐饮美食", "Expenses")
            .await
            .unwrap();

        let list = service.list(member_id, channel_id).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].category, "收支:餐饮美食");
    }

    #[tokio::test]
    async fn test_mapping_set_account_not_found() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        let result = service
            .set(member_id, channel_id, "收支:餐饮美食", "NonExistent:Path")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mapping_set_with_path() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        // Expenses:Fees 在种子数据中存在
        service
            .set(member_id, channel_id, "收支:手续费", "Expenses:Fees")
            .await
            .unwrap();

        let list = service.list(member_id, channel_id).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_mapping_delete() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        service
            .set(member_id, channel_id, "收支:餐饮美食", "Expenses")
            .await
            .unwrap();

        service
            .delete(member_id, channel_id, "收支:餐饮美食")
            .await
            .unwrap();

        let list = service.list(member_id, channel_id).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_mapping_delete_nonexistent() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        let result = service.delete(member_id, channel_id, "收支:不存在").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mapping_set_overwrite() {
        let (db, member_id, channel_id) = setup_db().await;
        let service = MappingService::new(db.clone());

        service
            .set(member_id, channel_id, "收支:餐饮美食", "Expenses")
            .await
            .unwrap();

        service
            .set(member_id, channel_id, "收支:餐饮美食", "Expenses:Fees")
            .await
            .unwrap();

        let list = service.list(member_id, channel_id).await.unwrap();
        assert_eq!(list.len(), 1);
        // 验证指向 Fees（子账户）而非 Expenses
        let fees_id = db
            .account_get_by_parent_and_name(
                db.account_get_by_name("Expenses")
                    .await
                    .unwrap()
                    .unwrap()
                    .id
                    .into(),
                "Fees",
            )
            .await
            .unwrap()
            .unwrap()
            .id;
        assert_eq!(list[0].account_id, fees_id);
    }
}
