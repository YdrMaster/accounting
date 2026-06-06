use rusqlite::Connection;

use crate::error::DbError;

/// 初始化数据库表结构
pub fn initialize_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

/// 插入系统内置数据，支持语言选择
pub fn insert_seed_data(conn: &Connection, lang: &str) -> Result<(), DbError> {
    let (accounts_root, accounts_child) = if lang.starts_with("zh") {
        (SEED_ACCOUNTS_ROOT_ZH, SEED_ACCOUNTS_CHILD_ZH)
    } else {
        (SEED_ACCOUNTS_ROOT_EN, SEED_ACCOUNTS_CHILD_EN)
    };
    let tags_sql = if lang.starts_with("zh") {
        SEED_TAGS_ZH
    } else {
        SEED_TAGS_EN
    };
    // 先插入根账户，再插入子账户（子查询依赖根账户已存在）
    conn.execute_batch(&format!(
        "{}{}{}{}{}",
        SEED_COMMODITIES, accounts_root, accounts_child, tags_sql, SEED_CLOSURE
    ))?;
    Ok(())
}

const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS commodities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    precision INTEGER NOT NULL DEFAULT 2,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL UNIQUE,
    account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 5),
    parent_id INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (date('now')),
    closed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (date('now')),
    is_system INTEGER NOT NULL DEFAULT 0,
    billing_day INTEGER CHECK(billing_day BETWEEN 1 AND 31),
    repayment_day INTEGER CHECK(repayment_day BETWEEN 1 AND 31)
);

CREATE TABLE IF NOT EXISTS account_ancestors (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    ancestor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    depth INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now')),
    PRIMARY KEY (account_id, ancestor_id)
);

CREATE TABLE IF NOT EXISTS account_owners (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now')),
    PRIMARY KEY (account_id, member_id)
);

CREATE TABLE IF NOT EXISTS members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date_time TEXT NOT NULL,
    description TEXT NOT NULL,
    member_id INTEGER REFERENCES members(id),
    is_template INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS postings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    commodity_id INTEGER NOT NULL REFERENCES commodities(id),
    amount INTEGER NOT NULL,
    cost INTEGER,
    cost_commodity_id INTEGER REFERENCES commodities(id),
    description TEXT,
    member_id INTEGER REFERENCES members(id),
    channel_id INTEGER REFERENCES channels(id),
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);

CREATE TABLE IF NOT EXISTS transaction_tags (
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now')),
    PRIMARY KEY (transaction_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_id);
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type);
CREATE INDEX IF NOT EXISTS idx_account_ancestors_ancestor ON account_ancestors(ancestor_id);
CREATE INDEX IF NOT EXISTS idx_account_ancestors_account ON account_ancestors(account_id);
CREATE INDEX IF NOT EXISTS idx_postings_tx ON postings(transaction_id);
CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);
CREATE INDEX IF NOT EXISTS idx_postings_commodity ON postings(commodity_id);
CREATE INDEX IF NOT EXISTS idx_postings_account_commodity ON postings(account_id, commodity_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date_time);
CREATE INDEX IF NOT EXISTS idx_attachments_tx ON attachments(transaction_id);
CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag ON transaction_tags(tag_id);

CREATE TRIGGER IF NOT EXISTS update_commodities_updated_at
AFTER UPDATE ON commodities
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE commodities SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_accounts_updated_at
AFTER UPDATE ON accounts
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE accounts SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_account_ancestors_updated_at
AFTER UPDATE ON account_ancestors
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE account_ancestors SET updated_at = date('now')
    WHERE account_id = NEW.account_id AND ancestor_id = NEW.ancestor_id;
END;

CREATE TRIGGER IF NOT EXISTS update_account_owners_updated_at
AFTER UPDATE ON account_owners
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE account_owners SET updated_at = date('now')
    WHERE account_id = NEW.account_id AND member_id = NEW.member_id;
END;

