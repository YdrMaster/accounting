# 数据模型精简重构设计

## 背景

提交 `81b307bf`（`docs: 审查代码，提出修改意见`）在 `plan/` 目录下新增了 6 份重构/分析计划：

- `account-name-refactor.md`
- `account-type-refactor.md`
- `audit-fields-improvement.md`
- `multi-tenant-analysis.md`
- `remove-redundant-fields.md`
- `sql-dual-backend.md`

本次任务只落地其中 4 份**数据模型精简重构**计划，不涉及多租户分析与 PostgreSQL 双后端改造。

## 范围

本次实现包含：

1. **前置迁移**：将 `main` 分支的 `fcb8fea64c5dc25b6228b1866beb5c1826d902e2` cherry-pick 到当前 `refactor` 分支。
2. **阶段 1**：移除冗余字段（`remove-redundant-fields.md`）。
3. **阶段 2**：审计字段改进（`audit-fields-improvement.md`）。
4. **阶段 3**：账户名称重构（`account-name-refactor.md`）。
5. **阶段 4**：账户类型重构（`account-type-refactor.md`）。

本次**不实现**：

- `multi-tenant-analysis.md` 中的 PostgreSQL schema-per-tenant 方案。
- `sql-dual-backend.md` 中的 Repo trait async 化、PostgreSQL 支持等改造。

## 前置迁移：`fcb8fea6`

### 操作

在当前 `refactor` 分支上执行：

```bash
git cherry-pick fcb8fea64c5dc25b6228b1866beb5c1826d902e2
```

### 内容

该提交移除了 `Account` 的 `position` 字段以及前端 `AccountTreeList` 的拖拽排序功能，涉及：

- `accounting/src/account.rs`
- `accounting-sql/src/schema.rs`
- `accounting-sql/src/repo/account.rs`
- `accounting-sql/src/impls/sqlite.rs`
- `accounting-service/src/account_service.rs`
- `accounting-service/src/report_service.rs`
- `accounting-service/src/transaction_service.rs`
- `accounting-api/src/dto.rs`
- `accounting-api/src/handlers/account.rs`
- `accounting-web/` 多个组件

### 冲突预期

当前 `refactor` 分支已有一个额外 commit `4b9fa057`（移除 `Liability` 账户类型），而 `fcb8fea6` 也基于一个同样移除 `Liability` 的 `main` 提交 `99cc834d`。两边在 `account.rs`、`schema.rs`、`dto.rs` 等文件上可能产生删除冲突。

### 解决策略

以 `fcb8fea6` 的删除意图为准，保留当前分支已移除的 `Liability` 相关改动，确保最终状态等价于：

- `Account.position` 已删除
- 拖拽排序已删除
- `Liability` 账户类型已删除

### 出口条件

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后，保留 cherry-pick 结果并进入阶段 1。

## 阶段 1：移除冗余字段

### 目标

根据 `remove-redundant-fields.md`，从核心数据模型中移除以下冗余字段/方法：

1. `posting.member_id`
2. `posting.channel_id`
3. `transaction.is_template`
4. `AccountType::is_permanent()`

### 变更详情

#### 1.1 `posting.member_id` 移除

- `accounting/src/posting.rs`：删除 `member_id` 字段。
- `accounting-sql/src/schema.rs`：`postings` 表删除 `member_id` 列。
- `accounting-sql/src/repo/posting.rs`：INSERT/SELECT 语句移除 `member_id`。
- `accounting-service/src/transaction_service.rs`：构造 `Posting` 时不再设置 `member_id`。
- `accounting-api/src/dto.rs`：`Posting` DTO 移除 `member_id`。
- `accounting-api/src/handlers/transaction.rs`：不再从请求中读取 posting 的 `member_id`。
- `accounting-cli/src/cmd/tx.rs`：不再从命令行读取 posting 的 `member_id`。

> 注：`sum_by_member()` 统计查询已经使用 `t.member_id`（transaction 级别），不受影响。

#### 1.2 `posting.channel_id` 移除

