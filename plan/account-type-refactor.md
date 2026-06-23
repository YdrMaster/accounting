# 账户类型重构：去掉 AccountType 存储字段，由树根推导

## 现状

当前 `Account` 结构体存储 `account_type: AccountType` 枚举（Asset/Liability/Equity/Income/Expense），数据库中为 `account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 5)`，并建立了索引 `idx_accounts_type`。

同时，账户通过 `parent_id` + 闭包表构成树状结构，5 棵子树分别对应 5 种账户类型：

```
Assets (根, type=1)
  └── Bank (type=1)
        └── Checking (type=1)
Liability (根, type=2)
Equity (根, type=3)
Income (根, type=4)
Expense (根, type=5)
```

## 问题

### 1. account_type 与树根构成冗余

`account_type` 完全由账户在树中的位置（根节点）决定。同一棵子树内所有节点的 `account_type` 必须等于根节点的类型——这是一条业务不变量，但当前没有任何机制强制保证。如果某条记录的 `account_type` 被错误修改（如手动 SQL），就会出现同棵子树内有不同类型账户的数据不一致。

### 2. 创建时需要手动设置并校验类型一致性

当前 `create_cascading()` 中需要：
1. 从 `full_name` 第一段用 `from_prefix()` 推导 `account_type`
2. 创建子节点时手动设置 `account_type`
3. 检查已存在节点的 `account_type` 是否与新推导的类型一致

如果类型由树根决定，这些代码全部不需要。

### 3. from_prefix() 本身就是从根节点名称推导的

`from_prefix("Assets") → Asset`、`from_prefix("负债") → Liability` 等逻辑，本质上就是"根节点的名称决定类型"。这证明了树根是更本质的信息源，`account_type` 是派生属性。

## 重构方案

### 核心原则

**去掉 `Account` 上的 `account_type` 存储字段，`AccountType` 枚举保留为领域概念，在需要时从根节点推导。**

### 数据模型变更

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

### 数据库变更

```sql
-- 去掉 account_type 列
-- CREATE TABLE accounts (
--   account_type INTEGER NOT NULL,  ← 删除
--   ...
-- );
-- 去掉索引 idx_accounts_type       ← 删除
-- 去掉 CHECK 约束                   ← 删除
```

### AccountType 的推导方式

`AccountType` 枚举保留在核心库中，提供从根节点推导的方法：

```rust
impl AccountType {
    /// 根据根账户名称推导类型
    pub fn from_root_name(root_name: &str) -> Option<Self> {
        // 复用现有 from_prefix 逻辑
        // 或从根账户 ID 查询
    }
}
```

需要 `account_type` 的场景改为运行时推导：

| 场景 | 当前方式 | 改后方式 |
|------|---------|---------|
| 关户验证 | `validate_account_close(account.account_type, ...)` | 先查根节点类型，再传参 |
| 报表分类 | `match account.account_type` | 从根节点推导类型后分类 |
| SQL 聚合 | `GROUP BY a.account_type` | JOIN 闭包表找根节点，GROUP BY 根节点 ID |
| 列表过滤 | `WHERE account_type = ?` | `WHERE id IN (SELECT account_id FROM account_ancestors WHERE ancestor_id = ?)` |

### 报表 SQL 示例

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

> 注：对于根节点自身，`depth = MAX(depth)` 就是自己，因此无需特殊处理。也可以在闭包表中增加 `is_root` 标记或用 `parent_id IS NULL` 简化查询。

### 类型一致性天然保证

去掉 `account_type` 字段后，"同棵子树类型一致"这条不变量由树结构天然保证——因为类型只取决于根节点，不可能出现子节点类型不一致的情况。不再需要 `create_cascading()` 中的类型一致性校验代码。

### 移动子树无需级联更新类型

如果未来支持移动子树（将某账户从一棵根子树移到另一棵下），不需要级联更新所有后代的 `account_type`——因为类型由新的根节点决定，运行时自动推导。

## 代价评估

| 代价 | 影响 |
|------|------|
| 报表 SQL 需要额外 JOIN 闭包表定位根节点 | 语句变长，但记账系统数据量小，性能无感 |
| 关户验证需要先查根节点类型 | 多一次查询 |
| CLI `--type` 过滤变为按根节点过滤 | 用户接口小幅调整 |
| API 响应中 `account_type` 变为推导字段 | 可保留在 DTO 中作为只读派生字段，不影响前端 |

## 影响范围

| 模块 | 变更 |
|------|------|
| `accounting/src/account.rs` | 去掉 `account_type` 字段 |
| `accounting/src/account_type.rs` | 保留枚举，增加 `from_root_name()`，去掉 `from_prefix()` |
| `accounting/src/validation.rs` | `validate_account_close()` 参数从 `AccountType` 改为根节点类型 |
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

## 结论

去掉 `Account` 上的 `account_type` 存储字段，改为从树根运行时推导。`account_type` 是树根的派生属性，同时存储两者构成冗余，且当前缺乏机制保证不变量（同子树类型一致）。去掉后消除了数据不一致的可能性，简化了创建流程，使移动子树无需级联更新类型。代价是报表 SQL 多一个 JOIN，对记账系统的数据规模完全可接受。

## 与账户名称重构的关系

本方案与 `account-name-refactor.md` 中的名称重构是互补的：

- 名称重构去掉 `full_name` 后，`from_prefix(segments[0])` 推导类型的机制也自然失效，需要本方案提供替代推导方式
- 两个重构可以一起实施，也可以独立实施（如果先做名称重构，需要临时保留 `from_prefix()` 并改为从根节点 `name` 推导）
