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

`Vec::default()` 是空 Vec，语义等价于"不筛选"，与 `Option<T>` 为 `None` 的行为一致。

### 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 空 Vec 语义 | 不筛选（匹配所有） | 与 `Option<T>` 为 `None` 的行为一致，最自然 |
| 类型选择 | `Vec<T>` 而非 `Option<Vec<T>>` | 空 Vec 已表达"不筛选"，无需额外 Option 层 |
| 多选逻辑 | OR（并集） | `WHERE id IN (1,2,3)` 匹配任一，符合用户直觉 |
| `has_installment` | 删除 | 完全死代码，SQL 层和数据库均未使用 |
| `has_reimbursable` | 保留 | transaction list/count 中已生效 |
| `keyword` | 保持 `Option<String>` | 文本搜索不适合多选 |
| 账户过滤 | JOIN → EXISTS | 避免多选 IN + JOIN 导致行膨胀和别名管理复杂化 |
| 标签过滤（transaction） | JOIN → EXISTS | 避免 JOIN 行膨胀，IN 表达 OR 语义 |
| 标签过滤（sum_by_tag） | 保留 JOIN | GROUP BY tag_id 需要 JOIN，改为 `IN (...)` 即可 |
| 标签过滤（sum_by_member/channel） | EXISTS + IN | 已使用 EXISTS，改为 `IN (...)` 即可 |

### SQL 生成变化

#### transaction.rs list/count — 账户过滤

```sql
-- 改前：JOIN postings 导致行膨胀
JOIN postings p ON p.transaction_id = transactions.id
...
WHERE p.account_id = ?

-- 改后：EXISTS 子查询避免行膨胀
AND EXISTS (
    SELECT 1 FROM postings p
    WHERE p.transaction_id = transactions.id
    AND p.account_id IN (?, ?, ?)
)
```

这一改动也简化了 `has_reimbursable` 的 JOIN 别名管理——当 `account_ids` 非空时不再需要检查 postings 是否已被 JOIN，因为账户过滤不再 JOIN postings 表。

#### transaction.rs list/count — 标签过滤

```sql
-- 改前
JOIN transaction_tags tt ON tt.transaction_id = transactions.id
...
WHERE tt.tag_id = ?

-- 改后
AND EXISTS (
    SELECT 1 FROM transaction_tags tt
    WHERE tt.transaction_id = transactions.id
    AND tt.tag_id IN (?, ?, ?)
)
```

#### transaction.rs list/count — has_reimbursable

`has_reimbursable` 保持独立 JOIN（使用专用别名 `p_reimb`），不再与账户过滤共享 JOIN：

```sql
-- 改后（has_reimbursable + 多选账户可共存）
AND EXISTS (
    SELECT 1 FROM postings p
    WHERE p.transaction_id = transactions.id
    AND p.account_id IN (?, ?)
)
AND EXISTS (
    SELECT 1 FROM postings p_reimb
    WHERE p_reimb.transaction_id = transactions.id
    AND p_reimb.is_reimbursable = 1
)
```

#### posting.rs — 三种方法的不同处理

| 方法 | 当前方式 | 多选后 |
|------|---------|--------|
| `sum_by_tag` | JOIN transaction_tags（用于 GROUP BY） | 保留 JOIN，`tt.tag_id = ?` → `tt.tag_id IN (...)`；`p.account_id = ?` → `p.account_id IN (...)` |
| `sum_by_member` | EXISTS + `tt.tag_id = ?` | `tt.tag_id IN (...)`；`p.account_id IN (...)`；`t.member_id IN (...)` |
| `sum_by_channel` | EXISTS + `tt.tag_id = ?` | `tt.tag_id IN (...)`；`p.account_id IN (...)`；`t.channel_id IN (...)` |

注意：`sum_by_tag` 中标签过滤 `tt.tag_id IN (...)` 与 GROUP BY `tt.tag_id` 的交互是正确的——查询 `tag_ids=[1,2]` 时，交易同时有 tag1 和 tag2，会在 tag1 和 tag2 两个分组各出现一次。

