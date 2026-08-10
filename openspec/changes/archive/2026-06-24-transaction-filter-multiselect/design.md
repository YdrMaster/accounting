# Design: transaction-filter-multiselect

## Context

回溯归档 ~2026-06-24。`TransactionFilter` 此前可枚举字段单选，无法多账户/多标签查询；`has_installment` 死代码。`simplify-data-model` review 将 `has_installment` 标为延后清理项，本变更接力完成。活规格 `transaction-filter` 记录筛选行为**现状**（多选），本归档补"从单选改多选"的决策来源与 SQL 形态选择。

## Goals / Non-Goals

**Goals:**

- 账户/成员/渠道/标签过滤支持多选，空选等价于不筛选。
- 删除 `has_installment` 死代码与 CLI `--installment`。
- 多选不引入行膨胀或别名复杂化。

**Non-Goals:**

- 不改 `has_reimbursable`（保留现状）。
- 不改 `keyword`（单选文本搜索不适合多选）。
- 不扩展 `StatsQuery` 多选过滤（留作后续）。
- 不新增过滤字段。

## Decisions

### D1: `Vec<Id>` 而非 `Option<Vec<Id>>`，空 Vec = 不筛选

字段 `account_ids: Vec<Id>` 等。`Vec::default()` 是空 Vec，语义"不筛选（匹配所有）"——与 `Option<T>` 为 `None` 的行为一致，因此无需再包一层 `Option`。

**备选**（否决）：`Option<Vec<Id>>`。否决：空 Vec 已表达"不筛选"，额外 Option 层是冗余壳，徒增构造与匹配成本且无新语义。

### D2: 多选 OR（并集）语义

`WHERE id IN (1,2,3)` 匹配任一，符合"查看多个账户交易"的筛选直觉（并集而非交集）。

### D3: 账户/标签过滤从 JOIN 改 EXISTS，避免行膨胀

```sql
-- 改前：JOIN postings 导致行膨胀
JOIN postings p ON p.transaction_id = transactions.id ... WHERE p.account_id = ?

-- 改后：EXISTS 子查询
AND EXISTS (SELECT 1 FROM postings p WHERE p.transaction_id = transactions.id AND p.account_id IN (?, ?, ?))
```

**理由**：多选 + JOIN 会令一条交易因命中多个筛选值而重复出现（行膨胀），需 `DISTINCT` 收尾且别名管理随多选复杂度上升。EXISTS 只判存在性、不产生行，天然无膨胀。这一改动附带简化了 `has_reimbursable` 的 JOIN 别名管理——账户过滤不再 JOIN postings，`has_reimbursable` 独立 `EXISTS`（别名 `p_reimb`）与之可共存。

### D4: `sum_by_tag` 保留 JOIN

```sql
-- 保留 JOIN（GROUP BY 需要）
... JOIN transaction_tags tt ... GROUP BY tt.tag_id
-- 仅把 = ? 改 IN (...)
WHERE tt.tag_id IN (1,2) ...
```

**理由**：`sum_by_tag` 需 `GROUP BY tt.tag_id`，JOIN 是 GROUP BY 的天然载体，改 EXISTS 反而需另设法分组。IN 表达 OR 语义：交易同时有 tag1、tag2 时会在两个分组各计一次——正是预期。`sum_by_member`/`sum_by_channel` 已用 EXISTS，只需 `=`→`IN (...)`。

### D5: 维度分组时清空该维度自身过滤

```rust
// sum_by_tag 清空 tag_ids、sum_by_member 清空 member_ids、sum_by_channel 清空 channel_ids
filter.tag_ids.clear(); // 原 filter.tag_id = None
```

**理由**："按标签分组统计"时若再过滤某标签，会把统计范围自限到该标签内，与"分组看各标签"语义相悖。清空自身维度 = "维度不过滤自身"。从 `Option<Id>` 的 `= None` 等价迁移为 `Vec` 的 `.clear()`。

### D6: `has_installment` 连同 CLI `--installment` 一起删

`has_installment: Option<bool>` 在 SQL 层从未消费、数据库无对应列、CLI `--installment` 无效果——完全死代码。趁多选重构一并删，避免"为死字段做迁移"的无意义工作。`has_reimbursable` 不同：它在 list/count 中已生效，保留。

### D7: API 标签按名查询、其余按 ID，CLI 与之对齐

| 参数 | 类型 | 说明 |
|------|------|------|
| `account`/`member`/`channel` | `Vec<i64>` | 直传 ID |
| `tag` | `Vec<String>` | handler 逐个解析为 ID，不存在则错误 |

**理由**：前端传标签名比传 ID 直观；标签数量少、逐个解析无性能问题。account/member/channel 走 ID 是既有路径，保持。某标签名不存在时明确报错而非静默忽略。`StatsQuery` 多选过滤不扩展——统计端的多选属后续改进，不在本重构范围。

## Risks / Trade-offs

- **空 Vec 与未过滤的等价性**靠调用方"空=全选"约定维持；若某查询误把空 Vec 当"无结果"会引入 bug。测试覆盖空/单元素/多元素三种情况。
- **`sum_by_tag` 的 IN + GROUP BY 交互**（D4）：交易同属多标签会在各分组重复计数——这是"按标签看总额"的预期，但若有人误以为是去重额会困惑。在设计文档与注释中点明。
- **API 标签按名、其余按 ID 的不对称**增加调用方记忆负担；权衡"直观性"胜出。
- **`--installment` 删除的向后兼容**：旧脚本若有此参数会报错。接受——它本就无效果，删除是行为透明化。

## Migration Plan

无数据迁移。Domain 改字段类型 → repo SQL 形态（JOIN→EXISTS、`=`→`IN`）→ service `.clear()` 适配 → API `TxQuery` → CLI `TxListArgs`/删 `--installment` → 前端 store 与筛选器 UI 多选。测试覆盖空/单/多元素、多标签 GROUP BY、`has_reimbursable`+多账户共存、维度清空。

## Open Questions

- 无（回溯归档，决策已定型）。
