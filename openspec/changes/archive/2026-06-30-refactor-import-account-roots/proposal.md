## Why

当前账单导入把无映射账户统一落到独立的 `Import/导入` 根账户下（如 `导入:支付宝:收支:餐饮美食`）。beancount 没有 `Import` 这一账户类型，导出时只能折中成 `Equity:Import:`，导致与标准 beancount 语义不兼容。为了让导入生成的账户直接符合 beancount 的五类账户模型，需要把导入 fallback 账户拆到 `Asset / Income / Expenses` 下，并移除 `Import` 根账户。

## What Changes

- **BREAKING** 移除 `Import/导入` 系统根账户及其 `AccountType::Import` 枚举值；数据库种子数据不再创建该根账户。
- **BREAKING** 无映射时的 fallback 账户路径改为：
  - 资产侧：`Assets:Import:<channel>:<category>`
  - 普通收支侧金额为正（支出）：`Expenses:Import:<channel>:<category>`
  - 普通收支侧金额为负（收入）：`Income:Import:<channel>:<category>`
  - 退款行：`Expenses:Import:<channel>:<category>`（金额为负，即“负的支出”）
- beancount 导出不再把 `Import` 映射到 `Equity:Import`，而是直接按 `Asset / Income / Expenses` 根账户输出。
- beancount 导入移除 `Equity:Import:` 到 `导入:` 的还原逻辑；旧的 `Equity:Import` 数据不再特殊处理。
- 删除 `ImportService` 中的 `resolve_import_root` 及 `ImportRootNotFound` 错误。

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `account-type-import`: 删除 Import 账户类型；该 spec 需要更新为“系统不再提供 Import 类型根账户”。
- `account-mapping`: 映射 key 前缀从 `收支:` / `资产:` 改为 `Assets:` / `Income:` / `Expenses:`（退款也使用 `Expenses:`）。
- `bill-import`: Import 根账户移除；fallback 账户路径按 `Asset / Income / Expenses` 构建；退款作为负支出归入 `Expenses:Import:<channel>:<category>`。
- `beancount-import`: 移除对 `Equity:Import:` 路径和 `account_type: Import` 的特殊处理。
- `beancount-export`: 移除 `Import → Equity:Import` 转换，按标准根账户类型导出。

## Impact

- `accounting/src/posting_role.rs`、`accounting/src/account_type.rs`：枚举与 key 前缀调整。
- `accounting-sql/src/schema.rs`：种子数据移除 `Import/导入` 根账户。
- `accounting-service/src/import_service.rs`、`accounting-service/src/mapping_service.rs`：映射 key 与 fallback 路径构建逻辑。
- `accounting-cli/src/cmd/import.rs`、`accounting-cli/src/cmd/mapping.rs`：错误与帮助文案更新。
- `accounting-beancount/src/export.rs`、`accounting-beancount/src/import.rs`：Import 类型特殊逻辑移除。
- 相关单元测试、集成测试、OpenSpec spec 文件同步更新。
