use accounting::error::AccountingError;
use accounting::id::MemberId;
use accounting::member::Member;
use accounting_sql::database::Database;

/// 成员服务
pub struct MemberService<D: Database> {
    db: D,
}

impl<D: Database> MemberService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 列出所有成员
    pub async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Member>, AccountingError> {
        let conn = self.db.connection();
        let mut members = self.db.member_repo().list(&conn).map_err(|e| AccountingError::Unknown(e.to_string()))?;
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
    pub async fn add(&self, name: String, description: Option<String>) -> Result<MemberId, AccountingError> {
        let conn = self.db.connection();
        let member = Member {
            id: MemberId(0),
            name,
            description,
        };
        let id = self.db.member_repo().create(&conn, &member).map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(id)
    }

    /// 删除成员
    pub async fn delete(&self, id: MemberId) -> Result<(), AccountingError> {
        let conn = self.db.connection();
        self.db.member_repo().delete(&conn, id).map_err(|e| AccountingError::Unknown(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::impls::sqlite::SqliteDatabase;

    #[tokio::test]
    async fn test_member_lifecycle() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = MemberService::new(db);

        let id = service.add("Alice".to_string(), Some("Tester".to_string())).await.unwrap();
        assert!(id.0 > 0);

        let list = service.list(None, None).await.unwrap();
        assert!(list.iter().any(|m| m.name == "Alice"));

        service.delete(id).await.unwrap();
        let list = service.list(None, None).await.unwrap();
        assert!(!list.iter().any(|m| m.name == "Alice"));
    }
}
