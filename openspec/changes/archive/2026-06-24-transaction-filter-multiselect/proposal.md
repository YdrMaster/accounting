# Proposal: transaction-filter-multiselect

> 回溯归档：本变更所述设计已实现（~2026-06-24）。活规格 `transaction-filter` 描述了筛选的**现状**（多选字段），但未记录"从单选 `Option<Id>` 改为多选 `Vec<Id>`"这一**重构决策**；本归档补录该决策来源。`--skip-specs` 归档。

## Why

`TransactionFilter` 的可枚举过滤字段（账户、成员、渠道、标签）此前均为单选 `Option<Id>`，无法表达"查看多个账户的交易"等常见需求。数据库层天然支持 `IN (?, ?, ?)` 多值匹配，单选是人为限制。

同时，`has_installment` 字段是完全死代码——SQL 层从未使用、数据库无对应列、CLI `--installment` 参数无任何效果。Review（`simplify-data-model` D2/RULE_06 延后项）已将其标为同类冗余死代码，建议趁此重构一并移除。

## What Changes

- **数据模型**：`account_id`/`member_id`/`channel_id`/`tag_id` 从 `Option<Id>` 改为 `Vec<Id>`（`account_ids`/`member_ids`/`channel_ids`/`tag_ids`），空 Vec = 不筛选（与 `None` 语义一致）。删 `has_installment`。
- **SQL 生成**：account/tag 过滤从 `JOIN … WHERE = ?` 改为 `EXISTS (SELECT 1 … WHERE … IN (?, ?, ?))`，避免多选 + JOIN 导致行膨胀与别名管理复杂化。`has_reimbursable` 保持独立 `EXISTS`（专用别名 `p_reimb`），与多选账户过滤可共存。
- **统计方法**：`sum_by_tag` 保留 JOIN（GROUP BY 需要），`tt.tag_id = ?`/`p.account_id = ?` 改 `IN (...)`；`sum_by_member`/`sum_by_channel` 的 EXISTS 中 `=` 改 `IN (...)`。维度分组时清空该维度自身过滤（"维度不过滤自身"）：`filter.tag_ids.clear()` 等。
- **API**：`GET /transactions?account=1&account=2&member=3&tag=food&tag=travel`，重复参数名天然映射 `Vec`。`account`/`member`/`channel` 直传 ID（`Vec<i64>`），`tag` 保持按名称查询（`Vec<String>`，handler 逐个解析为 ID，不存在则错误）。`StatsQuery` 多选过滤**不扩展**（留作后续）。
- **CLI**：`tx list --account 1 --account 2 --member 3 --tag food --tag travel`，clap 参数改 `Vec`；`--installment` 删除。

## Capabilities

### New Capabilities

（无——筛选多选是对既有 `transaction-filter` 能力的形态重构，活规格已记录多选现状。）

### Modified Capabilities

（无——本归档补决策来源，不改活规格。）

## Impact

- `accounting/src/transaction_filter.rs`：字段 `Option<Id>` → `Vec<Id>`，删 `has_installment`。
- `accounting-sql/src/repo/transaction.rs`：`list`/`count` 账户过滤 JOIN→EXISTS+IN；标签过滤 JOIN→EXISTS+IN；其他 `= ?`→`IN (...)`。
- `accounting-sql/src/repo/posting.rs`：`sum_by_tag`/`sum_by_member`/`sum_by_channel` 的 `=`→`IN (...)`。
- `accounting-api/src/handlers/transaction.rs`：`TxQuery` 字段改 `Vec`；标签 `Vec<String>` 按名解析。
- `accounting-cli/src/cmd/tx.rs`：`TxListArgs` 参数改 `Vec`；删 `--installment` 与 `build_filter` 中 `has_installment` 赋值；标签按名逐个解析 ID。
- `accounting-service/src/report_service.rs`：`filter.tag_id = None`→`filter.tag_ids.clear()` 等。
- `accounting-web/src/stores/transaction.ts` 及 `views/`：`fetchTransactions` 支持多值；过滤器 UI 单选→多选。
