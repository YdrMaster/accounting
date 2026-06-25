## 1. 核心模型层 (accounting crate)

- [x] 1.1 新增 `ChannelPath` 模型：定义 `ChannelPath { id, transaction_id, position, channel_id, reconciled }` 结构体及对应 `ChannelPathId` 类型
- [x] 1.2 修改 `Transaction` 结构体：移除 `channel_id: Option<ChannelId>` 字段（链路信息通过 channel_paths 按 transaction_id 查询）
- [x] 1.3 新增 `ChannelPathNode` 值类型：用于 API/Service 层传递链路节点，包含 `position`、`channel_id`、`reconciled`，支持同一 position 多条记录（末端多项）
- [x] 1.4 修改 `Channel` 结构体：新增 `account_id: Option<AccountId>` 可选字段，建立渠道与资产账户的一对一关联
- [x] 1.5 修改 `TransactionFilter`：将 `channel_ids: Vec<ChannelId>` 保留，语义调整为"链路中包含指定渠道的交易"

## 2. 数据库 Schema (accounting-sql crate)

- [x] 2.1 新增 `channel_paths` 表 DDL：`id INTEGER PRIMARY KEY AUTOINCREMENT, transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE, position INTEGER NOT NULL, channel_id INTEGER NOT NULL REFERENCES channels(id), reconciled INTEGER NOT NULL DEFAULT 0`（注意：无 `UNIQUE(transaction_id, position)` 约束，以支持末端多项）
- [x] 2.2 新增索引：`idx_channel_paths_transaction_id ON channel_paths(transaction_id)`、`idx_channel_paths_channel_id ON channel_paths(channel_id)`
- [x] 2.3 修改 `transactions` 表 DDL：移除 `channel_id` 列（不保留历史数据，直接移除）
- [x] 2.4 修改 `channels` 表 DDL：新增 `account_id INTEGER REFERENCES accounts(id)` 可选列

## 3. Repository 层 (accounting-sql crate)

- [x] 3.1 新增 `channel_path` Repository 模块：实现 `channel_path_create`、`channel_path_list_by_transaction`、`channel_path_delete_by_transaction`、`channel_path_find_transactions_by_channel`、`channel_path_update_reconciled` 方法
- [x] 3.2 修改 `transaction` Repository：插入/查询交易时移除 `channel_id` 相关逻辑
- [x] 3.3 修改 `TransactionFilter` 查询构建：按渠道过滤改为通过 `channel_paths` JOIN 实现
- [x] 3.4 修改 `posting_sum_by_channel` 报表查询：通过 `channel_paths` JOIN 统计
- [x] 3.5 修改 `channel_count_transactions_by_id`：改为查询 `channel_paths` 表中引用该渠道的记录数
- [x] 3.6 修改 `channel` Repository：创建/查询/更新渠道时处理 `account_id` 字段

## 4. Service 层 (accounting-service crate)

- [x] 4.1 修改 `TransactionService`：创建交易时，在同一事务内先创建交易获取 transaction_id，再写入 channel_paths 记录（渠道存在性由 FK 约束保证），支持末端多项（同一 position 多条记录）
- [x] 4.2 修改 `TransactionService`：更新交易时，整体替换链路（按 transaction_id 删除旧 channel_paths，创建新的）
- [x] 4.3 修改 `TransactionService`：删除交易时，channel_paths 通过 `ON DELETE CASCADE` 自动清理，无需手动删除
- [x] 4.4 修改 `TransactionService`：查询交易时，一并返回完整渠道序列（含 reconciled 状态）
- [x] 4.5 新增对账标记功能：支持按链路节点（channel_path 记录 ID）标记 `reconciled` 状态
- [x] 4.6 修改 `ReportService`：`stats_by_channel` 适配 channel_paths 查询
- [x] 4.7 修改渠道删除校验：检查 channel_paths 中是否有引用，有则拒绝删除

## 5. API 层 (accounting-api crate)

- [x] 5.1 修改 `CreateTransactionRequest` DTO：将 `channel_id: Option<i64>` 替换为渠道序列结构，支持末端多项（同一 position 多个渠道）
- [x] 5.2 修改 `TransactionDto` DTO：返回渠道序列（含 reconciled 状态）替代单一渠道
- [x] 5.3 修改交易创建 Handler：接收渠道序列，在同一事务内创建交易和链路记录
- [x] 5.4 修改交易查询 Handler：返回完整渠道序列（含 reconciled 状态）
- [x] 5.5 修改交易更新 Handler：支持更新渠道序列
- [x] 5.6 新增对账标记 API：支持按链路节点标记/取消标记对账状态
- [x] 5.7 修改渠道删除 Handler：适配新的引用检查逻辑
- [x] 5.8 修改渠道相关 DTO：`CreateChannelRequest` 和 `ChannelDto` 增加 `account_id` 字段
- [x] 5.9 新增渠道更新 API：支持修改渠道的 `account_id` 关联

## 6. CLI 层 (accounting-cli crate)

- [x] 6.1 修改交易创建命令：支持 `--channel` 参数多次指定（有序），支持末端多项语法
- [x] 6.2 修改交易列表/详情命令：展示完整渠道序列及对账状态
- [x] 6.3 新增对账标记命令：支持标记/取消标记指定环节的对账状态
- [x] 6.4 修改渠道创建/列表命令：支持 `account_id` 参数
- [x] 6.5 修改报表按渠道统计命令：适配新的查询逻辑

## 7. 集成与验证

- [x] 7.1 编译全项目，修复所有类型错误和编译警告
- [x] 7.2 运行现有测试，修复因模型变更导致的失败用例
- [x] 7.3 新增测试：创建多节点链路交易、末端多项链路、按渠道检索交易、删除渠道引用检查、级联删除链路（CASCADE）
- [x] 7.4 新增测试：对账标记（标记/取消标记/查询未对账环节）
- [x] 7.5 新增测试：渠道关联账户（创建/更新/查询）
