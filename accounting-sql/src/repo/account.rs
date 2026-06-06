use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::id::AccountId;
use chrono::NaiveDate;
use rusqlite::{Connection, params};

/// Account 仓库 trait
pub trait AccountRepo {
    /// 创建账户，返回新账户 ID
    fn create(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError>;
    /// 根据 ID 查询账户
    fn get(
        &self,
        conn: &Connection,
        id: AccountId,
    ) -> Result<Option<Account>, crate::error::DbError>;
    /// 根据 full_name 查询账户
    fn get_by_name(
        &self,
        conn: &Connection,
        name: &str,
    ) -> Result<Option<Account>, crate::error::DbError>;
    /// 列出所有账户
    fn list(&self, conn: &Connection) -> Result<Vec<Account>, crate::error::DbError>;
    /// 列出某账户的直接子账户
    fn list_children(
        &self,
        conn: &Connection,
        parent_id: AccountId,
    ) -> Result<Vec<Account>, crate::error::DbError>;
    /// 关闭账户（设置 closed_at）
    fn close(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
    /// 重新开启账户（清除 closed_at）
    fn reopen(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
    /// 创建账户并自动维护闭包表
    fn create_with_closure(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError>;
}

/// SQLite AccountRepo 实现
pub struct SqliteAccountRepo;

impl AccountRepo for SqliteAccountRepo {
    fn create(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO accounts
             (full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                account.full_name,
                account.account_type as i32,
                account.parent_id.map(|id| id.0),
                account.closed_at.map(|d| d.to_string()),
                account.is_system as i32,
                account.billing_day,
                account.repayment_day,
            ],
        )?;
        Ok(AccountId(conn.last_insert_rowid()))
    }

    fn get(
        &self,
        conn: &Connection,
        id: AccountId,
    ) -> Result<Option<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_by_name(
        &self,
        conn: &Connection,
        name: &str,
    ) -> Result<Option<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE full_name = ?1"
        )?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account(row)?))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts ORDER BY id"
        )?;
        let rows = stmt.query_map([], map_account)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn list_children(
        &self,
        conn: &Connection,
        parent_id: AccountId,
    ) -> Result<Vec<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, account_type, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE parent_id = ?1 ORDER BY id"
        )?;
        let rows = stmt.query_map(params![parent_id.0], map_account)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn close(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE accounts SET closed_at = date('now') WHERE id = ?1",
            params![id.0],
        )?;
        Ok(())
    }

    fn reopen(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE accounts SET closed_at = NULL WHERE id = ?1",
            params![id.0],
        )?;
        Ok(())
    }

    fn create_with_closure(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError> {
        let id = self.create(conn, account)?;

        // 维护闭包表：自身 depth=0
        conn.execute(
            "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?1, 0)",
            params![id.0],
        )?;

        if let Some(parent_id) = account.parent_id {
            // 直接父节点 depth=1
            conn.execute(
                "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?2, 1)",
                params![id.0, parent_id.0],
            )?;

            // 继承父节点的祖先
            let mut stmt = conn.prepare(
                "SELECT ancestor_id, depth FROM account_ancestors WHERE account_id = ?1",
            )?;
            let rows = stmt.query_map(params![parent_id.0], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
            })?;
            for result in rows {
                let (ancestor_id, depth) = result?;
                if ancestor_id == parent_id.0 {
                    continue; // 已在上面插入
                }
                conn.execute(
                    "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?2, ?3)",
                    params![id.0, ancestor_id, depth + 1],
                )?;
            }
        }

        Ok(id)
    }
}

fn map_account(row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
    let type_int: i32 = row.get(2)?;
    let account_type = match type_int {
        1 => AccountType::Asset,
        2 => AccountType::Liability,
        3 => AccountType::Equity,
        4 => AccountType::Income,
        5 => AccountType::Expense,
        _ => AccountType::Asset,
    };

    let closed_at: Option<String> = row.get(4)?;
    let closed_at = match closed_at {
        Some(s) => Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
        })?),
        None => None,
    };

    Ok(Account {
        id: AccountId(row.get(0)?),
        full_name: row.get(1)?,
        account_type,
        parent_id: row.get::<_, Option<i64>>(3)?.map(AccountId),
        closed_at,
        is_system: row.get::<_, i32>(5)? != 0,
        billing_day: row.get::<_, Option<i32>>(6)?.map(|v| v as u8),
        repayment_day: row.get::<_, Option<i32>>(7)?.map(|v| v as u8),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::AccountId;
    use rusqlite::Connection;

    fn setup() -> (Connection, SqliteAccountRepo) {
        let conn = Connection::open_in_memory().unwrap();
        crate::schema::initialize_schema(&conn).unwrap();
        crate::schema::insert_seed_data(&conn, "en").unwrap();
        (conn, SqliteAccountRepo)
    }

    #[test]
    fn test_create_and_get() {
        let (conn, repo) = setup();
        let account = Account {
            id: AccountId(0),
            full_name: "Assets:Cash".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = repo.create(&conn, &account).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.full_name, "Assets:Cash");
        assert_eq!(fetched.account_type, AccountType::Asset);
        assert!(!fetched.is_system);
    }

    #[test]
    fn test_get_by_name() {
        let (conn, repo) = setup();
        let found = repo.get_by_name(&conn, "Equity:OpeningBalances").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().account_type, AccountType::Equity);
    }

    #[test]
    fn test_list() {
        let (conn, repo) = setup();
        let list = repo.list(&conn).unwrap();
        assert!(list.len() >= 7);
    }

    #[test]
    fn test_list_children() {
        let (conn, repo) = setup();
        let parent = Account {
            id: AccountId(0),
            full_name: "Assets".to_string(),
            account_type: AccountType::Asset,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let parent_id = repo.create(&conn, &parent).unwrap();

        let child = Account {
            id: AccountId(0),
            full_name: "Assets:Bank".to_string(),
            account_type: AccountType::Asset,
            parent_id: Some(parent_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        repo.create(&conn, &child).unwrap();

        let children = repo.list_children(&conn, parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].full_name, "Assets:Bank");
    }

    #[test]
    fn test_close_and_reopen() {
        let (conn, repo) = setup();
        let found = repo
            .get_by_name(&conn, "Equity:OpeningBalances")
            .unwrap()
            .unwrap();
        repo.close(&conn, found.id).unwrap();
        let closed = repo.get(&conn, found.id).unwrap().unwrap();
        assert!(closed.closed_at.is_some());

        repo.reopen(&conn, found.id).unwrap();
        let reopened = repo.get(&conn, found.id).unwrap().unwrap();
        assert!(reopened.closed_at.is_none());
    }
}
