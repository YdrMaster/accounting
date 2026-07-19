use chrono::NaiveDate;
use sqlx::SqliteConnection;
use std::collections::HashMap;

use crate::error::DbError;
use crate::names::ACCOUNT_NAMES;
use accounting::account::Account;
use accounting::id::{AccountId, MemberId};

pub async fn account_create(
    conn: &mut SqliteConnection,
    account: &Account,
) -> Result<AccountId, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts
         (parent_id, is_system, billing_day, repayment_day)
         VALUES (?1, ?2, ?3, ?4) RETURNING id",
    )
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
        "SELECT id, parent_id, closed_at, is_system, billing_day, repayment_day
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
        "SELECT a.id, a.parent_id, a.closed_at, a.is_system, a.billing_day, a.repayment_day
         FROM accounts a
         JOIN account_names an ON an.account_id = a.id
         WHERE a.parent_id IS ?1 AND an.name = ?2",
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
        "SELECT id, parent_id, closed_at, is_system, billing_day, repayment_day
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
        "SELECT id, parent_id, closed_at, is_system, billing_day, repayment_day
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

/// 改名：把 (账户, lang) 的显示名改为 new_name；该语言无显示名时新增显示名。
///
/// 系统名字不可改文本：改名时降级为非显示名并插入用户自定义显示名；
/// 新名字须在父账户命名空间内唯一（不区分大小写）。
pub async fn account_rename(
    conn: &mut SqliteConnection,
    id: AccountId,
    new_name: &str,
    lang: &str,
) -> Result<(), DbError> {
    let account = account_get(conn, id)
        .await?
        .ok_or_else(|| DbError::Database(format!("账户 {} 不存在", id.0)))?;
    ACCOUNT_NAMES
        .rename_display(conn, id.0, account.parent_id.map(|p| p.0), lang, new_name)
        .await
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
    lang: &str,
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
                let new_account = Account {
                    id: AccountId(0),
                    parent_id,
                    closed_at: None,
                    is_system: false,
                    billing_day: None,
                    repayment_day: None,
                };
                let id = account_create_with_closure(conn, &new_account).await?;
                // 自动创建的名字写入 account_names，语言由调用方指定（导入场景传 'und'）
                ACCOUNT_NAMES
                    .insert(conn, id.0, lang, segment, false, true)
                    .await?;
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

/// 为单个账户写入闭包行：自身 depth=0，沿父链逐级 depth 递增。
async fn account_insert_closure_rows(
    conn: &mut SqliteConnection,
    account: &Account,
) -> Result<(), DbError> {
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

        let parent = account_get(&mut *conn, parent_id).await?;
        current = parent.and_then(|p| p.parent_id);
        depth += 1;
    }
    Ok(())
}

pub async fn account_rebuild_ancestors(conn: &mut SqliteConnection) -> Result<(), DbError> {
    sqlx::query("DELETE FROM account_ancestors")
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let accounts = account_list(conn).await?;

    for account in accounts {
        account_insert_closure_rows(conn, &account).await?;
    }

    Ok(())
}

/// 判断 `ancestor_id` 是否为 `account_id` 的严格祖先（depth > 0）。
///
/// 用于移动账户前的成环检测：目标父账户是被移动账户的后代时即成环。
pub async fn account_is_descendant_of(
    conn: &mut SqliteConnection,
    account_id: AccountId,
    ancestor_id: AccountId,
) -> Result<bool, DbError> {
    let hit: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM account_ancestors
         WHERE account_id = ?1 AND ancestor_id = ?2 AND depth > 0",
    )
    .bind(account_id.0)
    .bind(ancestor_id.0)
    .fetch_optional(conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    Ok(hit.is_some())
}

