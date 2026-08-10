# Tasks: transaction-filter-multiselect

> 回溯归档：以下任务均已实现（~2026-06-24），checkbox 标记为 `[x]` 复盘。

## 1. 数据模型（`accounting/src/transaction_filter.rs`）

- [x] 1.1 `account_id`→`account_ids: Vec<AccountId>`、`member_id`→`member_ids`、`channel_id`→`channel_ids`、`tag_id`→`tag_ids`
- [x] 1.2 删 `has_installment: Option<bool>`
- [x] 1.3 空 Vec = 不筛选（与原 `None` 行为一致）

## 2. SQL 层（`accounting-sql/src/repo`）

- [x] 2.1 `transaction.rs` `list`/`count`：账户过滤 `JOIN postings … WHERE = ?` → `EXISTS (SELECT 1 … WHERE … IN (?, ?, ?))`
- [x] 2.2 `transaction.rs` `list`/`count`：标签过滤 `JOIN transaction_tags … WHERE = ?` → `EXISTS … IN (...)`
- [x] 2.3 `transaction.rs`：日期/其他字段 `= ?`→`IN (...)`
- [x] 2.4 `posting.rs` `sum_by_tag`：保留 JOIN，`tt.tag_id = ?`→`IN (...)`、`p.account_id = ?`→`IN (...)`
- [x] 2.5 `posting.rs` `sum_by_member`/`sum_by_channel`：EXISTS 中 `= ?`→`IN (...)`
- [x] 2.6 `has_reimbursable` 独立 `EXISTS`（别名 `p_reimb`），与多选账户过滤可共存

## 3. Service 层

- [x] 3.1 `report_service.rs`：`filter.tag_id = None`→`filter.tag_ids.clear()`；`member_id`/`channel_id` 同理（维度清空：sum_by_tag 清 tag_ids 等）

## 4. API 层

- [x] 4.1 `handlers/transaction.rs` `TxQuery`：字段改 `Vec`；`account`/`member`/`channel` 为 `Vec<i64>`，`tag` 为 `Vec<String>`
- [x] 4.2 标签按名逐个解析为 ID，某标签不存在时返回明确错误
- [x] 4.3 `StatsQuery` 多选过滤不扩展（留作后续）

## 5. CLI 层

- [x] 5.1 `cmd/tx.rs` `TxListArgs`：`--account`/`--member`/`--channel`/`--tag` 参数改 `Vec`
- [x] 5.2 删 `--installment` 参数；删 `build_filter` 中 `has_installment` 赋值
- [x] 5.3 标签按名逐个解析为 ID（与 API 一致）
- [x] 5.4 `cmd/report.rs` `StatArgs` 相关参数适配

## 6. 前端

- [x] 6.1 `stores/transaction.ts` `fetchTransactions` 参数类型支持多值
- [x] 6.2 `views/` 过滤器 UI 从单选改为多选

## 7. 测试

- [x] 7.1 空 Vec（返回全部）、单元素（等价原单选）、多元素（OR 语义）
- [x] 7.2 多标签 + 多账户组合
- [x] 7.3 多标签在 `sum_by_tag` 中的 GROUP BY 行为（同交易多标签各分组计数一次）
- [x] 7.4 `has_reimbursable` + 多选账户过滤共存
- [x] 7.5 统计方法维度自身清空（`sum_by_tag` 忽略 `tag_ids`）
- [x] 7.6 CLI `--installment` 删除后向后兼容性
