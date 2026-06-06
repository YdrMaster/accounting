use accounting::commodity::Commodity;
use accounting::id::CommodityId;
use rusqlite::{Connection, params};

/// Commodity 仓库 trait
pub trait CommodityRepo {
    /// 根据 symbol 查询商品
    fn get_by_symbol(
        &self,
        conn: &Connection,
        symbol: &str,
    ) -> Result<Option<Commodity>, crate::error::DbError>;
    /// 列出所有商品
    fn list(&self, conn: &Connection) -> Result<Vec<Commodity>, crate::error::DbError>;
    /// 创建商品
    fn create(
        &self,
        conn: &Connection,
        commodity: &Commodity,
    ) -> Result<CommodityId, crate::error::DbError>;
}

/// SQLite CommodityRepo 实现
pub struct SqliteCommodityRepo;

impl CommodityRepo for SqliteCommodityRepo {
    fn get_by_symbol(
        &self,
        conn: &Connection,
        symbol: &str,
    ) -> Result<Option<Commodity>, crate::error::DbError> {
        let mut stmt =
            conn.prepare("SELECT id, symbol, name, precision FROM commodities WHERE symbol = ?1")?;
        let mut rows = stmt.query(params![symbol])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Commodity {
                id: CommodityId(row.get(0)?),
                symbol: row.get(1)?,
                name: row.get(2)?,
                precision: row.get::<_, i32>(3)? as u8,
            }))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Commodity>, crate::error::DbError> {
        let mut stmt =
            conn.prepare("SELECT id, symbol, name, precision FROM commodities ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Commodity {
                id: CommodityId(row.get(0)?),
                symbol: row.get(1)?,
                name: row.get(2)?,
                precision: row.get::<_, i32>(3)? as u8,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn create(
        &self,
        conn: &Connection,
        commodity: &Commodity,
    ) -> Result<CommodityId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO commodities (symbol, name, precision) VALUES (?1, ?2, ?3)",
            params![commodity.symbol, commodity.name, commodity.precision as i32],
        )?;
        Ok(CommodityId(conn.last_insert_rowid()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteCommodityRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn, "en").unwrap();
        (conn, SqliteCommodityRepo)
    }

    #[test]
    fn test_get_by_symbol() {
        let (conn, repo) = setup();
        let found = repo.get_by_symbol(&conn, "CNY").unwrap();
        assert!(found.is_some());
        let c = found.unwrap();
        assert_eq!(c.symbol, "CNY");
        assert_eq!(c.name, "人民币");
        assert_eq!(c.precision, 2);
    }

    #[test]
    fn test_list() {
        let (conn, repo) = setup();
        let list = repo.list(&conn).unwrap();
        assert!(!list.is_empty());
        assert!(list.iter().any(|c| c.symbol == "CNY"));
    }

    #[test]
    fn test_create() {
        let (conn, repo) = setup();
        let commodity = Commodity {
            id: CommodityId(0),
            symbol: "USD".to_string(),
            name: "美元".to_string(),
            precision: 2,
        };
        let id = repo.create(&conn, &commodity).unwrap();
        let fetched = repo.get_by_symbol(&conn, "USD").unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "美元");
    }
}