- `accounting/src/posting.rs`：删除 `channel_id` 字段。
- `accounting-sql/src/schema.rs`：`postings` 表删除 `channel_id` 列。
- `accounting-sql/src/repo/posting.rs`：INSERT/SELECT 语句移除 `channel_id`；`sum_by_channel()` 需从 `p.channel_id` 改为 `t.channel_id`，并确保 JOIN 了 `transactions` 表。
- `accounting-service/src/transaction_service.rs`：构造 `Posting` 时不再设置 `channel_id`。
- `accounting-api/src/dto.rs`：`Posting` DTO 移除 `channel_id`。
- `accounting-api/src/handlers/transaction.rs`：不再从请求中读取 posting 的 `channel_id`。
- `accounting-cli/src/cmd/tx.rs`：不再从命令行读取 posting 的 `channel_id`。

#### 1.3 `transaction.is_template` 移除

- `accounting/src/transaction.rs`：删除 `is_template` 字段。
- `accounting/src/transaction_filter.rs`：删除 `is_template` 过滤字段。
- `accounting-sql/src/schema.rs`：`transactions` 表删除 `is_template` 列。
- `accounting-sql/src/repo/transaction.rs`：INSERT/SELECT/UPDATE 移除 `is_template`。
- `accounting-sql/src/repo/posting.rs`：统计查询移除 `is_template` 过滤条件。
- `accounting-service/src/transaction_service.rs`：构造 `Transaction` 时不再设置 `is_template`。
- `accounting-cli/src/cmd/tx.rs`：移除 `--template` 参数。
- `accounting-cli/src/cmd/mod.rs`：移除显示字段。
- `accounting-api/src/dto.rs`：移除 DTO 字段。
- `accounting-api/src/handlers/transaction.rs`：移除 `is_template` 处理。
- `accounting-web/src/stores/transaction.ts`：移除 `is_template` 字段。

#### 1.4 `AccountType::is_permanent()` 移除

- `accounting/src/account_type.rs`：删除 `is_permanent()` 方法及对应测试。

### 数据库迁移

```sql
ALTER TABLE postings DROP COLUMN member_id;
ALTER TABLE postings DROP COLUMN channel_id;
ALTER TABLE transactions DROP COLUMN is_template;
```

> SQLite 从 3.35.0 开始支持 `DROP COLUMN`，当前 rusqlite bundled 版本应已支持。

### 出口条件

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后提交。

## 阶段 2：审计字段改进

### 目标

根据 `audit-fields-improvement.md`：

1. 将审计字段精度从天级 `date('now')` 提升到秒级 `datetime('now')`。
2. 给 `settings` 表补充 `created_at` / `updated_at` 字段和触发器。

### 变更详情

#### 2.1 精度升级

修改 `accounting-sql/src/schema.rs`：

- 所有表的 `created_at TEXT NOT NULL DEFAULT (date('now'))` 改为 `datetime('now')`。
- 所有表的 `updated_at TEXT NOT NULL DEFAULT (date('now'))` 改为 `datetime('now')`。
- 所有 `update_*_updated_at` 触发器中的 `date('now')` 同步改为 `datetime('now')`。

涉及表：

- `commodities`
- `members`
- `channels`
- `tags`
- `accounts`
- `account_ancestors`
- `account_owners`
- `transactions`
- `postings`
- `transaction_tags`
- `attachments`
- `settings`（新增）

> 注：`transactions.date_time` 是用户记录的交易时间，保持 `TEXT` 格式不变，与审计字段无关。

#### 2.2 `settings` 表补字段

改前：

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

改后：

```sql
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

#### 2.3 测试调整

如果 `schema.rs` 中的 `test_updated_at_trigger` 断言基于 `date('now')`，同步更新为 `datetime('now')` 格式。

### 影响范围

- 仅 `accounting-sql/src/schema.rs`。
- 不影响 Rust 模型层（审计字段当前未在 struct 中映射）。
- 不影响 API/CLI 层（审计字段不暴露给用户）。

### 数据库迁移

```sql
-- settings 表补充审计字段
ALTER TABLE settings ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE settings ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
```

> 注：已有数据库的审计字段默认值升级需要通过版本迁移脚本处理；本次直接修改 DDL，对新数据库生效。

### 出口条件

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后提交。

## 阶段 3：账户名称重构

### 目标

根据 `account-name-refactor.md`，将 `Account` 的 `full_name` 字段改为 `name` 字段，并通过 `parent_id` 维护层级关系。

### 变更详情

#### 3.1 数据模型变更

```rust
// 改前
pub struct Account {
    pub full_name: String,  // "Assets:Bank:Checking"
    // ...
}

