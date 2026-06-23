use accounting::account::Account;
use accounting::id::{AccountId, MemberId};
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
    /// 根据完整路径（如 Assets:Bank:Checking）逐级查询账户
    fn get_by_name(
        &self,
        conn: &Connection,
        name: &str,
    ) -> Result<Option<Account>, crate::error::DbError>;
    /// 根据父账户 ID 和本级名称查询账户
    fn get_by_parent_and_name(
        &self,
        conn: &Connection,
        parent_id: Option<AccountId>,
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
    /// 重命名账户
    fn rename(
        &self,
        conn: &Connection,
        id: AccountId,
        new_name: &str,
    ) -> Result<(), crate::error::DbError>;
    /// 关闭账户（设置 closed_at）
    fn close(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
    /// 重新开启账户（清除 closed_at）
    fn reopen(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
    /// 删除账户（物理删除）
    fn delete(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError>;
    /// 创建账户并自动维护闭包表
    fn create_with_closure(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError>;
    /// 获取账户所有者列表
    fn get_owners(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<MemberId>, crate::error::DbError>;
    /// 设置账户所有者（替换现有关系）
    fn set_owners(
        &self,
        conn: &Connection,
        account_id: AccountId,
        member_ids: &[MemberId],
    ) -> Result<(), crate::error::DbError>;
    /// 查找账户根节点名称
    fn find_root_name(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<String, crate::error::DbError>;
    /// 查找账户根节点 ID
    fn find_root_id(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<AccountId, crate::error::DbError>;
}

/// SQLite AccountRepo 实现
#[derive(Clone)]
pub struct SqliteAccountRepo;

impl AccountRepo for SqliteAccountRepo {
    fn create(
        &self,
        conn: &Connection,
        account: &Account,
    ) -> Result<AccountId, crate::error::DbError> {
        conn.execute(
            "INSERT INTO accounts
             (name, parent_id, is_system, billing_day, repayment_day)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                account.name,
                account.parent_id.map(|id| id.0),
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
            "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE id = ?1",
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
        let segments: Vec<&str> = name.split(':').collect();
        if segments.is_empty() {
            return Ok(None);
        }

        let mut parent_id: Option<AccountId> = None;
        for segment in segments {
            match self.get_by_parent_and_name(conn, parent_id, segment)? {
                Some(account) => parent_id = Some(account.id),
                None => return Ok(None),
            }
        }

        self.get(conn, parent_id.unwrap())
    }

    fn get_by_parent_and_name(
        &self,
        conn: &Connection,
        parent_id: Option<AccountId>,
        name: &str,
    ) -> Result<Option<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE parent_id IS ?1 AND name = ?2",
        )?;
        let mut rows = stmt.query(params![parent_id.map(|id| id.0), name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_account(row)?))
        } else {
            Ok(None)
        }
    }

    fn list(&self, conn: &Connection) -> Result<Vec<Account>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts ORDER BY id",
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
            "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
             FROM accounts WHERE parent_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![parent_id.0], map_account)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn rename(
        &self,
        conn: &Connection,
        id: AccountId,
        new_name: &str,
    ) -> Result<(), crate::error::DbError> {
        conn.execute(
            "UPDATE accounts SET name = ?1 WHERE id = ?2",
            params![new_name, id.0],
        )?;
        Ok(())
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

    fn delete(&self, conn: &Connection, id: AccountId) -> Result<(), crate::error::DbError> {
        conn.execute(
            "DELETE FROM account_owners WHERE account_id = ?1",
            params![id.0],
        )?;
        conn.execute(
            "DELETE FROM account_ancestors WHERE account_id = ?1",
            params![id.0],
        )?;
        conn.execute("DELETE FROM accounts WHERE id = ?1", params![id.0])?;
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

    fn get_owners(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<Vec<MemberId>, crate::error::DbError> {
        let mut stmt =
            conn.prepare("SELECT member_id FROM account_owners WHERE account_id = ?1")?;
        let rows = stmt.query_map(params![account_id.0], |row| Ok(MemberId(row.get(0)?)))?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    fn set_owners(
        &self,
        conn: &Connection,
        account_id: AccountId,
        member_ids: &[MemberId],
    ) -> Result<(), crate::error::DbError> {
        conn.execute(
            "DELETE FROM account_owners WHERE account_id = ?1",
            params![account_id.0],
        )?;
        for member_id in member_ids {
            conn.execute(
                "INSERT INTO account_owners (account_id, member_id) VALUES (?1, ?2)",
                params![account_id.0, member_id.0],
            )?;
        }
        Ok(())
    }

    fn find_root_name(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<String, crate::error::DbError> {
        let name: String = conn.query_row(
            "SELECT a.name FROM account_ancestors aa
             JOIN accounts a ON aa.ancestor_id = a.id
             WHERE aa.account_id = ?1
             ORDER BY aa.depth DESC
             LIMIT 1",
            params![account_id.0],
            |row| row.get(0),
        )?;
        Ok(name)
    }

    fn find_root_id(
        &self,
        conn: &Connection,
        account_id: AccountId,
    ) -> Result<AccountId, crate::error::DbError> {
        let id: i64 = conn.query_row(
            "SELECT ancestor_id FROM account_ancestors
             WHERE account_id = ?1
             ORDER BY depth DESC
             LIMIT 1",
            params![account_id.0],
            |row| row.get(0),
        )?;
        Ok(AccountId(id))
    }
}

fn map_account(row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
    let closed_at: Option<String> = row.get(3)?;
    let closed_at = match closed_at {
        Some(s) => Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?),
        None => None,
    };

    Ok(Account {
        id: AccountId(row.get(0)?),
        name: row.get(1)?,
        parent_id: row.get::<_, Option<i64>>(2)?.map(AccountId),
        closed_at,
        is_system: row.get::<_, i32>(4)? != 0,
        billing_day: row.get::<_, Option<i32>>(5)?.map(|v| v as u8),
        repayment_day: row.get::<_, Option<i32>>(6)?.map(|v| v as u8),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
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
            name: "Bank".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = repo.create(&conn, &account).unwrap();
        let fetched = repo.get(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.name, "Bank");
        assert!(!fetched.is_system);
    }

    #[test]
    fn test_get_by_name() {
        let (conn, repo) = setup();
        let found = repo.get_by_name(&conn, "Equity:OpeningBalances").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "OpeningBalances");
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
            name: "TestParent".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let parent_id = repo.create(&conn, &parent).unwrap();

        let child = Account {
            id: AccountId(0),
            name: "Child".to_string(),
            parent_id: Some(parent_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        repo.create(&conn, &child).unwrap();

        let children = repo.list_children(&conn, parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Child");
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

    #[test]
    fn test_find_root_name_returns_root_for_child() {
        let (conn, repo) = setup();
        let assets = Account {
            id: AccountId(0),
            name: "Assets".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = repo.create_with_closure(&conn, &assets).unwrap();
        let bank = Account {
            id: AccountId(0),
            name: "Bank".to_string(),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = repo.create_with_closure(&conn, &bank).unwrap();
        let checking = Account {
            id: AccountId(0),
            name: "Checking".to_string(),
            parent_id: Some(bank_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let checking_id = repo.create_with_closure(&conn, &checking).unwrap();

        assert_eq!(repo.find_root_name(&conn, checking_id).unwrap(), "Assets");
        assert_eq!(repo.find_root_id(&conn, bank_id).unwrap(), assets_id);
    }

    #[test]
    fn test_find_root_returns_self_for_root() {
        let (conn, repo) = setup();
        let equity = Account {
            id: AccountId(0),
            name: "Equity".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let equity_id = repo.create_with_closure(&conn, &equity).unwrap();

        assert_eq!(repo.find_root_name(&conn, equity_id).unwrap(), "Equity");
        assert_eq!(repo.find_root_id(&conn, equity_id).unwrap(), equity_id);
    }

    #[test]
    fn test_find_root_name_with_chinese_name() {
        let (conn, repo) = setup();
        let assets_cn = Account {
            id: AccountId(0),
            name: "资产".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = repo.create_with_closure(&conn, &assets_cn).unwrap();
        let bank_cn = Account {
            id: AccountId(0),
            name: "银行".to_string(),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = repo.create_with_closure(&conn, &bank_cn).unwrap();

        assert_eq!(repo.find_root_name(&conn, bank_id).unwrap(), "资产");
        assert_eq!(repo.find_root_id(&conn, bank_id).unwrap(), assets_id);
        assert_eq!(repo.find_root_name(&conn, assets_id).unwrap(), "资产");
    }
}
