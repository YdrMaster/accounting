## Why

当前系统只能手动逐笔录入交易，无法从支付宝、微信、银行等渠道 App 导出的账单文件批量导入。真实场景中，月度账单动辄数百行，逐条手动录入不可行。需要一种机制将外部账单数据批量转换为系统内交易，同时避免污染用户定义的账户体系。

## What Changes

- 修改 `accounting-service` crate：新增 `import` 模块，包含账单适配器 trait（`BillAdapter`）、具体渠道适配器（首批支付宝 CSV）和 `ImportService` 编排逻辑，输入原始字节 + 补充上下文，输出 `Iterator<Item = BillEntry>`，编排适配器选择 → 账户自动创建 → 迭代写入 → 返回批次交易 ID 列表
- 新增 `Import` 系统根账户（`is_system=true`）：导入的交易 Posting 全部挂在 `Import:<来源>:<分类>` 下，不污染用户账户体系
- 新增 `待处理` 系统 Tag（`is_system=true`）：标记来自导入的未确认交易，用户修正后去除 tag 即可，无需专门的移动/合并操作
- 修改 `accounting-cli` crate：新增 `import` 子命令，接受文件路径、来源、成员等参数
- 修改 seed data 初始化：添加 `Import` 根账户和 `待处理` 系统 Tag

## Capabilities

### New Capabilities
- `bill-import`: 账单导入功能——从外部渠道 App 导出的账单文件批量导入交易，通过适配器模式支持多渠道，导入的交易使用 Import 根账户隔离并标记待处理 Tag

### Modified Capabilities
- `transaction-filter`: 新增按"待处理"Tag 筛选交易的能力（复用现有 tag_ids 过滤）

## Impact

- **核心模型层**（accounting crate）：无变更
- **数据库层**（accounting-sql crate）：新增 `Import` 根账户和 `待处理` 系统 Tag 的 seed data
- **Service 层**（accounting-service crate）：新增 `import` 模块（含 `BillAdapter` trait、适配器实现、`ImportService`），依赖 `accounting` + `accounting-sql`
- **CLI 层**（accounting-cli crate）：新增 `import` 子命令