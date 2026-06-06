use accounting::commodity::Commodity;
use accounting::error::AccountingError;
use accounting::id::CommodityId;
use accounting_sql::database::Database;

/// 商品/货币服务
pub struct CommodityService<D: Database> {
    db: D,
}

impl<D: Database> CommodityService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 列出所有商品
    pub async fn list(&self) -> Result<Vec<Commodity>, AccountingError> {
        let conn = self.db.connection();
        let commodities = self
            .db
            .commodity_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(commodities)
    }

    /// 添加商品
    pub async fn add(
        &self,
        symbol: String,
        name: String,
        precision: u8,
    ) -> Result<CommodityId, AccountingError> {
        let conn = self.db.connection();
        let commodity = Commodity {
            id: CommodityId(0),
            symbol,
            name,
            precision,
        };
        let id = self
            .db
            .commodity_repo()
            .create(&conn, &commodity)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::impls::sqlite::SqliteDatabase;

    #[tokio::test]
    async fn test_commodity_lifecycle() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let service = CommodityService::new(db);

        let id = service
            .add("USD".to_string(), "美元".to_string(), 2)
            .await
            .unwrap();
        assert!(id.0 > 0);

        let list = service.list().await.unwrap();
        assert!(list.iter().any(|c| c.symbol == "USD"));
    }
}
