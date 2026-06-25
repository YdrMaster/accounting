use sqlx::SqliteConnection;

use crate::error::DbError;

/// 初始化数据库表结构
pub async fn initialize_schema(conn: &mut SqliteConnection) -> Result<(), DbError> {
    for sql in SCHEMA_STATEMENTS {
        sqlx::query(*sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }
    Ok(())
}

/// 插入系统内置数据，支持语言选择
pub async fn insert_seed_data(conn: &mut SqliteConnection, lang: &str) -> Result<(), DbError> {
    let (accounts_root, accounts_child, tags_sql) = if lang.starts_with("zh") {
        (SEED_ACCOUNTS_ROOT_ZH, SEED_ACCOUNTS_CHILD_ZH, SEED_TAGS_ZH)
    } else {
        (SEED_ACCOUNTS_ROOT_EN, SEED_ACCOUNTS_CHILD_EN, SEED_TAGS_EN)
    };

    for sql in [
        SEED_COMMODITIES,
        accounts_root,
        accounts_child,
        tags_sql,
        SEED_CLOSURE,
    ] {
        sqlx::query(sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
    }

    Ok(())
}

const SCHEMA_STATEMENTS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS commodities (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        symbol TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        precision INTEGER NOT NULL DEFAULT 2,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        parent_id INTEGER REFERENCES accounts(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        closed_at TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        is_system INTEGER NOT NULL DEFAULT 0,
        billing_day INTEGER CHECK(billing_day BETWEEN 1 AND 31),
        repayment_day INTEGER CHECK(repayment_day BETWEEN 1 AND 31),
        UNIQUE(parent_id, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS account_ancestors (
        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        ancestor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        depth INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (account_id, ancestor_id)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS account_owners (
        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (account_id, member_id)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS members (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS channels (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        description TEXT,
        account_id INTEGER REFERENCES accounts(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        description TEXT,
        is_system INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS transactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date_time TEXT NOT NULL,
        description TEXT NOT NULL,
        member_id INTEGER REFERENCES members(id),
        kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS channel_paths (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        channel_id INTEGER NOT NULL REFERENCES channels(id),
        reconciled INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS postings (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
        account_id INTEGER NOT NULL REFERENCES accounts(id),
        commodity_id INTEGER NOT NULL REFERENCES commodities(id),
        amount INTEGER NOT NULL,
        cost INTEGER,
        cost_commodity_id INTEGER REFERENCES commodities(id),
        description TEXT,
        is_reimbursable INTEGER NOT NULL DEFAULT 0,
        linked_posting_id INTEGER REFERENCES postings(id),
        reversal_total INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS attachments (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
        filename TEXT NOT NULL,
        data BLOB NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS transaction_tags (
        transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
        tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (transaction_id, tag_id)
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_id);",
    "CREATE INDEX IF NOT EXISTS idx_account_ancestors_ancestor ON account_ancestors(ancestor_id);",
    "CREATE INDEX IF NOT EXISTS idx_account_ancestors_account ON account_ancestors(account_id);",
    "CREATE INDEX IF NOT EXISTS idx_postings_tx ON postings(transaction_id);",
    "CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);",
    "CREATE INDEX IF NOT EXISTS idx_postings_commodity ON postings(commodity_id);",
    "CREATE INDEX IF NOT EXISTS idx_postings_account_commodity ON postings(account_id, commodity_id);",
    "CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date_time);",
    "CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind);",
    "CREATE INDEX IF NOT EXISTS idx_channel_paths_transaction_id ON channel_paths(transaction_id);",
    "CREATE INDEX IF NOT EXISTS idx_channel_paths_channel_id ON channel_paths(channel_id);",
    "CREATE INDEX IF NOT EXISTS idx_postings_reimbursable ON postings(is_reimbursable);",
    "CREATE INDEX IF NOT EXISTS idx_postings_linked ON postings(linked_posting_id);",
    "CREATE INDEX IF NOT EXISTS idx_attachments_tx ON attachments(transaction_id);",
    "CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag ON transaction_tags(tag_id);",
    r#"
    CREATE TRIGGER IF NOT EXISTS update_commodities_updated_at
    AFTER UPDATE ON commodities
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE commodities SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_accounts_updated_at
    AFTER UPDATE ON accounts
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE accounts SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_account_ancestors_updated_at
    AFTER UPDATE ON account_ancestors
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE account_ancestors SET updated_at = datetime('now')
        WHERE account_id = NEW.account_id AND ancestor_id = NEW.ancestor_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_account_owners_updated_at
    AFTER UPDATE ON account_owners
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE account_owners SET updated_at = datetime('now')
        WHERE account_id = NEW.account_id AND member_id = NEW.member_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_members_updated_at
    AFTER UPDATE ON members
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE members SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_channels_updated_at
    AFTER UPDATE ON channels
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE channels SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_channel_paths_updated_at
    AFTER UPDATE ON channel_paths
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE channel_paths SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_tags_updated_at
    AFTER UPDATE ON tags
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE tags SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_transactions_updated_at
    AFTER UPDATE ON transactions
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE transactions SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_postings_updated_at
    AFTER UPDATE ON postings
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE postings SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_insert
    AFTER INSERT ON postings
    WHEN (SELECT kind FROM transactions WHERE id = NEW.transaction_id) IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
    BEGIN
        UPDATE postings
        SET reversal_total = reversal_total + NEW.amount
        WHERE id = NEW.linked_posting_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_delete
    AFTER DELETE ON postings
    WHEN (SELECT kind FROM transactions WHERE id = OLD.transaction_id) IN (2, 3) AND OLD.linked_posting_id IS NOT NULL
    BEGIN
        UPDATE postings
        SET reversal_total = reversal_total - OLD.amount
        WHERE id = OLD.linked_posting_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_update
    AFTER UPDATE OF amount ON postings
    WHEN (SELECT kind FROM transactions WHERE id = NEW.transaction_id) IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
    BEGIN
        UPDATE postings
        SET reversal_total = reversal_total - OLD.amount + NEW.amount
        WHERE id = NEW.linked_posting_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_attachments_updated_at
    AFTER UPDATE ON attachments
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE attachments SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_transaction_tags_updated_at
    AFTER UPDATE ON transaction_tags
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE transaction_tags SET updated_at = datetime('now')
        WHERE transaction_id = NEW.transaction_id AND tag_id = NEW.tag_id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_settings_updated_at
    AFTER UPDATE ON settings
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE settings SET updated_at = datetime('now') WHERE key = NEW.key;
    END;
    "#,
];

const SEED_ACCOUNTS_ROOT_EN: &str = r#"
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('Assets', NULL, 1),
('Equity', NULL, 1),
('Income', NULL, 1),
('Expenses', NULL, 1);
"#;

const SEED_ACCOUNTS_CHILD_EN: &str = r#"
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('OpeningBalances', (SELECT id FROM accounts WHERE name = 'Equity' AND parent_id IS NULL), 1),
('Fees', (SELECT id FROM accounts WHERE name = 'Expenses' AND parent_id IS NULL), 1),
('Discounts', (SELECT id FROM accounts WHERE name = 'Expenses' AND parent_id IS NULL), 1),
('InstallmentFees', (SELECT id FROM accounts WHERE name = 'Expenses' AND parent_id IS NULL), 1),
('Cash', (SELECT id FROM accounts WHERE name = 'Assets' AND parent_id IS NULL), 1),
('Cashback', (SELECT id FROM accounts WHERE name = 'Assets' AND parent_id IS NULL), 1);
"#;

const SEED_ACCOUNTS_ROOT_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('资产', NULL, 1),
('权益', NULL, 1),
('收入', NULL, 1),
('支出', NULL, 1);
"#;

const SEED_ACCOUNTS_CHILD_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (name, parent_id, is_system) VALUES
('期初余额', (SELECT id FROM accounts WHERE name = '权益' AND parent_id IS NULL), 1),
('手续费', (SELECT id FROM accounts WHERE name = '支出' AND parent_id IS NULL), 1),
('折扣', (SELECT id FROM accounts WHERE name = '支出' AND parent_id IS NULL), 1),
('分期手续费', (SELECT id FROM accounts WHERE name = '支出' AND parent_id IS NULL), 1),
('现金', (SELECT id FROM accounts WHERE name = '资产' AND parent_id IS NULL), 1),
('返现', (SELECT id FROM accounts WHERE name = '资产' AND parent_id IS NULL), 1);
"#;

const SEED_COMMODITIES: &str = r#"
INSERT OR IGNORE INTO commodities (symbol, name, precision) VALUES ('CNY', '人民币', 2);
"#;

const SEED_TAGS_EN: &str = r#"
INSERT OR IGNORE INTO tags (name, description, is_system) VALUES
('repayment', 'Installment or credit card repayment marker', 1);
"#;

const SEED_TAGS_ZH: &str = r#"
INSERT OR IGNORE INTO tags (name, description, is_system) VALUES
('还款', '分期/信用卡还款标记', 1);
"#;

/// 维护系统内置账户的闭包表
const SEED_CLOSURE: &str = r#"
-- 1. 每个账户的自身关系 (depth = 0)
INSERT OR IGNORE INTO account_ancestors (account_id, ancestor_id, depth)
SELECT id, id, 0 FROM accounts WHERE is_system = 1;

-- 2. 递归维护祖先关系 (depth >= 1)
WITH RECURSIVE ancestors AS (
    SELECT id, parent_id, 1 AS depth
    FROM accounts
    WHERE is_system = 1 AND parent_id IS NOT NULL
    UNION ALL
    SELECT a.id, p.parent_id, a.depth + 1
    FROM ancestors a
    JOIN accounts p ON p.id = a.parent_id
    WHERE p.parent_id IS NOT NULL
)
INSERT OR IGNORE INTO account_ancestors (account_id, ancestor_id, depth)
SELECT id, parent_id, depth FROM ancestors;
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        initialize_schema(&mut conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        let mut conn = setup().await;

        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch_all(&mut conn)
                .await
                .unwrap();

        assert!(tables.contains(&"commodities".to_string()));
        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"account_ancestors".to_string()));
        assert!(tables.contains(&"account_owners".to_string()));
        assert!(tables.contains(&"members".to_string()));
        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"channel_paths".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"postings".to_string()));
        assert!(tables.contains(&"attachments".to_string()));
        assert!(tables.contains(&"transaction_tags".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[tokio::test]
    async fn test_seed_data() {
        let mut conn = setup().await;
        insert_seed_data(&mut conn, "en").await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM commodities WHERE symbol='CNY'")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE is_system=1")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 10);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE name='repayment'")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_audit_columns_exist() {
        let mut conn = setup().await;

        let tables = [
            "commodities",
            "accounts",
            "account_ancestors",
            "account_owners",
            "members",
            "channels",
            "channel_paths",
            "tags",
            "transactions",
            "postings",
            "attachments",
            "transaction_tags",
            "settings",
        ];

        for table in tables {
            let cols: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info(?)")
                .bind(table)
                .fetch_all(&mut conn)
                .await
                .unwrap();
            assert!(
                cols.contains(&"created_at".to_string()),
                "{} 缺少 created_at",
                table
            );
            assert!(
                cols.contains(&"updated_at".to_string()),
                "{} 缺少 updated_at",
                table
            );
        }
    }

    #[tokio::test]
    async fn test_updated_at_trigger() {
        let mut conn = setup().await;
        insert_seed_data(&mut conn, "en").await.unwrap();

        // 手动将 updated_at 设为过去日期，以便触发器能体现出变化
        sqlx::query("UPDATE accounts SET updated_at = '2000-01-01' WHERE id = 1")
            .execute(&mut conn)
            .await
            .unwrap();

        sqlx::query("UPDATE accounts SET name = name || 'X' WHERE id = 1")
            .execute(&mut conn)
            .await
            .unwrap();

        let after: String = sqlx::query_scalar("SELECT updated_at FROM accounts WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_ne!(after, "2000-01-01", "updated_at 触发器未生效");
    }
}
