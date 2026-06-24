use sqlx::{FromRow, SqliteConnection};

use crate::error::DbError;
use accounting::commodity::Commodity;
use accounting::id::CommodityId;

#[derive(FromRow)]
struct CommodityRow {
    id: i64,
    symbol: String,
    name: String,
    precision: i32,
}

impl CommodityRow {
    fn into_commodity(self) -> Commodity {
        Commodity {
            id: CommodityId(self.id),
            symbol: self.symbol,
            name: self.name,
            precision: self.precision as u8,
        }
    }
}

pub async fn commodity_get_by_symbol(
    conn: &mut SqliteConnection,
    symbol: &str,
) -> Result<Option<Commodity>, DbError> {
    let row: Option<CommodityRow> =
        sqlx::query_as("SELECT id, symbol, name, precision FROM commodities WHERE symbol = ?1")
            .bind(symbol)
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_commodity()))
}

pub async fn commodity_list(conn: &mut SqliteConnection) -> Result<Vec<Commodity>, DbError> {
    let rows: Vec<CommodityRow> =
        sqlx::query_as("SELECT id, symbol, name, precision FROM commodities ORDER BY id")
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_commodity()).collect())
}

pub async fn commodity_create(
    conn: &mut SqliteConnection,
    commodity: &Commodity,
) -> Result<CommodityId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO commodities (symbol, name, precision) VALUES (?1, ?2, ?3) RETURNING id",
    )
    .bind(&commodity.symbol)
    .bind(&commodity.name)
    .bind(commodity.precision as i32)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(CommodityId(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn, "en")
            .await
            .unwrap();
        conn
    }

    #[tokio::test]
    async fn test_get_by_symbol() {
        let mut conn = setup().await;
        let found = commodity_get_by_symbol(&mut conn, "CNY").await.unwrap();
        assert!(found.is_some());
        let c = found.unwrap();
        assert_eq!(c.symbol, "CNY");
        assert_eq!(c.name, "人民币");
        assert_eq!(c.precision, 2);
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let list = commodity_list(&mut conn).await.unwrap();
        assert!(!list.is_empty());
        assert!(list.iter().any(|c| c.symbol == "CNY"));
    }

    #[tokio::test]
    async fn test_create() {
        let mut conn = setup().await;
        let commodity = Commodity {
            id: CommodityId(0),
            symbol: "USD".to_string(),
            name: "美元".to_string(),
            precision: 2,
        };
        let id = commodity_create(&mut conn, &commodity).await.unwrap();
        let fetched = commodity_get_by_symbol(&mut conn, "USD")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "美元");
    }
}