/// 变更账户父节点，并重建被移动子树（自身 + 所有后代）的闭包行。
///
/// 调用方须保证：目标父账户存在、移动不会成环（可用 `account_is_descendant_of` 预检）。
/// 多步写入需由调用方包在一个事务里，失败整体回滚。
pub async fn account_update_parent(
    conn: &mut SqliteConnection,
    id: AccountId,
    new_parent_id: AccountId,
) -> Result<(), DbError> {
    sqlx::query("UPDATE accounts SET parent_id = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(new_parent_id.0)
        .bind(id.0)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    // 被移动子树 = 自身 + 所有后代（闭包表 depth=0 行即自身）
    let subtree: Vec<i64> =
        sqlx::query_scalar("SELECT account_id FROM account_ancestors WHERE ancestor_id = ?1")
            .bind(id.0)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;

    // 删除子树全部闭包行后按新父链重建
    for node in &subtree {
        sqlx::query("DELETE FROM account_ancestors WHERE account_id = ?1")
            .bind(node)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }
    for node in subtree {
        let account = account_get(&mut *conn, AccountId(node))
            .await?
            .ok_or_else(|| DbError::Database(format!("账户 {} 不存在", node)))?;
        account_insert_closure_rows(&mut *conn, &account).await?;
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

/// 用户创建账户并附带名字。
///
/// 名字语言由调用方传入；创建前校验父账户命名空间唯一性（不区分大小写，
/// 撞系统内置名同样拒绝）。
pub async fn account_create_with_name(
    conn: &mut SqliteConnection,
    account: &Account,
    name: &str,
    lang: &str,
) -> Result<AccountId, DbError> {
    ACCOUNT_NAMES
        .ensure_available(conn, account.parent_id.map(|p| p.0), None, name)
        .await?;
    let id = account_create_with_closure(conn, account).await?;
    ACCOUNT_NAMES
        .insert(conn, id.0, lang, name, false, true)
        .await?;
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

pub async fn account_created_at_map(
    conn: &mut SqliteConnection,
) -> Result<HashMap<AccountId, NaiveDate>, DbError> {
    #[derive(sqlx::FromRow)]
    struct CreatedAtRow {
        id: i64,
        created_at: String,
    }

    let rows: Vec<CreatedAtRow> = sqlx::query_as("SELECT id, created_at FROM accounts")
        .fetch_all(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;

    let mut map = HashMap::new();
    for row in rows {
        if let Ok(date) = NaiveDate::parse_from_str(&row.created_at, "%Y-%m-%d %H:%M:%S") {
            map.insert(AccountId(row.id), date);
        }
    }
    Ok(map)
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

/// 取账户的显示名（回退链：所选语言 → en → zh-CN → 插入序）。
pub async fn account_display_name(
    conn: &mut SqliteConnection,
    id: AccountId,
    lang: &str,
) -> Result<Option<String>, DbError> {
    ACCOUNT_NAMES.resolve_display(conn, id.0, lang).await
}

/// 取账户所属根账户的显示名（回退链：所选语言 → en → zh-CN → 插入序）。
pub async fn account_find_root_name(
    conn: &mut SqliteConnection,
    account_id: AccountId,
    lang: &str,
) -> Result<String, DbError> {
    let root_id = account_find_root_id(conn, account_id).await?;
    ACCOUNT_NAMES
        .resolve_display(conn, root_id.0, lang)
        .await?
        .ok_or_else(|| DbError::Database(format!("账户 {} 没有任何名字", root_id.0)))
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
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let mut conn = setup().await;
        let account = Account {
            id: AccountId(0),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let id = account_create(&mut conn, &account).await.unwrap();
        let fetched = account_get(&mut conn, id).await.unwrap().unwrap();
        assert!(!fetched.is_system);
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let mut conn = setup().await;
        let found = account_get_by_name(&mut conn, "Equity:OpeningBalances")
            .await
            .unwrap();
        assert!(found.is_some());
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
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let parent_id = account_create(&mut conn, &parent).await.unwrap();

        let child = Account {
            id: AccountId(0),
            parent_id: Some(parent_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        account_create(&mut conn, &child).await.unwrap();

        let children = account_list_children(&mut conn, parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
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
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = account_create_with_closure(&mut conn, &assets)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(assets_id.0)
        .bind("Assets")
        .execute(&mut conn)
        .await
        .unwrap();
        let bank = Account {
            id: AccountId(0),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = account_create_with_closure(&mut conn, &bank).await.unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(bank_id.0)
        .bind("Bank")
        .execute(&mut conn)
        .await
        .unwrap();
        let checking = Account {
            id: AccountId(0),
            parent_id: Some(bank_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let checking_id = account_create_with_closure(&mut conn, &checking)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(checking_id.0)
        .bind("Checking")
        .execute(&mut conn)
        .await
        .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, checking_id, "en")
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
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let equity_id = account_create_with_closure(&mut conn, &equity)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(equity_id.0)
        .bind("Equity")
        .execute(&mut conn)
        .await
        .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, equity_id, "en")
                .await
                .unwrap(),
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
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let assets_id = account_create_with_closure(&mut conn, &assets_cn)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(assets_id.0)
        .bind("资产")
        .execute(&mut conn)
        .await
        .unwrap();
        let bank_cn = Account {
            id: AccountId(0),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let bank_id = account_create_with_closure(&mut conn, &bank_cn)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display) VALUES (?1, 'en', ?2, 0, 1)",
        )
        .bind(bank_id.0)
        .bind("银行")
        .execute(&mut conn)
        .await
        .unwrap();

        assert_eq!(
            account_find_root_name(&mut conn, bank_id, "en")
                .await
                .unwrap(),
            "资产"
        );
        assert_eq!(
            account_find_root_id(&mut conn, bank_id).await.unwrap(),
            assets_id
        );
        assert_eq!(
            account_find_root_name(&mut conn, assets_id, "en")
                .await
                .unwrap(),
            "资产"
        );
        // 只有英文（此处 fixture 把中文文本放在 en 语言上）名字时，以中文查询按回退链取同一名字
        assert_eq!(
            account_find_root_name(&mut conn, assets_id, "zh-CN")
                .await
                .unwrap(),
            "资产"
        );
    }

    fn bare_account(parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    #[tokio::test]
    async fn test_create_with_name_and_namespace_uniqueness() {
        let mut conn = setup().await;
        let assets_id = account_get_by_name(&mut conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        let expenses_id = account_get_by_name(&mut conn, "Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;

        // 正常创建：名字按调用方语言写入，按任意语言名命中
        let id =
            account_create_with_name(&mut conn, &bare_account(Some(assets_id)), "招行", "zh-CN")
                .await
                .unwrap();
        assert!(
            account_get_by_name(&mut conn, "Assets:招行")
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            crate::names::ACCOUNT_NAMES
                .resolve_display(&mut conn, id.0, "en")
                .await
                .unwrap(),
            Some("招行".to_string())
        );

        // 同父级撞系统名（NOCASE）→ 拒绝
        assert!(
            account_create_with_name(&mut conn, &bare_account(Some(assets_id)), "cash", "en")
                .await
                .is_err()
        );
        assert!(
            account_create_with_name(&mut conn, &bare_account(Some(assets_id)), "现金", "zh-CN")
                .await
                .is_err()
        );
        // 不同父级允许同名
        assert!(
            account_create_with_name(&mut conn, &bare_account(Some(expenses_id)), "Cash", "en")
                .await
                .is_ok()
        );
        // 根命名空间撞系统根账户名 → 拒绝
        assert!(
            account_create_with_name(&mut conn, &bare_account(None), "ASSETS", "en")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_account_rename_applies() {
        let mut conn = setup().await;
        let assets_id = account_get_by_name(&mut conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        let id =
            account_create_with_name(&mut conn, &bare_account(Some(assets_id)), "OldBank", "en")
                .await
                .unwrap();

        // 英文改名生效：新名命中，旧名不再命中
        account_rename(&mut conn, id, "NewBank", "en")
            .await
            .unwrap();
        assert!(
            account_get_by_name(&mut conn, "Assets:NewBank")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            account_get_by_name(&mut conn, "Assets:OldBank")
                .await
                .unwrap()
                .is_none()
        );

        // 中文改名：该语言无显示名 → 新增显示名，英文名不受影响
        account_rename(&mut conn, id, "新银行", "zh-CN")
            .await
            .unwrap();
        let names = crate::names::ACCOUNT_NAMES
            .batch_resolve_display(&mut conn, &[id.0], "zh-CN")
            .await
            .unwrap();
        assert_eq!(names.get(&id.0).unwrap(), "新银行");
        assert!(
            account_get_by_name(&mut conn, "Assets:NewBank")
                .await
                .unwrap()
                .is_some()
        );

        // 改为父级命名空间内已占用的名字 → 拒绝
        assert!(account_rename(&mut conn, id, "cash", "en").await.is_err());

        // 系统账户改名：系统名降级为非显示（文本保留），用户名成为显示名
        account_rename(&mut conn, assets_id, "MyAssets", "en")
            .await
            .unwrap();
        let names = crate::names::ACCOUNT_NAMES
            .batch_resolve_display(&mut conn, &[assets_id.0], "en")
            .await
            .unwrap();
        assert_eq!(names.get(&assets_id.0).unwrap(), "MyAssets");
        // 系统名仍可命中
        assert!(
            account_get_by_name(&mut conn, "Assets")
                .await
                .unwrap()
                .is_some()
        );

        // 不存在的账户显式报错
        assert!(
            account_rename(&mut conn, AccountId(99999), "X", "en")
                .await
                .is_err()
        );
    }

    /// 查账户的完整祖先链（含自身 depth=0），按 depth 升序返回 (ancestor_id, depth)。
    async fn ancestor_chain(conn: &mut SqliteConnection, id: AccountId) -> Vec<(i64, i32)> {
        sqlx::query_as(
            "SELECT ancestor_id, depth FROM account_ancestors WHERE account_id = ?1 ORDER BY depth",
        )
        .bind(id.0)
        .fetch_all(conn)
        .await
        .unwrap()
    }

    /// 建链 A→B→C→D（A 为根），返回 (a, b, c, d)。
    async fn build_chain(
        conn: &mut SqliteConnection,
    ) -> (AccountId, AccountId, AccountId, AccountId) {
        let a = account_create_with_closure(conn, &bare_account(None))
            .await
            .unwrap();
        let b = account_create_with_closure(conn, &bare_account(Some(a)))
            .await
            .unwrap();
        let c = account_create_with_closure(conn, &bare_account(Some(b)))
            .await
            .unwrap();
        let d = account_create_with_closure(conn, &bare_account(Some(c)))
            .await
            .unwrap();
        (a, b, c, d)
    }

    #[tokio::test]
    async fn test_is_descendant_of() {
        let mut conn = setup().await;
        let assets_id = account_get_by_name(&mut conn, "Assets")
            .await
            .unwrap()
            .unwrap()
            .id;
        let cash_id = account_get_by_name(&mut conn, "Assets:Cash")
            .await
            .unwrap()
            .unwrap()
            .id;

        assert!(
            account_is_descendant_of(&mut conn, cash_id, assets_id)
                .await
                .unwrap()
        );
        assert!(
            !account_is_descendant_of(&mut conn, assets_id, cash_id)
                .await
                .unwrap()
        );
        // 自身不算严格后代
        assert!(
            !account_is_descendant_of(&mut conn, assets_id, assets_id)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_update_parent_rebuilds_subtree_closure() {
        let mut conn = setup().await;
        let (a, b, c, d) = build_chain(&mut conn).await;
        // 另一个独立的父节点 E（根）
        let e = account_create_with_closure(&mut conn, &bare_account(None))
            .await
            .unwrap();

        account_update_parent(&mut conn, b, e).await.unwrap();

        // B 的 parent_id 更新
        let moved = account_get(&mut conn, b).await.unwrap().unwrap();
        assert_eq!(moved.parent_id, Some(e));

        // B/C/D 的祖先链都换成新父链，不再含旧父 A
        assert_eq!(ancestor_chain(&mut conn, b).await, vec![(b.0, 0), (e.0, 1)]);
        assert_eq!(
            ancestor_chain(&mut conn, c).await,
            vec![(c.0, 0), (b.0, 1), (e.0, 2)]
        );
        assert_eq!(
            ancestor_chain(&mut conn, d).await,
            vec![(d.0, 0), (c.0, 1), (b.0, 2), (e.0, 3)]
        );
        // 旧父 A 不再是子树任何节点的祖先
        for node in [b, c, d] {
            assert!(
                !account_is_descendant_of(&mut conn, node, a).await.unwrap(),
                "{:?} 不应再以 A 为祖先",
                node
            );
        }
        // 根推导随新父链变化
        assert_eq!(account_find_root_id(&mut conn, d).await.unwrap(), e);
    }

    #[tokio::test]
    async fn test_update_parent_inside_transaction_rolls_back() {
        let mut conn = setup().await;
        let (a, b, _c, _d) = build_chain(&mut conn).await;
        let e = account_create_with_closure(&mut conn, &bare_account(None))
            .await
            .unwrap();

        {
            let mut tx = sqlx::Connection::begin(&mut conn).await.unwrap();
            account_update_parent(&mut tx, b, e).await.unwrap();
            // 事务内可见新父
            let moved = account_get(&mut tx, b).await.unwrap().unwrap();
            assert_eq!(moved.parent_id, Some(e));
            tx.rollback().await.unwrap();
        }

        // 回滚后 parent_id 与闭包都保持原样
        let restored = account_get(&mut conn, b).await.unwrap().unwrap();
        assert_eq!(restored.parent_id, Some(a));
        assert_eq!(ancestor_chain(&mut conn, b).await, vec![(b.0, 0), (a.0, 1)]);
    }

    #[tokio::test]
    async fn test_get_or_create_by_path_uses_given_lang() {
        let mut conn = setup().await;

        // 导入场景：自动创建的名字标 'und'
        let id = account_get_or_create_by_path(&mut conn, "Expenses:餐饮美食", "und")
            .await
            .unwrap();
        let lang: String = sqlx::query_scalar(
            "SELECT lang FROM account_names WHERE account_id = ?1 AND name = '餐饮美食'",
        )
        .bind(id.0)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(lang, "und");

        // 重复调用幂等，返回同一账户
        let again = account_get_or_create_by_path(&mut conn, "Expenses:餐饮美食", "und")
            .await
            .unwrap();
        assert_eq!(again, id);
    }
}
