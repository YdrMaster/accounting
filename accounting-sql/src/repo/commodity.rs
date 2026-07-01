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
    created_at: Option<String>,
}

impl CommodityRow {
    fn into_commodity(self) -> Commodity {
        let created_at = self.created_at.and_then(|s| {
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .or_else(|| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        });
        Commodity {
            id: CommodityId(self.id),
            symbol: self.symbol,
            name: self.name,
            precision: self.precision as u8,
            created_at,
        }
    }
}

pub async fn commodity_get_by_symbol(
    conn: &mut SqliteConnection,
    symbol: &str,
) -> Result<Option<Commodity>, DbError> {
    let row: Option<CommodityRow> = sqlx::query_as(
        "SELECT id, symbol, name, precision, created_at FROM commodities WHERE symbol = ?1",
    )
    .bind(symbol)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(row.map(|r| r.into_commodity()))
}

pub async fn commodity_list(conn: &mut SqliteConnection) -> Result<Vec<Commodity>, DbError> {
    let rows: Vec<CommodityRow> = sqlx::query_as(
        "SELECT id, symbol, name, precision, created_at FROM commodities ORDER BY id",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|r| r.into_commodity()).collect())
}

pub async fn commodity_created_at_map(
    conn: &mut SqliteConnection,
) -> Result<std::collections::HashMap<accounting::id::CommodityId, chrono::NaiveDate>, DbError> {
    #[derive(sqlx::FromRow)]
    struct CreatedAtRow {
        id: i64,
        created_at: Option<String>,
    }

    let rows: Vec<CreatedAtRow> = sqlx::query_as("SELECT id, created_at FROM commodities")
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let mut map = std::collections::HashMap::new();
    for row in rows {
        if let Some(date) = row.created_at.and_then(|s| {
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .or_else(|| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        }) {
            map.insert(accounting::id::CommodityId(row.id), date);
        }
    }
    Ok(map)
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

pub async fn commodity_upsert_by_symbol(
    conn: &mut SqliteConnection,
    symbol: &str,
    name: &str,
    precision: u8,
) -> Result<CommodityId, DbError> {
    if let Some(existing) = commodity_get_by_symbol(conn, symbol).await? {
        sqlx::query("UPDATE commodities SET name = ?1, precision = ?2 WHERE id = ?3")
            .bind(name)
            .bind(precision as i32)
            .bind(existing.id.0)
            .execute(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(existing.id)
    } else {
        let commodity = Commodity {
            id: CommodityId(0),
            symbol: symbol.to_string(),
            name: name.to_string(),
            precision,
            created_at: None,
        };
        commodity_create(conn, &commodity).await
    }
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
            created_at: None,
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
