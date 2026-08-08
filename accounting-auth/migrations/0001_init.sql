-- 认证数据库初始 schema：users + sessions
-- 时间戳统一为 Unix 秒（INTEGER）

CREATE TABLE users (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    username       TEXT    NOT NULL UNIQUE,
    password_hash  TEXT    NOT NULL,
    display_name   TEXT    NOT NULL DEFAULT '',
    -- TOTP 密钥原始字节（自用单库明文存储；产品化时需加密，见 design.md D5）
    totp_secret    BLOB,
    totp_enabled   INTEGER NOT NULL DEFAULT 0,
    -- 恢复码 SHA-256 哈希的 JSON 数组，未开启 TOTP 时为 NULL
    recovery_codes TEXT,
    -- 上次成功验证的 TOTP 时间步（防重放，有意放在 users 而非 sessions）
    last_totp_step INTEGER,
    created_at     INTEGER NOT NULL
);

CREATE TABLE sessions (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    token_hash    TEXT    NOT NULL UNIQUE,
    user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- pending=1 表示等待 TOTP 第二步，不得通过业务 API 认证
    pending       INTEGER NOT NULL DEFAULT 0,
    -- pending session 上 TOTP 连续失败次数，达到 5 次作废
    totp_attempts INTEGER NOT NULL DEFAULT 0,
    expires_at    INTEGER NOT NULL,
    created_at    INTEGER NOT NULL
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
