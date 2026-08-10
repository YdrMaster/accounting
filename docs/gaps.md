# 已知缺口与待办

> 本文件记录已识别、但本轮文档清理**不在范围内解决**的缺口。每条说明现状、为何保留、将来如何推进。

## 退款与报销（refund / reimbursement）

### 现状（数据模型已落地，规格未记录，UI 未实现）

退款/报销能力的数据模型已经在代码中，但其**行为从未进入 openspec 活规格**，且原设计的**完整 UI 流程未实现**。三者并存于"半成品"状态。

**已落地的数据层**（代码即事实）：

- `transactions.kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3)` — 交易级类型（`accounting-sql/src/schema.rs` 的 `transactions` 表）。
- `TransactionKind` 枚举（`accounting/src/transaction.rs`）：`Normal=1`、`Refund=2`、`Reimbursement=3`；`Transaction` 结构体携带 `kind: TransactionKind`。
- `Posting.is_reimbursable: bool`（`accounting/src/posting.rs`）— 标记某分录可被报销冲减。
- `postings.linked_posting_id` + `postings.reversal_total`（`accounting-sql/src/schema.rs`）— 冲减分录指向原分录；`reversal_total` 由触发器自动维护（`trg_postings_reversal_insert`/`_delete`/`_update`，触发条件 `SELECT kind FROM transactions WHERE id = NEW.transaction_id) IN (2, 3) AND NEW.linked_posting_id IS NOT NULL`）。
- 索引 `idx_postings_linked`、`idx_transactions_kind`。
- 前端 `TransactionFormOverlay.vue`：分录行有 `isReimbursable` 复选框（`t('txForm.reimbursable')`），DTO 透传 `kind`/`is_reimbursable`/`linked_posting_id`（`accounting-api/src/dto.rs`、`handlers/transaction.rs`、`accounting-web/src/types/api.ts`）。

**规格缺口**：

- openspec 活规格 `transaction-form` **完全未提及** `TransactionKind` / `is_reimbursable` / `linked_posting_id` / `reversal_total` / 退款报销任何行为（`grep` 计数为 0）。即"代码能存这些字段，但无规格定义其行为语义"。

**模型分歧（设计文档 vs 实际代码）**：

最早的设计文档 `spec/refund-reimbursement-design.md`（已随 `spec/` 收敛移除，要点见下）将 `kind` 设计在 **postings 级**（`PostingKind { Normal/Refund/Reimbursement }`），由 posting 级 `kind` 驱动冲减触发器。实际代码把 `kind` 放在了 **transactions 级**（`TransactionKind`），触发器改为读 `transactions.kind`。`Posting` 结构体则用 `is_reimbursable: bool` 而非 posting 级 `kind` enum。即：**冲减的"类型维度"上移到了交易级，posting 级只保留可报销标记**。这是相对原设计的简化变体，但其决策来源未在任何 openspec 归档 `design.md` 中记录。

### 未实现的设计（原 `spec/refund-reimbursement-design.md` 与 superpowers #7 设想的 UI 流程）

以下部分在设计中描述、但代码中**不存在**：

- **Dashboard 4 模式切换**（普通 / 范围 / 退款 / 报销）页面级筛选。
- **底部抽屉 + 分录多选**（`selectedPostings`）批量冲减交互。
- **专用路由** `/transaction/refund`、`/transaction/reimbursement`。
- **原交易选择器**：表单中选择退款/报销后弹出原交易，自动列出其 Expense/Income 分录供选择被冲减对象，并自动填充相同账户与金额方向。
- **交易详情的冲减展示**：Normal 分录显示"已被冲减 ¥X"（用 `reversal_total`）；Refund/Reimbursement 分录显示"退"/"报"标记并可点击跳转原交易；原交易详情列出所有冲减它的分录。

### 为什么本轮不解决

经确认为"记录缺口、留待以后"（非本轮文档清理范围）：

1. 这是一个**进行中/未完成的功能**，而非纯文档问题；推进需要产品决策（是否仍按交易级 `kind` 的简化模型走，还是回到设计文档的 posting 级模型；UI 是否按原设计做）。
2. 规格补写应与实现决策同步——在"做不做、怎么做的"未定时先写规格会锚定一个可能被推翻的形态。

### 将来如何推进

若决定推进，建议作为一个新的 openspec 变更提案走 `/openspec-propose` 流程，至少澄清以下决策：

1. **模型口径**：保留交易级 `TransactionKind`（现状）还是改为设计文档的 posting 级 `PostingKind`？两者触发器语义不同。
2. **`is_reimbursable` 的角色**：它是"可被报销冲减的原分录标记"，与交易级 `kind` 如何配合？
3. **UI 形态**：是否实现原设计的 4 模式 Dashboard / 原交易选择器 / 抽屉多选，还是采用更轻量的形态？
4. **规格落点**：行为补入 `transaction-form` 规格的 Requirements/Scenarios，还是新建 `refund-reimbursement` 能力？

推进时，可参考的设计输入：原 `spec/refund-reimbursement-design.md`（验证规则、触发器、统计回溯规则、双向查询、边界情况）与已删除的 superpowers `2026-06-12-refund-reimbursement-ui-redesign-design.md`（UI 4 模式设计）。前者内容可在本文件上述"未实现的设计"段对照；后者已在 `docs/superpowers/` 清理时删除，如需复原见 git 历史 `docs/superpowers/specs/2026-06-12-refund-reimbursement-ui-redesign-design.md`。