// 改后
pub struct Account {
    pub name: String,       // "Checking"
    // ...
}
```

#### 3.2 数据库变更

改前：

```sql
full_name TEXT NOT NULL UNIQUE
```

改后：

```sql
name TEXT NOT NULL,
UNIQUE(parent_id, name)
```

去掉 `full_name` 列和 `UNIQUE` 约束，改为 `name` 列 + `(parent_id, name)` 联合唯一约束。

#### 3.3 按名查找的替代方案

CLI 的 `get_by_name("Assets:Bank:Checking")` 改为逐级查找：

1. 查 `name = "Assets" AND parent_id IS NULL`
2. 查 `name = "Bank" AND parent_id = <上一步id>`
3. 查 `name = "Checking" AND parent_id = <上一步id>`

或者使用递归 CTE 一条 SQL 完成。

#### 3.4 重命名与移动子树

- 重命名中间节点只需更新本节点 `name`，无需级联更新后代。
- 移动子树只需更新目标节点 `parent_id` 并重新维护闭包表，不需要重写后代名称。

#### 3.5 各层改动

| 模块 | 变更 |
|------|------|
| `accounting/src/account.rs` | `full_name` → `name` |
| `accounting/src/account_type.rs` | `from_prefix()` 不再需要（类型由树根决定，见阶段 4） |
| `accounting/src/closure.rs` | `AccountNode.full_name` → `name` |
| `accounting-sql/src/schema.rs` | 表结构变更 + 种子数据调整 |
| `accounting-sql/src/repo/account.rs` | `get_by_name()` 改为逐级查找或递归 CTE |
| `accounting-sql/src/repo/posting.rs` | 测试中的 `full_name` 引用 |
| `accounting-service/src/account_service.rs` | `create_cascading()` 重写，不再拼接路径 |
| `accounting-service/src/report_service.rs` | 显示名调整 |
| `accounting-cli/src/cmd/` | 命令参数和输出格式调整 |
| `accounting-api/src/dto.rs` | DTO 字段调整 |
| `accounting-api/src/handlers/` | handler 逻辑调整 |
| `accounting-web/src/` | 去掉所有 `split(':')` 操作 |

### 出口条件

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后提交。

## 阶段 4：账户类型重构

### 目标

根据 `account-type-refactor.md`：

1. 去掉 `Account` 上的 `account_type` 存储字段。
2. `AccountType` 枚举保留为领域概念，但只实现 `FromStr`（从根节点 `name` 推导）。
3. 需要 `account_type` 的场景改为运行时从根节点推导。

### 变更详情

#### 4.1 数据模型变更

```rust
// 改前
pub struct Account {
    pub account_type: AccountType,
    // ...
}

// 改后
pub struct Account {
    // 去掉 account_type 字段
    // ...
}
```

#### 4.2 数据库变更

```sql
-- 去掉 account_type 列
-- 去掉索引 idx_accounts_type
-- 去掉 CHECK 约束
```

#### 4.3 `AccountType` 的推导方式

`AccountType` 枚举保留在核心库中，只实现 `FromStr`：

```rust
impl FromStr for AccountType {
    type Err = ...;

