use accounting::commodity::Commodity;
use accounting::error::AccountingError;
use accounting::id::CommodityId;
use accounting_sql::SqliteDatabase;

/// 商品/货币服务
pub struct CommodityService {
    db: SqliteDatabase,
}

impl CommodityService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 列出所有商品
    pub async fn list(&self) -> Result<Vec<Commodity>, AccountingError> {
        self.db
            .commodity_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 添加商品
    ///
    /// 名字按 `lang` 语言写入名字表；同 symbol 商品已存在时更新精度并返回既有 ID。
    pub async fn add(
        &self,
        symbol: String,
        name: String,
        precision: u8,
        lang: &str,
    ) -> Result<CommodityId, AccountingError> {
        self.db
            .commodity_upsert_by_symbol(&symbol, &name, precision, lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    #[tokio::test]
    async fn test_commodity_lifecycle() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let service = CommodityService::new(db);

        let id = service
            .add("USD".to_string(), "美元".to_string(), 2, "zh-CN")
            .await
            .unwrap();
        assert!(id.0 > 0);

        let list = service.list().await.unwrap();
        assert!(list.iter().any(|c| c.symbol == "USD"));
    }
}
