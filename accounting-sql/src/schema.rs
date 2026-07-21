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
    create_unique_indexes(conn).await?;
    Ok(())
}

/// 为自然键创建唯一索引。
///
/// 名字的唯一性由 names 表的 UNIQUE(entity_id, lang, name) 约束保证，
/// 此处仅保留对 account_ancestors 等辅助表的索引。
async fn create_unique_indexes(_conn: &mut SqliteConnection) -> Result<(), DbError> {
    // No additional unique indexes needed - names tables have their own UNIQUE constraints
    Ok(())
}

/// 插入系统内置数据（无语言参数，语言不再是库的属性）。
///
/// 幂等：重复执行时按系统英文名找到已有实体，不产生重复实体、不报
/// RowNotFound；名字与闭包表依赖各自的 UNIQUE / PRIMARY KEY 约束以
/// INSERT OR IGNORE 去重。
pub async fn insert_seed_data(conn: &mut SqliteConnection) -> Result<(), DbError> {
    // === 1. Commodity ===
    let cny_id = seeded_commodity_id(conn, "CNY", 2).await?;

    // Commodity names
    sqlx::query(
        "INSERT OR IGNORE INTO commodity_names (commodity_id, lang, name, is_system, is_display)
         VALUES (?1, 'en', 'Chinese Yuan', 1, 1), (?1, 'zh-CN', '人民币', 1, 1)",
    )
    .bind(cny_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    // === 2. Root accounts ===
    let assets_id = seeded_account_id(conn, "Assets", None).await?;
    let equity_id = seeded_account_id(conn, "Equity", None).await?;
    let income_id = seeded_account_id(conn, "Income", None).await?;
    let expenses_id = seeded_account_id(conn, "Expenses", None).await?;

    // Root account names
    for (id, en_name, zh_name) in [
        (assets_id, "Assets", "资产"),
        (equity_id, "Equity", "权益"),
        (income_id, "Income", "收入"),
        (expenses_id, "Expenses", "支出"),
    ] {
        sqlx::query(
            "INSERT OR IGNORE INTO account_names (account_id, lang, name, is_system, is_display)
             VALUES (?1, 'en', ?2, 1, 1), (?1, 'zh-CN', ?3, 1, 1)",
        )
        .bind(id)
        .bind(en_name)
        .bind(zh_name)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    // === 3. Child accounts ===
    let child_specs: &[(&str, &str, i64)] = &[
        ("OpeningBalances", "期初余额", equity_id),
        ("Fees", "手续费", expenses_id),
        ("InstallmentFees", "分期手续费", expenses_id),
        ("Cash", "现金", assets_id),
        ("Discounts", "折扣", equity_id),
        ("Cashback", "返现", equity_id),
    ];

    let mut child_ids = Vec::new();
    for (en_name, zh_name, parent_id) in child_specs {
        let child_id = seeded_account_id(conn, en_name, Some(*parent_id)).await?;
        child_ids.push((child_id, *parent_id));

        sqlx::query(
            "INSERT OR IGNORE INTO account_names (account_id, lang, name, is_system, is_display)
             VALUES (?1, 'en', ?2, 1, 1), (?1, 'zh-CN', ?3, 1, 1)",
        )
        .bind(child_id)
        .bind(en_name)
        .bind(zh_name)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    // === 4. Tags ===
    let tag_specs: &[(&str, &str, &str)] = &[
        (
            "repayment",
            "还款",
            "Installment or credit card repayment marker",
        ),
        ("pending", "待处理", "Imported transaction pending review"),
        (
            "exclude-from-income-statement",
            "不计收支",
            "Excluded from income/expense statistics",
        ),
        (
            "exclude-from-budget",
            "不计预算",
            "Excluded from budget statistics",
        ),
    ];

    for (en_name, zh_name, en_desc) in tag_specs {
        let tag_id = seeded_tag_id(conn, en_name, en_desc).await?;

        sqlx::query(
            "INSERT OR IGNORE INTO tag_names (tag_id, lang, name, is_system, is_display)
             VALUES (?1, 'en', ?2, 1, 1), (?1, 'zh-CN', ?3, 1, 1)",
        )
        .bind(tag_id)
        .bind(en_name)
        .bind(zh_name)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    // === 5. Channel ===
    let channel_id = seeded_channel_id(conn, "Alipay").await?;

    sqlx::query(
        "INSERT OR IGNORE INTO channel_names (channel_id, lang, name, is_system, is_display)
         VALUES (?1, 'en', 'Alipay', 1, 1), (?1, 'zh-CN', '支付宝', 1, 1)",
    )
    .bind(channel_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;

    // === 6. Closure table for system accounts ===
    // Self-references (depth = 0)
    for (id, _parent_id) in &child_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO account_ancestors (account_id, ancestor_id, depth)
             VALUES (?1, ?1, 0)",
        )
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }
    for root_id in &[assets_id, equity_id, income_id, expenses_id] {
        sqlx::query(
            "INSERT OR IGNORE INTO account_ancestors (account_id, ancestor_id, depth)
             VALUES (?1, ?1, 0)",
        )
        .bind(root_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    // Parent references (depth = 1) for child accounts
    for (child_id, parent_id) in &child_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO account_ancestors (account_id, ancestor_id, depth)
             VALUES (?1, ?2, 1)",
        )
        .bind(child_id)
        .bind(parent_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    }

    Ok(())
}

/// 按系统英文名查找已 seed 的账户；不存在则插入。保证 insert_seed_data 幂等。
async fn seeded_account_id(
    conn: &mut SqliteConnection,
    en_name: &str,
    parent_id: Option<i64>,
) -> Result<i64, DbError> {
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT a.id FROM accounts a
         JOIN account_names an ON an.account_id = a.id
         WHERE an.is_system = 1 AND an.lang = 'en' AND an.name = ?1 AND a.parent_id IS ?2",
    )
    .bind(en_name)
    .bind(parent_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    if let Some(id) = existing {
        return Ok(id);
    }
    sqlx::query_scalar("INSERT INTO accounts (parent_id, is_system) VALUES (?1, 1) RETURNING id")
        .bind(parent_id)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))
}

/// 按 symbol 查找已 seed 的币种；不存在则插入。
async fn seeded_commodity_id(
    conn: &mut SqliteConnection,
    symbol: &str,
    precision: i32,
) -> Result<i64, DbError> {
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM commodities WHERE symbol = ?1")
        .bind(symbol)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))?;
    if let Some(id) = existing {
        return Ok(id);
    }
    sqlx::query_scalar("INSERT INTO commodities (symbol, precision) VALUES (?1, ?2) RETURNING id")
        .bind(symbol)
        .bind(precision)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))
}

