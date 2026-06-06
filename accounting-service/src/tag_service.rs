use accounting::error::AccountingError;
use accounting::id::TagId;
use accounting::tag::Tag;
use accounting_sql::database::Database;

/// 标签服务
pub struct TagService<D: Database> {
    db: D,
}

impl<D: Database> TagService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 列出所有标签
    pub async fn list(&self) -> Result<Vec<Tag>, AccountingError> {
        let conn = self.db.connection();
        let tags = self
            .db
            .tag_repo()
            .list(&conn)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(tags)
    }

    /// 添加标签
    pub async fn add(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<TagId, AccountingError> {
        let conn = self.db.connection();
        let tag = Tag {
            id: TagId(0),
            name,
            description,
            is_system: false,
        };
        let id = self
            .db
            .tag_repo()
            .create(&conn, &tag)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(id)
    }

    /// 删除标签
    pub async fn delete(&self, name: &str) -> Result<(), AccountingError> {
        let conn = self.db.connection();
        self.db
            .tag_repo()
            .delete(&conn, name)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::impls::sqlite::SqliteDatabase;

    #[tokio::test]
    async fn test_tag_lifecycle() {
        let db = SqliteDatabase::open_in_memory().unwrap();
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
