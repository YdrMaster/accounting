use chrono::NaiveDate;
use sqlx::SqliteConnection;

use crate::error::DbError;
use accounting::account::Account;
use accounting::id::{AccountId, MemberId};

pub async fn account_create(
    conn: &mut SqliteConnection,
    account: &Account,
) -> Result<AccountId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts
         (name, parent_id, is_system, billing_day, repayment_day)
         VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
    )
    .bind(&account.name)
    .bind(account.parent_id.map(|id| id.0))
    .bind(account.is_system as i32)
    .bind(account.billing_day.map(|v| v as i32))
    .bind(account.repayment_day.map(|v| v as i32))
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(AccountId(id))
}

pub async fn account_get(
    conn: &mut SqliteConnection,
    id: AccountId,
) -> Result<Option<Account>, DbError> {
    let row: Option<AccountRow> = sqlx::query_as(
        "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
         FROM accounts WHERE id = ?1",
    )
    .bind(id.0)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn account_get_by_name(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<Option<Account>, DbError> {
    let segments: Vec<&str> = name.split(':').collect();
    if segments.is_empty() {
        return Ok(None);
    }

    let mut parent_id: Option<AccountId> = None;
    for segment in segments {
        match account_get_by_parent_and_name(conn, parent_id, segment).await? {
            Some(account) => parent_id = Some(account.id),
            None => return Ok(None),
        }
    }

    account_get(conn, parent_id.unwrap()).await
}

pub async fn account_get_by_parent_and_name(
    conn: &mut SqliteConnection,
    parent_id: Option<AccountId>,
    name: &str,
) -> Result<Option<Account>, DbError> {
    let row: Option<AccountRow> = sqlx::query_as(
        "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
         FROM accounts WHERE parent_id IS ?1 AND name = ?2",
    )
    .bind(parent_id.map(|id| id.0))
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    row.map(|r| r.try_into()).transpose()
}

pub async fn account_list(conn: &mut SqliteConnection) -> Result<Vec<Account>, DbError> {
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
         FROM accounts ORDER BY id",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    rows.into_iter()
        .map(|r| r.try_into())
        .collect::<Result<_, _>>()
}

pub async fn account_list_children(
    conn: &mut SqliteConnection,
    parent_id: AccountId,
) -> Result<Vec<Account>, DbError> {
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT id, name, parent_id, closed_at, is_system, billing_day, repayment_day
         FROM accounts WHERE parent_id = ?1 ORDER BY id",
    )
    .bind(parent_id.0)
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    rows.into_iter()
        .map(|r| r.try_into())
        .collect::<Result<_, _>>()
}

pub async fn account_rename(
    conn: &mut SqliteConnection,
    id: AccountId,
    new_name: &str,
) -> Result<(), DbError> {
    sqlx::query("UPDATE accounts SET name = ?1 WHERE id = ?2")
        .bind(new_name)
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn account_update_fields(
    conn: &mut SqliteConnection,
    id: AccountId,
    billing_day: Option<u8>,
    repayment_day: Option<u8>,
) -> Result<(), DbError> {
    sqlx::query("UPDATE accounts SET billing_day = ?1, repayment_day = ?2 WHERE id = ?3")
        .bind(billing_day.map(|v| v as i32))
        .bind(repayment_day.map(|v| v as i32))
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn account_close(conn: &mut SqliteConnection, id: AccountId) -> Result<(), DbError> {
    sqlx::query("UPDATE accounts SET closed_at = date('now') WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn account_reopen(conn: &mut SqliteConnection, id: AccountId) -> Result<(), DbError> {
    sqlx::query("UPDATE accounts SET closed_at = NULL WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn account_delete(conn: &mut SqliteConnection, id: AccountId) -> Result<(), DbError> {
    sqlx::query("DELETE FROM account_owners WHERE account_id = ?1")
        .bind(id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    sqlx::query("DELETE FROM account_ancestors WHERE account_id = ?1")
        .bind(id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    sqlx::query("DELETE FROM accounts WHERE id = ?1")
        .bind(id.0)
        .execute(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(())
}

pub async fn account_get_or_create_by_path(
    conn: &mut SqliteConnection,
    path: &str,
) -> Result<AccountId, DbError> {
    if let Some(account) = account_get_by_name(conn, path).await? {
        return Ok(account.id);
    }

    let segments: Vec<&str> = path.split(':').collect();
    if segments.is_empty() {
        return Err(DbError::Database("empty account path".to_string()));
    }

    let mut parent_id: Option<AccountId> = None;
    for segment in segments {
        match account_get_by_parent_and_name(conn, parent_id, segment).await? {
            Some(account) => {
                parent_id = Some(account.id);
            }
            None => {
                let account = Account {
                    id: AccountId(0),
                    name: segment.to_string(),
                    parent_id,
                    closed_at: None,
                    is_system: false,
                    billing_day: None,
                    repayment_day: None,
                };
                let id = account_create_with_closure(conn, &account).await?;
                parent_id = Some(id);
            }
        }
    }

    parent_id.ok_or_else(|| DbError::Database(format!("failed to create account path: {}", path)))
}

pub async fn account_update_by_path(
    conn: &mut SqliteConnection,
    path: &str,
    closed_at: Option<NaiveDate>,
    billing_day: Option<u8>,
    repayment_day: Option<u8>,
) -> Result<(), DbError> {
    let account = account_get_by_name(conn, path)
        .await?
        .ok_or_else(|| DbError::Database(format!("account not found: {}", path)))?;

    sqlx::query(
        "UPDATE accounts SET closed_at = ?1, billing_day = ?2, repayment_day = ?3 WHERE id = ?4",
    )
    .bind(closed_at.map(|d| d.to_string()))
    .bind(billing_day.map(|v| v as i32))
    .bind(repayment_day.map(|v| v as i32))
    .bind(account.id.0)
    .execute(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    Ok(())
}

pub async fn account_rebuild_ancestors(conn: &mut SqliteConnection) -> Result<(), DbError> {
    sqlx::query("DELETE FROM account_ancestors")
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let accounts = account_list(conn).await?;

    for account in accounts {
        sqlx::query(
            "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?1, 0)",
        )
        .bind(account.id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

        let mut current = account.parent_id;
        let mut depth = 1i32;
        while let Some(parent_id) = current {
            sqlx::query(
                "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?2, ?3)",
            )
            .bind(account.id.0)
            .bind(parent_id.0)
            .bind(depth)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;

            let parent = account_get(conn, parent_id).await?;
            current = parent.and_then(|p| p.parent_id);
            depth += 1;
        }
    }

    Ok(())
}

pub async fn account_create_with_closure(
    conn: &mut SqliteConnection,
    account: &Account,
) -> Result<AccountId, DbError> {
    let id = account_create(conn, account).await?;

    // 维护闭包表：自身 depth=0
    sqlx::query(
        "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?1, 0)",
    )
    .bind(id.0)
    .execute(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    if let Some(parent_id) = account.parent_id {
        // 直接父节点 depth=1
        sqlx::query(
            "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?2, 1)",
        )
        .bind(id.0)
        .bind(parent_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

        // 继承父节点的祖先
        let rows: Vec<(i64, i32)> = sqlx::query_as(
            "SELECT ancestor_id, depth FROM account_ancestors WHERE account_id = ?1",
        )
        .bind(parent_id.0)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

        for (ancestor_id, depth) in rows {
            if ancestor_id == parent_id.0 {
                continue; // 已在上面插入
            }
            sqlx::query(
                "INSERT INTO account_ancestors (account_id, ancestor_id, depth) VALUES (?1, ?2, ?3)",
            )
            .bind(id.0)
            .bind(ancestor_id)
            .bind(depth + 1)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        }
    }

    Ok(id)
}

pub async fn account_get_owners(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<Vec<MemberId>, DbError> {
    let rows: Vec<(i64,)> =
        sqlx::query_as("SELECT member_id FROM account_owners WHERE account_id = ?1")
            .bind(account_id.0)
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows.into_iter().map(|(id,)| MemberId(id)).collect())
}

pub async fn account_list_owners(
    conn: &mut SqliteConnection,
) -> Result<Vec<(AccountId, MemberId)>, DbError> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT ao.account_id, ao.member_id
         FROM account_owners ao
         ORDER BY ao.account_id, ao.member_id",
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(a, m)| (AccountId(a), MemberId(m)))
        .collect())
}

pub async fn account_set_owners(
    conn: &mut SqliteConnection,
    account_id: AccountId,
    member_ids: &[MemberId],
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM account_owners WHERE account_id = ?1")
        .bind(account_id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    for member_id in member_ids {
        sqlx::query("INSERT INTO account_owners (account_id, member_id) VALUES (?1, ?2)")
            .bind(account_id.0)
            .bind(member_id.0)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }
    Ok(())
}

pub async fn account_find_root_name(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<String, DbError> {
    let name: String = sqlx::query_scalar(
        "SELECT a.name FROM account_ancestors aa
         JOIN accounts a ON aa.ancestor_id = a.id
         WHERE aa.account_id = ?1
         ORDER BY aa.depth DESC
         LIMIT 1",
    )
    .bind(account_id.0)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(name)
}

pub async fn account_find_root_id(
    conn: &mut SqliteConnection,
    account_id: AccountId,
) -> Result<AccountId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "SELECT ancestor_id FROM account_ancestors
         WHERE account_id = ?1
         ORDER BY depth DESC
         LIMIT 1",
    )
    .bind(account_id.0)
    .fetch_one(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(AccountId(id))
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: i64,
    name: String,
    parent_id: Option<i64>,
    closed_at: Option<String>,
    is_system: i32,
    billing_day: Option<i32>,
    repayment_day: Option<i32>,
}

impl TryFrom<AccountRow> for Account {
    type Error = DbError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        let closed_at = match row.closed_at {
            Some(s) => Some(
                NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map_err(|e| DbError::Database(e.to_string()))?,
            ),
            None => None,
        };

        Ok(Account {
            id: AccountId(row.id),
            name: row.name,
            parent_id: row.parent_id.map(AccountId),
            closed_at,
            is_system: row.is_system != 0,
            billing_day: row.billing_day.map(|v| v as u8),
            repayment_day: row.repayment_day.map(|v| v as u8),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::AccountId;
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
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let account = Account {
            id: AccountId(0),
            name: "Bank".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = account_create(&mut conn, &account).await.unwrap();
        let fetched = account_get(&mut conn, id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Bank");
        assert!(!fetched.is_system);
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let mut conn = setup().await;
        let found = account_get_by_name(&mut conn, "Equity:OpeningBalances")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "OpeningBalances");
    }

    #[tokio::test]
    async fn test_list() {
        let mut conn = setup().await;
        let list = account_list(&mut conn).await.unwrap();
        assert!(list.len() >= 7);
    }

    #[tokio::test]
    async fn test_list_children() {
        let mut conn = setup().await;
        let parent = Account {
            id: AccountId(0),
            name: "TestParent".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let parent_id = account_create(&mut conn, &parent).await.unwrap();

        let child = Account {
            id: AccountId(0),
            name: "Child".to_string(),
            parent_id: Some(parent_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        account_create(&mut conn, &child).await.unwrap();

        let children = account_list_children(&mut conn, parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Child");
    }

    #[tokio::test]
    async fn test_close_and_reopen() {
        let mut conn = setup().await;
        let found = account_get_by_name(&mut conn, "Equity:OpeningBalances")
            .await
            .unwrap()
            .unwrap();
        account_close(&mut conn, found.id).await.unwrap();
        let closed = account_get(&mut conn, found.id).await.unwrap().unwrap();
        assert!(closed.closed_at.is_some());

        account_reopen(&mut conn, found.id).await.unwrap();
        let reopened = account_get(&mut conn, found.id).await.unwrap().unwrap();
        assert!(reopened.closed_at.is_none());
    }

    #[tokio::test]
    async fn test_find_root_name_returns_root_for_child() {
        let mut conn = setup().await;
        let assets = Account {
            id: AccountId(0),
            name: "Assets".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = account_create_with_closure(&mut conn, &assets)
            .await
            .unwrap();
        let bank = Account {
            id: AccountId(0),
            name: "Bank".to_string(),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = account_create_with_closure(&mut conn, &bank).await.unwrap();
        let checking = Account {
            id: AccountId(0),
            name: "Checking".to_string(),
            parent_id: Some(bank_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let checking_id = account_create_with_closure(&mut conn, &checking)
            .await
            .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, checking_id)
                .await
                .unwrap(),
            "Assets"
        );
        assert_eq!(
            account_find_root_id(&mut conn, bank_id).await.unwrap(),
            assets_id
        );
    }

    #[tokio::test]
    async fn test_find_root_returns_self_for_root() {
        let mut conn = setup().await;
        let equity = Account {
            id: AccountId(0),
            name: "Equity".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let equity_id = account_create_with_closure(&mut conn, &equity)
            .await
            .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, equity_id).await.unwrap(),
            "Equity"
        );
        assert_eq!(
            account_find_root_id(&mut conn, equity_id).await.unwrap(),
            equity_id
        );
    }

    #[tokio::test]
    async fn test_find_root_name_with_chinese_name() {
        let mut conn = setup().await;
        let assets_cn = Account {
            id: AccountId(0),
            name: "资产".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = account_create_with_closure(&mut conn, &assets_cn)
            .await
            .unwrap();
        let bank_cn = Account {
            id: AccountId(0),
            name: "银行".to_string(),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = account_create_with_closure(&mut conn, &bank_cn)
            .await
            .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, bank_id).await.unwrap(),
            "资产"
        );
        assert_eq!(
            account_find_root_id(&mut conn, bank_id).await.unwrap(),
            assets_id
        );
        assert_eq!(
            account_find_root_name(&mut conn, assets_id).await.unwrap(),
            "资产"
        );
    }
}