#### 统计方法中的维度清空

统计方法按某个维度分组时，应清空该维度的过滤条件（"维度自身不过滤自身"）：

```rust
// 改前
filter.tag_id = None;      // stats_by_tag
filter.member_id = None;   // stats_by_member
filter.channel_id = None;  // stats_by_channel

// 改后
filter.tag_ids.clear();
filter.member_ids.clear();
filter.channel_ids.clear();
```

### API 层

```
GET /transactions?account=1&account=2&member=3&tag=food&tag=travel
```

重复参数名天然映射到 `Vec`。

**参数类型决策**：

| 参数 | 类型 | 说明 |
|------|------|------|
| `account` | `Vec<i64>` | 直接传 ID |
| `member` | `Vec<i64>` | 直接传 ID |
| `channel` | `Vec<i64>` | 直接传 ID |
| `tag` | `Vec<String>` | 按名称查询，handler 内逐个解析为 ID |

标签保持按名称查询，与当前行为一致。前端传标签名更直观，且标签数量通常较少，逐个解析无性能问题。当某个标签名不存在时，返回错误提示。

**`StatsQuery` 扩展**：当前 API 统计端点只支持日期过滤，本次不扩展。统计端点的多选过滤支持留作后续改进。

### CLI 层

```bash
# 改后（支持多值）
tx list --account 1 --account 2 --member 3 --member 4 --tag food --tag travel
```

clap 参数类型改为 `Vec<_>`。标签参数保持按名称查询，与 API 一致。`--installment` 参数删除。

## 影响范围

| 文件 | 变更 |
|------|------|
| `accounting/src/transaction_filter.rs` | 字段 `Option<Id>` → `Vec<Id>`，删除 `has_installment` |
| `accounting-sql/src/repo/transaction.rs` | `list()`/`count()`：账户过滤从 JOIN 改为 EXISTS + IN；标签过滤从 JOIN 改为 EXISTS + IN；其他字段 `= ?` → `IN (...)`；两处需同步修改 |
| `accounting-sql/src/repo/posting.rs` | `sum_by_tag()`：`tt.tag_id = ?` → `IN (...)`，`p.account_id = ?` → `IN (...)` 等；`sum_by_member()`/`sum_by_channel()`：EXISTS 中 `= ?` → `IN (...)` |
| `accounting-api/src/handlers/transaction.rs` | `TxQuery` 字段改为 `Vec`；标签保持 `Vec<String>` 按名称解析 |
| `accounting-cli/src/cmd/tx.rs` | `TxListArgs` 参数改为 `Vec`；删除 `--installment`；删除 `build_filter()` 中 `has_installment` 赋值；标签按名称逐个解析为 ID |
| `accounting-cli/src/cmd/report.rs` | `StatArgs` 相关参数适配 |
| `accounting-service/src/report_service.rs` | `filter.tag_id = None` → `filter.tag_ids.clear()` 等；`filter.member_id = None` → `filter.member_ids.clear()`；`filter.channel_id = None` → `filter.channel_ids.clear()` |
| `accounting-web/src/stores/transaction.ts` | `fetchTransactions` 参数类型支持多值 |
| `accounting-web/src/views/` | 过滤器 UI 从单选改为多选 |

## 不做的事

- 不修改 `has_reimbursable`，保留现有行为
- 不改变 `keyword` 的单选语义
- 不增加新的过滤字段
- 不修改 posting 统计查询中 `has_reimbursable` 未使用的问题（后续单独处理）
- 不扩展 `StatsQuery` 支持多选过滤（后续改进）

## 测试计划

- 空 Vec 过滤（应返回全部）
- 单元素 Vec 过滤（等价于原 `Option<Id>`）
- 多元素 Vec 过滤（OR 语义）
- 多标签过滤 + 多选账户过滤组合
- 多选标签在 `sum_by_tag` 中的 GROUP BY 行为
- `has_reimbursable` + 多选账户过滤共存
- 统计方法中维度自身清空（`sum_by_tag` 忽略 `tag_ids`）
- CLI `--installment` 参数删除后向后兼容性