/// 按系统英文名查找已 seed 的标签；不存在则插入。
async fn seeded_tag_id(
    conn: &mut SqliteConnection,
    en_name: &str,
    en_desc: &str,
) -> Result<i64, DbError> {
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT tag_id FROM tag_names WHERE is_system = 1 AND lang = 'en' AND name = ?1",
    )
    .bind(en_name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    if let Some(id) = existing {
        return Ok(id);
    }
    sqlx::query_scalar("INSERT INTO tags (description, is_system) VALUES (?1, 1) RETURNING id")
        .bind(en_desc)
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))
}

/// 按系统英文名查找已 seed 的渠道；不存在则插入。
async fn seeded_channel_id(conn: &mut SqliteConnection, en_name: &str) -> Result<i64, DbError> {
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT channel_id FROM channel_names WHERE is_system = 1 AND lang = 'en' AND name = ?1",
    )
    .bind(en_name)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| DbError::Database(e.to_string()))?;
    if let Some(id) = existing {
        return Ok(id);
    }
    sqlx::query_scalar("INSERT INTO channels (is_system) VALUES (1) RETURNING id")
        .fetch_one(conn)
        .await
        .map_err(|e| DbError::Database(e.to_string()))
}

const SCHEMA_STATEMENTS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS commodities (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        symbol TEXT NOT NULL UNIQUE,
        precision INTEGER NOT NULL DEFAULT 2,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        parent_id INTEGER REFERENCES accounts(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        closed_at TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        is_system INTEGER NOT NULL DEFAULT 0,
        billing_day INTEGER CHECK(billing_day BETWEEN 1 AND 31),
        repayment_day INTEGER CHECK(repayment_day BETWEEN 1 AND 31)
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
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS channels (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        description TEXT,
        account_id INTEGER REFERENCES accounts(id),
        is_system INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
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
        member_id INTEGER NOT NULL REFERENCES members(id),
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
        status INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS account_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(account_id, lang, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS tag_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(tag_id, lang, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS channel_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        channel_id INTEGER NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(channel_id, lang, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS commodity_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        commodity_id INTEGER NOT NULL REFERENCES commodities(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(commodity_id, lang, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS member_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(member_id, lang, name)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS budget_names (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
        lang TEXT NOT NULL,
        name TEXT NOT NULL COLLATE NOCASE,
        is_system INTEGER NOT NULL DEFAULT 0,
        is_display INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(budget_id, lang, name)
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
    "CREATE INDEX IF NOT EXISTS idx_account_names_account ON account_names(account_id);",
    "CREATE INDEX IF NOT EXISTS idx_tag_names_tag ON tag_names(tag_id);",
    "CREATE INDEX IF NOT EXISTS idx_channel_names_channel ON channel_names(channel_id);",
    "CREATE INDEX IF NOT EXISTS idx_commodity_names_commodity ON commodity_names(commodity_id);",
    "CREATE INDEX IF NOT EXISTS idx_member_names_member ON member_names(member_id);",
    "CREATE INDEX IF NOT EXISTS idx_budget_names_budget ON budget_names(budget_id);",
    // 每个 (entity, lang) 至多一条显示名
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_account_names_display ON account_names(account_id, lang) WHERE is_display = 1;",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_tag_names_display ON tag_names(tag_id, lang) WHERE is_display = 1;",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_channel_names_display ON channel_names(channel_id, lang) WHERE is_display = 1;",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_commodity_names_display ON commodity_names(commodity_id, lang) WHERE is_display = 1;",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_member_names_display ON member_names(member_id, lang) WHERE is_display = 1;",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_budget_names_display ON budget_names(budget_id, lang) WHERE is_display = 1;",
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
    r#"
    CREATE TRIGGER IF NOT EXISTS update_account_names_updated_at
    AFTER UPDATE ON account_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE account_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_tag_names_updated_at
    AFTER UPDATE ON tag_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE tag_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_channel_names_updated_at
    AFTER UPDATE ON channel_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE channel_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_commodity_names_updated_at
    AFTER UPDATE ON commodity_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE commodity_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_member_names_updated_at
    AFTER UPDATE ON member_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE member_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_budget_names_updated_at
    AFTER UPDATE ON budget_names
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE budget_names SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS budgets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        period INTEGER NOT NULL,
        commodity_id INTEGER NOT NULL REFERENCES commodities(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS budget_limits (
        budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
        account_id INTEGER NOT NULL REFERENCES accounts(id),
        amount INTEGER NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (budget_id, account_id)
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_budget_limits_account ON budget_limits(account_id);",
    r#"
    CREATE TRIGGER IF NOT EXISTS update_budgets_updated_at
    AFTER UPDATE ON budgets
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE budgets SET updated_at = datetime('now') WHERE id = NEW.id;
    END;
    "#,
    r#"
    CREATE TRIGGER IF NOT EXISTS update_budget_limits_updated_at
    AFTER UPDATE ON budget_limits
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE budget_limits SET updated_at = datetime('now')
        WHERE budget_id = NEW.budget_id AND account_id = NEW.account_id;
    END;
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS account_mappings (
        member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
        channel_id INTEGER NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
        category TEXT NOT NULL,
        account_id INTEGER NOT NULL REFERENCES accounts(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (member_id, channel_id, category)
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_account_mappings_account_id ON account_mappings(account_id);",
    r#"
    CREATE TRIGGER IF NOT EXISTS update_account_mappings_updated_at
    AFTER UPDATE ON account_mappings
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at
    BEGIN
        UPDATE account_mappings SET updated_at = datetime('now')
        WHERE member_id = NEW.member_id AND channel_id = NEW.channel_id AND category = NEW.category;
    END;
    "#,
];

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

    async fn setup_with_seed() -> SqliteConnection {
        let mut conn = setup().await;
        insert_seed_data(&mut conn).await.unwrap();
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
        assert!(tables.contains(&"account_names".to_string()));
        assert!(tables.contains(&"tag_names".to_string()));
        assert!(tables.contains(&"channel_names".to_string()));
        assert!(tables.contains(&"commodity_names".to_string()));
        assert!(tables.contains(&"member_names".to_string()));
        assert!(tables.contains(&"budget_names".to_string()));
    }

    #[tokio::test]
    async fn test_seed_data() {
        let mut conn = setup_with_seed().await;

        // Commodity
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM commodities WHERE symbol='CNY'")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 1);

        // System accounts (4 roots + 6 children = 10)
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE is_system=1")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 10);

        // English names in account_names and tag_names
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tag_names WHERE name='repayment' AND lang='en' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tag_names WHERE name='pending' AND lang='en' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Chinese names in account_names
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_names WHERE name='资产' AND lang='zh-CN' AND is_system=1 AND is_display=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // English names in account_names
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM account_names WHERE name='Expenses' AND lang='en' AND is_system=1 AND is_display=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Channel names
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM channel_names WHERE name='Alipay' AND lang='en' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM channel_names WHERE name='支付宝' AND lang='zh-CN' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Commodity names
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commodity_names WHERE name='Chinese Yuan' AND lang='en' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commodity_names WHERE name='人民币' AND lang='zh-CN' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Closure table entries: 10 self-refs + 6 parent-refs = 16
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_ancestors")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count, 16);

        // Cashback 与 Discounts 挂在 Equity 根下（返现/折扣是权益调整项，非资产/支出）
        let equity_id: i64 = sqlx::query_scalar(
            "SELECT account_id FROM account_names WHERE name='Equity' AND lang='en' AND is_system=1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        for leaf in ["Cashback", "Discounts"] {
            let parent_id: Option<i64> = sqlx::query_scalar(
                "SELECT a.parent_id FROM accounts a
                 JOIN account_names an ON an.account_id = a.id
                 WHERE an.name=?1 AND an.lang='en' AND an.is_system=1",
            )
            .bind(leaf)
            .fetch_one(&mut conn)
            .await
            .unwrap();
            assert_eq!(parent_id, Some(equity_id), "{} 应挂在 Equity 根下", leaf);
        }
    }

    #[tokio::test]
    async fn test_seed_data_idempotent() {
        let mut conn = setup().await;
        insert_seed_data(&mut conn).await.unwrap();
        // 重复执行：不产生重复实体、不报 RowNotFound
        insert_seed_data(&mut conn).await.unwrap();

        let checks: &[(&str, i64)] = &[
            ("SELECT COUNT(*) FROM accounts WHERE is_system=1", 10),
            ("SELECT COUNT(*) FROM account_ancestors", 16),
            ("SELECT COUNT(*) FROM account_names", 20),
            ("SELECT COUNT(*) FROM tag_names", 8),
            ("SELECT COUNT(*) FROM channel_names", 2),
            ("SELECT COUNT(*) FROM commodity_names", 2),
            ("SELECT COUNT(*) FROM commodities WHERE symbol='CNY'", 1),
        ];
        for (sql, expected) in checks {
            let count: i64 = sqlx::query_scalar(*sql).fetch_one(&mut conn).await.unwrap();
            assert_eq!(&count, expected, "重复 seed 后计数异常: {}", sql);
        }
    }

    #[tokio::test]
    async fn test_display_name_unique_per_entity_lang() {
        let mut conn = setup_with_seed().await;

        // 同一 (account_id, lang) 插入第二条 is_display=1 → 唯一索引拒绝
        let result = sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display)
             SELECT account_id, lang, name || '2', 0, 1 FROM account_names
             WHERE lang = 'en' AND is_display = 1 LIMIT 1",
        )
        .execute(&mut conn)
        .await;
        assert!(result.is_err(), "同语言第二条显示名应被唯一索引拒绝");

        // 同语言 is_display=0 的别名允许存在
        let inserted = sqlx::query(
            "INSERT INTO account_names (account_id, lang, name, is_system, is_display)
             SELECT account_id, lang, name || '2', 0, 0 FROM account_names
             WHERE lang = 'en' AND is_display = 1 LIMIT 1",
        )
        .execute(&mut conn)
        .await;
        assert!(inserted.is_ok(), "同语言非显示名别名应允许");
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
            "account_names",
            "tag_names",
            "channel_names",
            "commodity_names",
            "member_names",
            "budget_names",
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
        let mut conn = setup_with_seed().await;

        // 手动将 updated_at 设为过去日期，以便触发器能体现出变化
        sqlx::query("UPDATE accounts SET updated_at = '2000-01-01' WHERE id = 1")
            .execute(&mut conn)
            .await
            .unwrap();

        sqlx::query("UPDATE account_names SET name = name || 'X' WHERE id = 1")
            .execute(&mut conn)
            .await
            .unwrap();

        let after: String = sqlx::query_scalar("SELECT updated_at FROM account_names WHERE id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();

        assert_ne!(after, "2000-01-01", "updated_at 触发器未生效");
    }
}
