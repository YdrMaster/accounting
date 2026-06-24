# TransactionFilter 多选重构设计

## 动机

当前 `TransactionFilter` 的可枚举过滤字段（账户、成员、渠道、标签）均为单选 `Option<Id>`，无法表达"查看多个账户的交易"等常见需求。数据库层天然支持 `IN (?, ?, ?)` 多值匹配，单选是人为限制。

同时，`has_installment` 字段是完全死代码——SQL 层从未使用，数据库也无对应列，CLI 的 `--installment` 参数无任何效果，应趁此重构一并移除。

## 变更

### 数据模型

```rust
// 改前
pub struct TransactionFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub account_id: Option<AccountId>,
    pub member_id: Option<MemberId>,
    pub channel_id: Option<ChannelId>,
    pub tag_id: Option<TagId>,
    pub keyword: Option<String>,
    pub has_installment: Option<bool>,      // 删除
    pub has_reimbursable: Option<bool>,
}

// 改后
pub struct TransactionFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub account_ids: Vec<AccountId>,        // 多选，空=不筛选
    pub member_ids: Vec<MemberId>,           // 多选，空=不筛选
    pub channel_ids: Vec<ChannelId>,         // 多选，空=不筛选
    pub tag_ids: Vec<TagId>,                 // 多选，空=不筛选
    pub keyword: Option<String>,
    pub has_reimbursable: Option<bool>,
}
```

### 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 空 Vec 语义 | 不筛选（匹配所有） | 与 `Option<T>` 为 `None` 的行为一致，最自然 |
| 类型选择 | `Vec<T>` 而非 `Option<Vec<T>>` | 空 Vec 已表达"不筛选"，无需额外 Option 层 |
| 多选逻辑 | OR（并集） | `WHERE id IN (1,2,3)` 匹配任一，符合用户直觉 |
| `has_installment` | 删除 | 完全死代码，SQL 层和数据库均未使用 |
| `has_reimbursable` | 保留 | transaction list/count 中已生效 |
| `keyword` | 保持 `Option<String>` | 文本搜索不适合多选 |

### SQL 生成

```sql
-- 改前
WHERE transactions.account_id = ?

-- 改后（非空）
WHERE transactions.account_id IN (?, ?, ?)

-- 改后（空 Vec）
-- 不追加此条件
```

标签过滤原本使用 `JOIN transaction_tags` + `tt.tag_id = ?`，改为 `IN` 时需切换为 `EXISTS` 子查询以正确处理多标签 OR 语义：

```sql
-- 改后（多标签）
AND EXISTS (
    SELECT 1 FROM transaction_tags tt
    WHERE tt.transaction_id = transactions.id
    AND tt.tag_id IN (?, ?, ?)
)
```

### API 层

```
GET /transactions?account=1&account=2&member=3&tag=food&tag=travel
```

重复参数名天然映射到 `Vec`。`TxQuery` 结构体对应字段改为 `Vec<i64>`。

### CLI 层

```bash
# 改后（支持多值）
tx list --account 1 --account 2 --member 3 --member 4
```

clap 参数类型改为 `Vec<_>`，`--installment` 参数删除。

## 影响范围

| 文件 | 变更 |
|------|------|
| `accounting/src/transaction_filter.rs` | 字段 `Option<Id>` → `Vec<Id>`，删除 `has_installment` |
| `accounting-sql/src/repo/transaction.rs` | `list()`/`count()` 条件从 `= ?` 改为 `IN (...)` 或跳过；标签改为 EXISTS 子查询 |
| `accounting-sql/src/repo/posting.rs` | `sum_by_tag()`/`sum_by_member()`/`sum_by_channel()` 同样修改 |
| `accounting-api/src/handlers/transaction.rs` | `TxQuery` 改为 `Vec` 参数，构建 filter 适配 |
| `accounting-api/src/handlers/report.rs` | `StatsQuery` 如有相关字段适配 |
| `accounting-cli/src/cmd/tx.rs` | `TxListArgs` 参数改为 `Vec`，删除 `--installment`，构建 filter 适配 |
| `accounting-cli/src/cmd/report.rs` | `StatArgs` 相关参数适配 |
| `accounting-service/src/report_service.rs` | `filter.tag_id = None` → `filter.tag_ids.clear()` 等适配 |
| `accounting-web/src/` | 前端 filter 相关代码适配 |

## 不做的事

- 不修改 `has_reimbursable`，保留现有行为
- 不改变 `keyword` 的单选语义
- 不增加新的过滤字段
- 不修改 posting 统计查询中 `has_reimbursable` 未使用的问题（后续单独处理）