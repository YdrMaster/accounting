# 审计字段改进

## 现状

所有表（除 `settings` 外）都已有 `created_at` 和 `updated_at` 字段，并由触发器自动维护 `updated_at`。但存在两个问题：

### 1. 精度不足：`date('now')` 只到天

当前所有审计字段使用 `date('now')`，返回值如 `2026-06-23`，没有时分秒。

```sql
created_at TEXT NOT NULL DEFAULT (date('now')),
updated_at TEXT NOT NULL DEFAULT (date('now'))
```

问题：
- 同一天内创建的多条记录 `created_at` 完全相同，无法区分先后
- 一天内多次修改同一条记录，`updated_at` 看不出变化
- 审计字段的核心价值是精确记录变更时间，天级精度不够

### 2. `settings` 表缺失审计字段

`settings` 表是唯一没有 `created_at` / `updated_at` 的表：

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

虽然数据量极小，但为一致性应补上。

## 改进方案

### 审计字段精度改为秒级

将 `date('now')` 改为 `datetime('now')`，返回值如 `2026-06-23 10:30:45`。

**涉及位置：**

| 类型 | 文件 | 内容 |
|------|------|------|
| Schema DDL | `accounting-sql/src/schema.rs` | 所有表的 `created_at` / `updated_at` DEFAULT 值 |
| 触发器 | `accounting-sql/src/schema.rs` | 所有 `update_*_updated_at` 触发器中的 `date('now')` |
| 测试 | `accounting-sql/src/schema.rs` | `test_updated_at_trigger` 中的断言 |

**注意**：`transactions.date_time` 是用户记录的交易时间，保持 `TEXT` 格式不变（它已经存储了完整时间戳），与审计字段无关。

### `settings` 表补充审计字段

```sql
-- 改前
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 改后
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS update_settings_updated_at
AFTER UPDATE ON settings
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE key = NEW.key;
END;
```

## 影响范围

- `accounting-sql/src/schema.rs` — Schema DDL 和触发器
- 不影响 Rust 模型层（`created_at` / `updated_at` 当前未在 Rust struct 中映射，仅存于数据库）
- 不影响 API/CLI 层（审计字段不暴露给用户）

## 数据库迁移

```sql
-- 审计字段精度升级（需要对每张表执行）
ALTER TABLE commodities ALTER COLUMN created_at SET DEFAULT (datetime('now'));
ALTER TABLE commodities ALTER COLUMN updated_at SET DEFAULT (datetime('now'));
-- ... 对所有表重复
-- 注意：SQLite 的 ALTER TABLE 功能有限，可能需要重建表

-- settings 表补充审计字段
ALTER TABLE settings ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE settings ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
```

> **注意**：SQLite 不支持 `ALTER COLUMN`，迁移时可能需要采用"创建新表 → 复制数据 → 删除旧表 → 重命名"的模式。对于已有的 `initialize_schema` 机制，更简单的做法是直接修改 DDL，对新数据库生效，已有数据库通过版本迁移脚本处理。
