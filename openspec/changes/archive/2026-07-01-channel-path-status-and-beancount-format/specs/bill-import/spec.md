## MODIFIED Requirements

### Requirement: ImportService 编排导入流程
系统 SHALL 在 `accounting-service` crate 中由 `ImportService` 编排完整导入流程：确保待处理 Tag 存在 → 选择适配器 → 调用适配器解析 → 迭代 BillEntry → 对每个 BillPosting 查询账户映射表决定目标账户（有映射用映射账户，无映射走 `Assets:Import:<channel>` / `Income:Import:<channel>` / `Expenses:Import:<channel>` fallback）→ 调用 `TransactionService::submit` → 收集 TransactionId → 返回批次结果。

#### Scenario: 第三方导入交易渠道状态默认为 pending
- **当** 用户通过 `import` 命令从支付宝、微信等第三方渠道导入交易
- **那么** 每笔交易的 channel_path 中所有渠道的 status 必须为 `pending`

#### Scenario: 多资产 Posting 的渠道链路
- **当** 支付宝适配器解析一笔交易产生多个 `role=Asset` 的 BillPosting
- **那么** ImportService 构造的 channel_path 只包含 ImportContext 中的单一渠道，status=pending，不包含各个资产账户作为渠道

## REMOVED Requirements

### Requirement: 第三方导入交易渠道状态为未对账
系统 SHALL 在导入时将 channel_path 中所有渠道的 `reconciled` 标记为 `false`。

**Reason**: 渠道状态已扩展为 `default` / `pending` / `verified` 三种，第三方导入数据需要人工校验，统一使用 `pending`。

**Migration**: 新导入的交易使用 `pending`；历史已导入且 reconciled=false 的数据迁移为 `default`，不影响新导入语义。
