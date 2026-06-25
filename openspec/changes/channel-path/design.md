## Context

当前系统中，交易（Transaction）通过 `channel_id` 关联单个渠道（Channel），渠道仅包含 `id`、`name`、`description` 三个字段。这种扁平设计无法表达真实交易中的多环节链路，例如"淘宝→支付宝→花呗"，其中每个环节对应不同的账单来源，需要支持交叉对账以保证记账准确性。

项目技术栈：Rust (Edition 2024)，SQLite (sqlx)，axum Web 框架，clap CLI。数据模型基于 Kleppmann 端点图模型，每笔交易由多个 Posting（分录）组成。数据库层不需要保留历史数据，因此无需数据迁移脚本。

现有相关文件：
- 核心模型：`accounting/src/channel.rs`、`accounting/src/transaction.rs`
- 数据库 Schema：`accounting-sql/src/schema.rs`
- 渠道 Repository：`accounting-sql/src/repo/channel.rs`
- 交易 Repository：`accounting-sql/src/repo/transaction.rs`
- 报表统计：`accounting-service/src/report_service.rs`、`accounting-sql/src/repo/posting.rs`
- API Handler：`accounting-api/src/handlers/channel.rs`、`accounting-api/src/handlers/transaction.rs`
- DTO：`accounting-api/src/dto.rs`
- CLI：`accounting-cli/src/cmd/tx.rs`

## Goals / Non-Goals

**Goals:**
- 支持为交易设置可变长度的有序渠道序列（交易链）
- 支持交易链末端多项（同一最大 position 可有多条记录，如"淘宝→支付宝→(花呗+信用卡)"）
- 支持逐环节对账标记（每个链路节点可独立标记是否已对账）
- 渠道保持一等实体，支持按渠道检索交易（对账场景）
- 渠道可直接关联资产账户（可选一对一）
- 交易链不做去重聚合，每笔交易独立存储自己的链路

**Non-Goals:**
- 链路不作为一等实体（无独立表、无链路元数据）
- 不做链路去重或聚合优化
- 不做链路级统计报表（统计可离线做）
- 不为渠道添加类型标签（如"平台"/"渠道"/"方式"），位置仅表示顺序
- 不做数据迁移（数据库层不保留历史数据）

## Decisions

### D1: 存储方案 — 有序数组表（channel_paths）而非 JSON 字段或链表

**选择**：新增 `channel_paths` 表，使用 `(transaction_id, position, channel_id, reconciled)` 四字段存储有序渠道序列，直接以 `transaction_id` 关联交易。

**替代方案**：
- JSON 字段（`transactions.channel_path = '[5,8,12]'`）：零新增表，但无法使用 FK 约束，查询需 `json_each()` 全表扫描，完整性无保证，且无法为每个节点附加 `reconciled` 标记。
- 链表（`id, channel_id, next_id`）：递归 CTE 查询性能差，且链路是线性结构不需要链表的分叉能力，over-engineering。

**理由**：有序数组表在 SQLite 中查询最友好——按 transaction_id 一次查询+排序即得完整链路，按 channel_id 直接索引扫描找到所有相关交易。`transaction_id` 和 `channel_id` 均可声明 FK 约束保证引用完整性。每行记录可独立携带 `reconciled` 字段。

### D2: 直接用 transaction_id 关联，无需 chain_id

**选择**：`channel_paths` 表直接使用 `transaction_id REFERENCES transactions(id)` 关联交易，不引入中间的 `chain_id`。

**替代方案**：
- 使用 `chain_id` 作为分组标识，`transactions.chain_id` 引用 `channel_paths.chain_id`：需要额外生成 chain_id，且 chain_id 不是独立表的主键，无法声明 FK 约束，需应用层保证一致性，并发场景下 chain_id 生成有竞争风险。

