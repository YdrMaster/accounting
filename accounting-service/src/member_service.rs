use accounting::error::AccountingError;
use accounting::id::MemberId;
use accounting::member::Member;
use accounting_sql::SqliteDatabase;

/// 成员服务
pub struct MemberService {
    db: SqliteDatabase,
}

impl MemberService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 列出所有成员
    pub async fn list(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Member>, AccountingError> {
        let mut members = self
            .db
            .member_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize).unwrap_or(members.len());
        if offset >= members.len() {
            members.clear();
        } else {
            let end = (offset + limit).min(members.len());
            members = members[offset..end].to_vec();
        }
        Ok(members)
    }

    /// 添加成员
    ///
    /// 名字按 `lang` 语言写入名字表；同名成员已存在时返回既有成员 ID。
    pub async fn add(&self, name: String, lang: &str) -> Result<MemberId, AccountingError> {
        self.db
            .member_get_or_create_by_name(&name, lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除成员
    pub async fn delete(&self, id: MemberId) -> Result<(), AccountingError> {
        self.db
            .member_delete(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 按名称查找成员
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Member>, AccountingError> {
        self.db
            .member_get_by_name(name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    #[tokio::test]
    async fn test_member_lifecycle() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let service = MemberService::new(db);

        let id = service.add("Alice".to_string(), "en").await.unwrap();
        assert!(id.0 > 0);

        let list = service.list(None, None).await.unwrap();
        assert!(list.iter().any(|m| m.id == id));
        assert!(
            service
                .db
                .member_get_by_name("Alice")
                .await
                .unwrap()
                .is_some()
        );

        service.delete(id).await.unwrap();
        let list = service.list(None, None).await.unwrap();
        assert!(!list.iter().any(|m| m.id == id));
    }
}
