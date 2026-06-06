use rusqlite::Connection;

use crate::error::DbError;

/// 初始化数据库表结构
pub fn initialize_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

/// 插入系统内置数据
pub fn insert_seed_data(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SEED_SQL)?;
    Ok(())
}

const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS commodities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    precision INTEGER NOT NULL DEFAULT 2
);

CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL UNIQUE,
    account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 5),
    parent_id INTEGER REFERENCES accounts(id),
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    is_system INTEGER NOT NULL DEFAULT 0,
    billing_day INTEGER CHECK(billing_day BETWEEN 1 AND 31),
    repayment_day INTEGER CHECK(repayment_day BETWEEN 1 AND 31)
);

CREATE TABLE IF NOT EXISTS account_ancestors (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    ancestor_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    depth INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, ancestor_id)
);

CREATE TABLE IF NOT EXISTS account_owners (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, member_id)
);

CREATE TABLE IF NOT EXISTS members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    member_id INTEGER REFERENCES members(id),
    is_template INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
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
    channel_id INTEGER REFERENCES channels(id)
);

CREATE TABLE IF NOT EXISTS attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    data BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_tags (
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
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
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_attachments_tx ON attachments(transaction_id);
CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag ON transaction_tags(tag_id);
"#;

const SEED_SQL: &str = r#"
INSERT OR IGNORE INTO commodities (symbol, name, precision) VALUES ('CNY', '人民币', 2);

INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, opened_at, is_system) VALUES
('Equity:OpeningBalances', 3, NULL, '2000-01-01', 1),
('Income:Uncategorized', 4, NULL, '2000-01-01', 1),
('Expenses:Uncategorized', 5, NULL, '2000-01-01', 1),
('Expenses:Fees', 5, NULL, '2000-01-01', 1),
('Expenses:Discounts', 5, NULL, '2000-01-01', 1),
('Expenses:InstallmentFees', 5, NULL, '2000-01-01', 1),
('Assets:Cashback', 1, NULL, '2000-01-01', 1);

INSERT OR IGNORE INTO tags (name, description, is_system) VALUES
('repayment', '分期/信用卡还款标记', 1);
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
    }

    #[test]
    fn test_seed_data() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        insert_seed_data(&conn).unwrap();

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
        assert_eq!(count, 7);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tags WHERE name='repayment'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
