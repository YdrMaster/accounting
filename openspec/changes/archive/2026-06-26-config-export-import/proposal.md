## Why

用户需要一种机制，把数据库中除账本（交易与分录）以外的所有配置信息导出到人类可读、可写的文本文件中，并在新数据库或已有数据库中重新导入。这便于备份账户树、渠道、标签、预算、导入映射规则等元数据，也方便在不同环境之间迁移配置，而不涉及交易流水本身。

## What Changes

- 新增配置导出能力：将数据库中的 `settings`、`commodities`、`members`、`channels`、`tags`、`accounts`、`account_owners`、`account_mappings`、`budgets` 及 `budget_limits` 导出为单个 YAML 文件。
- 新增配置导入能力：读取单个 YAML 文件，按自然键合并更新到目标数据库。
- 新增 CLI 命令：`accounting config export <file>` 与 `accounting config import <file>`。
- 在 `accounting-service` 中新增 `ConfigService`，封装导出/导入业务逻辑。
- 在 `accounting-sql` 中补齐配置相关表的查询与按自然键创建/更新能力。
- 不导出也不导入 `transactions`、`postings`、`channel_paths`、`attachments`、`transaction_tags` 等账本数据。
- 不导出数据库自增 ID，所有引用均使用自然键（如账户路径、币种 symbol、成员/渠道/标签名称）。
- 不处理 `is_system` 标记，依赖目标数据库已完成初始化并包含系统内置数据。

## Capabilities

### New Capabilities

- `config-export-import`: 配置数据的 YAML 导出与导入，包括 settings、commodities、members、channels、tags、accounts、account_owners、account_mappings、budgets。

### Modified Capabilities

- 无。本变更仅新增能力，不改变现有功能的需求规格。

## Impact

- **CLI**: `accounting-cli` 新增 `config` 子命令。
- **Service**: `accounting-service` 新增 `ConfigService` 及 YAML DTO 类型。
- **SQL**: `accounting-sql` 的各配置 repo 需补充 list/upsert 方法。
- **Dependencies**: workspace 新增 YAML 序列化依赖（如 `serde_yaml` 或 `yaml-rust2`）。
- **API/UI**: 本次不涉及，后续可扩展。