    fn from_str(root_name: &str) -> Result<Self, Self::Err> {
        // 根据根账户名称推导类型
    }
}
```

需要 `account_type` 的场景改为运行时推导：

| 场景 | 当前方式 | 改后方式 |
|------|---------|---------|
| 关户验证 | `validate_account_close(account.account_type, ...)` | 先查根节点，再用根节点 `name` 推导 `AccountType` |
| 报表分类 | `match account.account_type` | 从根节点推导类型后分类 |
| SQL 聚合 | `GROUP BY a.account_type` | JOIN 闭包表找根节点，GROUP BY 根节点 ID |
| 列表过滤 | `WHERE account_type = ?` | `WHERE id IN (SELECT account_id FROM account_ancestors WHERE ancestor_id = ?)` |

#### 4.4 报表 SQL 示例

改前：

```sql
SELECT tt.tag_id, p.commodity_id, a.account_type, SUM(p.amount)
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transaction_tags tt ON p.transaction_id = tt.transaction_id
GROUP BY tt.tag_id, p.commodity_id, a.account_type
```

改后：

```sql
SELECT tt.tag_id, p.commodity_id, ra.id AS root_id, SUM(p.amount)
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN account_ancestors aa ON a.id = aa.account_id AND aa.depth = (
    SELECT MAX(depth) FROM account_ancestors WHERE account_id = a.id
)
JOIN accounts ra ON aa.ancestor_id = ra.id
JOIN transaction_tags tt ON p.transaction_id = tt.transaction_id
GROUP BY tt.tag_id, p.commodity_id, ra.id
```

对于根节点自身，`depth = MAX(depth)` 就是自己，无需特殊处理。

#### 4.5 类型一致性天然保证

去掉 `account_type` 字段后，"同棵子树类型一致"这条不变量由树结构天然保证——因为类型只取决于根节点，不可能出现子节点类型不一致的情况。不再需要 `create_cascading()` 中的类型一致性校验代码。

#### 4.6 移动子树无需级联更新类型

如果未来支持移动子树（将某账户从一棵根子树移到另一棵下），不需要级联更新所有后代的 `account_type`——因为类型由新的根节点决定，运行时自动推导。

#### 4.7 各层改动

| 模块 | 变更 |
|------|------|
| `accounting/src/account.rs` | 去掉 `account_type` 字段 |
| `accounting/src/account_type.rs` | 保留枚举，删除 `from_prefix()` 等方法，只实现 `FromStr` |
| `accounting/src/validation.rs` | `validate_account_close()` 参数从 `AccountType` 改为根节点类型或根节点名称 |
| `accounting/src/closure.rs` | `AccountNode` 去掉 `account_type` |
| `accounting-sql/src/schema.rs` | 去掉 `account_type` 列、索引、CHECK 约束；种子数据调整 |
| `accounting-sql/src/repo/account.rs` | 去掉 `account_type` 的读写；`map_account()` 去掉类型映射 |
| `accounting-sql/src/repo/posting.rs` | SQL 聚合改为 JOIN 闭包表找根节点 |
| `accounting-service/src/account_service.rs` | 去掉类型一致性校验；`create_cascading()` 简化 |
| `accounting-service/src/report_service.rs` | 所有 `match account.account_type` 改为从根节点推导 |
| `accounting-service/src/transaction_service.rs` | 测试代码调整 |
| `accounting-cli/src/cmd/` | `--type` 参数改为按根节点过滤 |
| `accounting-api/src/dto.rs` | `account_type` 改为只读推导字段 |
| `accounting-api/src/handlers/` | handler 逻辑调整 |

### 出口条件

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后提交。

## 实施顺序与依赖关系

```
前置迁移 fcb8fea6
    ↓
阶段 1：移除冗余字段
    ↓
阶段 2：审计字段改进
    ↓
阶段 3：账户名称重构（full_name → name）
    ↓
阶段 4：账户类型重构（删除 account_type 字段）
```

**关键依赖：**

- 阶段 3 必须在阶段 4 之前完成。因为阶段 4 需要从根节点 `name` 推导 `AccountType`，而阶段 3 才将根节点存储从 `full_name` 改为 `name`。
- 阶段 1 和阶段 2 相对独立，可以放在最前面做，减少后续改动的表面积。
- 前置迁移必须最先完成，因为它会改变多个后续阶段要修改的文件（如 `account.rs`、`schema.rs`）。

## 验证策略

每个阶段（包括前置迁移）完成后必须依次执行：

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

全部通过后方可提交。这样可以保证：

- 代码格式一致。
- Clippy 无警告。
- 测试覆盖的现有行为不被破坏。
- 每个阶段都是一个可回滚的稳定点。

## 风险与注意事项

1. **cherry-pick 冲突：** `fcb8fea6` 与当前分支 `4b9fa057` 在 `Liability` 删除上有重叠，需要仔细合并，避免重复删除或遗漏。
2. **`DROP COLUMN` 兼容性：** 需要确认当前 rusqlite bundled 的 SQLite 版本 >= 3.35.0。如果版本不足，需要改为"创建新表 → 复制数据 → 删除旧表 → 重命名"的迁移方式。
3. **`sum_by_channel()` 改动：** 移除 `posting.channel_id` 后，必须确保该统计查询改为 JOIN `transactions` 并按 `t.channel_id` 分组，否则数据会丢失。
4. **前端 `split(':')`：** 阶段 3 需要全面检查 `accounting-web/src/` 中对 `full_name` 的 split/pop 操作，避免遗漏。
5. **根节点类型推导：** 阶段 4 中，根节点自身的闭包表深度就是最大值，无需特殊分支；但需要确保 SQL 在空树等边界场景下行为正确。
6. **测试数据：** 多个阶段的 schema 和种子数据会连续变更，需要同步更新测试中的硬编码字段和 fixture。
