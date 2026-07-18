use accounting::error::AccountingError;
use accounting::id::TagId;
use accounting::tag::Tag;
use accounting_sql::SqliteDatabase;
/// 标签服务
pub struct TagService {
    db: SqliteDatabase,
}

impl TagService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 列出所有标签
    pub async fn list(&self) -> Result<Vec<Tag>, AccountingError> {
        self.db
            .tag_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 添加标签
    ///
    /// 名字按 `lang` 语言写入名字表；同名标签已存在时返回既有标签 ID。
    pub async fn add(
        &self,
        name: String,
        description: Option<String>,
        lang: &str,
    ) -> Result<TagId, AccountingError> {
        self.db
            .tag_upsert_by_name(&name, description.as_deref(), lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除标签
    pub async fn delete(&self, name: &str) -> Result<(), AccountingError> {
        self.db
            .tag_delete(name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    #[tokio::test]
    async fn test_tag_lifecycle() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let service = TagService::new(db);

        let id = service
            .add("travel".to_string(), Some("旅行".to_string()), "en")
            .await
            .unwrap();
        assert!(id.0 > 0);

        let list = service.list().await.unwrap();
        assert!(list.iter().any(|t| t.id == id));
        assert!(
            service
                .db
                .tag_get_by_name("travel")
                .await
                .unwrap()
                .is_some()
        );

        service.delete("travel").await.unwrap();
        let list = service.list().await.unwrap();
        assert!(!list.iter().any(|t| t.id == id));
        assert!(
            service
                .db
                .tag_get_by_name("travel")
                .await
                .unwrap()
                .is_none()
        );
    }
}
