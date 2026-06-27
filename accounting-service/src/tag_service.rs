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
    pub async fn add(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<TagId, AccountingError> {
        let tag = Tag {
            id: TagId(0),
            name,
            description,
            is_system: false,
        };
        self.db
            .tag_create(&tag)
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
        db.initialize(Some("en")).await.unwrap();
        let service = TagService::new(db);

        let id = service
            .add("travel".to_string(), Some("旅行".to_string()))
            .await
            .unwrap();
        assert!(id.0 > 0);

        let list = service.list().await.unwrap();
        assert!(list.iter().any(|t| t.name == "travel"));

        service.delete("travel").await.unwrap();
        let list = service.list().await.unwrap();
        assert!(!list.iter().any(|t| t.name == "travel"));
    }
}