**理由**：链路不是一等实体，每笔交易的链路记录天然属于该交易，用 `transaction_id` 直接关联是最自然的方式。优势：
- `transaction_id` 可以声明 `REFERENCES transactions(id) ON DELETE CASCADE`，删除交易时自动清理链路记录
- 无需 chain_id 生成逻辑，消除并发竞争问题
- 查询更直观：`WHERE transaction_id = ?` 而非先查找 chain_id 再查询
- `transactions` 表无需新增任何字段（移除 `channel_id` 即可）

### D3: 末端多项 — 允许同一 position 多条记录

**选择**：无 `UNIQUE(transaction_id, position)` 约束，允许同一 transaction_id 下同一 position 有多条记录。链路的末端定义为：position 值等于该 transaction_id 下最大 position 值的所有记录。

**替代方案**：
- 维持 UNIQUE 约束，末端不同渠道使用递增 position（如花呗 position=2，信用卡 position=3）。但这改变了语义——它们在链路中是并行关系而非顺序关系。

**理由**：交易如"淘宝→支付宝→(花呗+信用卡)"，花呗和信用卡都是末端，语义上是并行的（同一笔交易从支付宝同时扣了花呗和信用卡），共享同一个 position 值更准确地表达这一关系。

### D4: 对账标记 — channel_paths 表增加 reconciled 字段

**选择**：`channel_paths` 表新增 `reconciled INTEGER NOT NULL DEFAULT 0` 布尔字段（SQLite 无原生 bool，用 0/1 表示），标记该环节是否已完成对账。

**理由**：交易链的每个环节对应一个账单来源（如淘宝账单、支付宝账单、花呗账单），逐环节标记对账状态是交叉对账的核心需求。将对账状态放在 channel_paths 而非 transaction 上，因为同一笔交易的不同环节可能在不同时间完成对账。

### D5: 渠道-账户关联 — channels 表增加可选 account_id

**选择**：`channels` 表新增 `account_id INTEGER REFERENCES accounts(id)` 可选字段（允许 NULL），建立渠道与资产账户的直接一对一关联。

**替代方案**：
- 中间表 `channel_accounts`（多对多）：过于灵活，当前需求是一对一。
- 不在渠道上关联，仅通过 Posting 的 account_id 间接体现：用户需要手动从 Posting 推断渠道与账户的对应关系，不够直观。

**理由**：很多渠道天然对应一个资产账户（如"花呗"对应 Assets:花呗，"信用卡"对应 Assets:信用卡），可选的一对一关联既满足常见场景，又不强制要求所有渠道都关联账户。

### D6: 删除渠道时的引用检查

**选择**：删除渠道时，通过 `channel_paths` 表检查是否有交易链路引用该渠道，若有则拒绝删除。

**理由**：与现有 `channel_force_delete_by_id` 的模式一致，保证数据完整性。

### D7: 数据库变更 — 直接替换，不保留历史数据

**选择**：直接从 `transactions` 表移除 `channel_id` 列，新增 `channel_paths` 表以 `transaction_id` 关联交易。不编写数据迁移脚本。同时修改 `channels` 表结构（新增 `account_id`）。

**理由**：数据库层不需要保留历史数据，可以直接重建 schema。

### D8: 报表查询适配

**选择**：现有的 `posting_sum_by_channel` 查询改为通过 `channel_paths` JOIN，支持按链路中任意位置的渠道进行统计。

**理由**：对账场景需要"所有经过渠道 X 的交易"，不限于链路首节点。

## Risks / Trade-offs

- **[末端多项的约束放宽]** → 无 `UNIQUE(transaction_id, position)` 约束后，同一 transaction_id + position 可有多条记录，需要通过应用层确保 position 值合理（非末端 position 不应有多条）。数据库层不做此约束，应用层负责。
- **[reconciled 标记的初始值]** → 创建链路节点时 `reconciled` 默认为 0（未对账），用户可后续逐个标记为已对账。
- **[channel_paths 与 transactions 的写入顺序]** → 需先创建交易获得 transaction_id，再写入 channel_paths 记录。应在同一事务内完成以保证原子性。
