## Why

本系统目前缺乏标准化的数据备份和迁移手段。数据存储在 SQLite 中，二进制格式不便于版本管理、人工审查和跨系统迁移。Beancount 是成熟的纯文本复式记账格式，生态工具丰富（fava 可视化、lint 校验等），且其账户路径（`:` 分隔）与本系统天然兼容。通过 beancount 格式实现导入导出，可以获得可读、可 diff、可迁移的数据备份能力。

## What Changes

- 新增 CLI 命令 `accounting-cli export` — 将数据库全量账目导出为 beancount 格式文件及附件目录
- 新增 CLI 命令 `accounting-cli import` — 从 beancount 格式文件导入账目到数据库，支持通过 `internal_id` metadata 重建内部引用关系
- 导出范围：Commodity、Account（含类型/账单日/还款日/关闭状态）、Transaction（含 Posting/冲减关系/渠道链路/成员/标签/附件）
- 不导出范围：AccountMapping、Budget 等配置数据（已有独立 YAML 导入导出机制）
- **BREAKING**: 移除 `Posting.description` 字段（误加的字段，涉及数据库 schema、repo 层、API 层）
- AccountType::Import 导出时映射到 `Equity:Import:xxx`，通过 `account_type` metadata 保留原始类型信息
- 附件以外部文件形式存储在 `attachments/` 子目录，通过 beancount `document` 指令引用

## Capabilities

### New Capabilities
- `beancount-export`: 将数据库账目导出为 beancount 文本文件及附件目录
- `beancount-import`: 从 beancount 文本文件导入账目到数据库，重建内部引用关系

### Modified Capabilities

## Impact

- **数据库**: 移除 `postings` 表的 `description` 列（schema 迁移）
- **核心库** (`accounting`): 移除 `Posting.description` 字段
- **SQL 层** (`accounting-sql`): 移除 posting repo 中 description 的读写
- **API 层** (`accounting-api`): 移除 posting DTO 中 description 相关字段
- **CLI 层** (`accounting-cli`): 新增 export/import 子命令
- **依赖**: 需引入 beancount 文本解析库（或手写解析器）