CREATE TRIGGER IF NOT EXISTS update_members_updated_at
AFTER UPDATE ON members
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE members SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_channels_updated_at
AFTER UPDATE ON channels
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE channels SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_tags_updated_at
AFTER UPDATE ON tags
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE tags SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_transactions_updated_at
AFTER UPDATE ON transactions
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE transactions SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_postings_updated_at
AFTER UPDATE ON postings
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE postings SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_attachments_updated_at
AFTER UPDATE ON attachments
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE attachments SET updated_at = date('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_transaction_tags_updated_at
AFTER UPDATE ON transaction_tags
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE transaction_tags SET updated_at = date('now')
    WHERE transaction_id = NEW.transaction_id AND tag_id = NEW.tag_id;
END;
"#;

const SEED_ACCOUNTS_ROOT_EN: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('Assets', 1, NULL, 1),
('Liabilities', 2, NULL, 1),
('Equity', 3, NULL, 1),
('Income', 4, NULL, 1),
('Expenses', 5, NULL, 1);
"#;

const SEED_ACCOUNTS_CHILD_EN: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('Equity:OpeningBalances', 3, (SELECT id FROM accounts WHERE full_name = 'Equity'), 1),
('Income:Uncategorized', 4, (SELECT id FROM accounts WHERE full_name = 'Income'), 1),
('Expenses:Uncategorized', 5, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Expenses:Fees', 5, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Expenses:Discounts', 5, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Expenses:InstallmentFees', 5, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Assets:Cashback', 1, (SELECT id FROM accounts WHERE full_name = 'Assets'), 1);
"#;

const SEED_ACCOUNTS_ROOT_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('资产', 1, NULL, 1),
('负债', 2, NULL, 1),
('权益', 3, NULL, 1),
('收入', 4, NULL, 1),
('支出', 5, NULL, 1);
"#;

const SEED_ACCOUNTS_CHILD_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('权益:期初余额', 3, (SELECT id FROM accounts WHERE full_name = '权益'), 1),
('收入:未分类', 4, (SELECT id FROM accounts WHERE full_name = '收入'), 1),
('支出:未分类', 5, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('支出:手续费', 5, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('支出:折扣', 5, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('支出:分期手续费', 5, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('资产:返现', 1, (SELECT id FROM accounts WHERE full_name = '资产'), 1);
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
('repayment', '分期/信用卡还款标记', 1);
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
    use rusqlite::Connection;

    #[test]
    fn test_schema_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert!(tables.contains(&"commodities".to_string()));
        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"account_ancestors".to_string()));
        assert!(tables.contains(&"account_owners".to_string()));
        assert!(tables.contains(&"members".to_string()));
        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"postings".to_string()));
        assert!(tables.contains(&"attachments".to_string()));
        assert!(tables.contains(&"transaction_tags".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_seed_data() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        insert_seed_data(&conn, "en").unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM commodities WHERE symbol='CNY'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE is_system=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 12);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name='repayment'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_audit_columns_exist() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        let tables = [
            "commodities",
            "accounts",
            "account_ancestors",
            "account_owners",
            "members",
            "channels",
            "tags",
            "transactions",
            "postings",
            "attachments",
            "transaction_tags",
        ];

        for table in tables {
            let cols: Vec<String> = conn
                .prepare(&format!("SELECT name FROM pragma_table_info('{}')", table))
                .unwrap()
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
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

    #[test]
    fn test_updated_at_trigger() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        insert_seed_data(&conn, "en").unwrap();

        // 手动将 updated_at 设为过去日期，以便触发器能体现出变化
        conn.execute(
            "UPDATE accounts SET updated_at = '2000-01-01' WHERE id = 1",
            [],
        )
        .unwrap();

        conn.execute(
            "UPDATE accounts SET full_name = full_name || 'X' WHERE id = 1",
            [],
        )
        .unwrap();

        let after: String = conn
            .query_row("SELECT updated_at FROM accounts WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_ne!(after, "2000-01-01", "updated_at 触发器未生效");
    }
}
