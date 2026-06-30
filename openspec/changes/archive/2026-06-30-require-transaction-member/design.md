## 背景

当前 `Transaction` 结构体将记账人保存为 `member_id: Option<MemberId>`，SQLite schema 也允许 `transactions.member_id` 为 `NULL`。因此 CLI `tx add` 可以接受没有记账人的交易，API DTO 也反映了这个可选字段。业务规则要求每笔交易都必须有记账人，所以该字段应该从端到端改为必填。

## 目标 / 非目标

**目标：**
- 在数据库 schema 中将 `member_id` 强制为 `NOT NULL`。
- 在核心 `Transaction` 模型中将 `member_id` 表示为非可选的 `MemberId`。
- 要求 `accounting-cli tx add` 必须传入 `--member`。
- 当 `accounting-cli tx update` 未传入 `--member` 时，保留原交易的 `member_id`。
- 拒绝缺少 `member` metadata 或引用未知成员的 Beancount 交易。
- 保持项目可编译，包括对 API handler/DTO 的最小化调整。
- 更新所有构造交易的测试和 fixture。

**非目标：**
- 迁移已有数据库中包含 `NULL` `member_id` 的历史数据（按用户指示不在本次范围内）。
- 新增 Web 交易创建页面或修改 Web 展示逻辑（仅做类型同步）。
- 将字段重命名为 `creator_id` 或 `bookkeeper_id`；保留现有 `member_id` 名称。

## 决策

### 1. 在数据库层和模型层同时强制
- **决策**：将 `transactions.member_id` 改为 `NOT NULL`，并将 `Transaction.member_id` 改为 `MemberId`。
- **理由**：数据库约束是最强的保证；非可选的模型类型则让“无记账人”这种无效状态在内存中无法被构造。两者结合最符合“让非法状态不可表示”的原则。

### 2. API 最小化改动
- **决策**：将 `CreateTransactionRequest.member_id` 从 `Option<i64>` 改为 `i64`，并在 handler 中直接 `MemberId(req.member_id)`。
- **理由**：模型改变后 API 必须继续编译。用户表示 API/Web 功能不是重点，但代码不能处于编译失败状态。保持改动最小可以避免范围蔓延。

### 3. CLI `tx update` 保留原成员
- **决策**：`tx update` 未传 `--member` 时，先读取原交易并复用其 `member_id`。
- **理由**：`tx update` 是全量替换。要求用户每次更新都重新指定成员既繁琐又容易出错；而静默设为 `NULL` 会违反新的约束。

### 4. Beancount 导入对缺失/未知成员报错
- **决策**：在 `accounting-beancount/src/import.rs` 中，若 `tx.member` 为 `None` 或引用的名字不在已导入成员列表中，则返回错误。
- **理由**：Beancount 文件应是从本系统导出的自包含数据，每笔交易都应带有成员。快速失败可以暴露坏输入，而不是生成无效数据。

## 风险 / 权衡

- **【风险】** 大量现有测试使用 `Transaction { member_id: None, ... }` 构造交易。
  - **缓解**：更新所有此类测试，使其创建或引用真实成员。
- **【风险】** 修改核心 `Transaction` 结构体后，所有对 `member_id` 为 `Option` 进行模式匹配的下游代码都会失效。
  - **缓解**：项目是单体仓库，所有调用方在同一变更中同步更新。
- **【风险】** 若对已存在 `NULL` `member_id` 的生产数据库执行 schema 变更，会失败。
  - **缓解**：按用户指示不在本次范围内处理；新部署或历史数据迁移需单独处理。

## 迁移计划

1. 更新新数据库的 schema 定义。
2. 更新核心模型、SQL repo 和服务代码。
3. 更新 CLI 命令和 Beancount 导入/导出。
4. 应用 API 最小编译修复。
5. 更新所有测试和 fixture。
6. 运行完整测试套件。

本次不包含对已有 `NULL` 行的显式迁移。

## 待解决问题

_无_
