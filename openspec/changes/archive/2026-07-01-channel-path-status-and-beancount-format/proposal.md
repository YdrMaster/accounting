## Why

当前 beancount 导出存在两处体验问题：commodity 指令日期硬编码为 `1970-01-01`，无法反映真实创建时间；`channel_path` 使用 JSON 存储，既不易读也无法在 beancount 文件中直接人工编辑。同时，渠道链路只有“已对账/未对账”两种状态，无法区分“从第三方导入待校验”与“默认/已确认”的场景。

## What Changes

- **BREAKING** 将 `channel_paths.reconciled` 布尔字段替换为 `status` 三态字段：`default`（默认）、`pending`（待校验）、`verified`（已校验）。
- 第三方渠道（如支付宝）导入的交易链路默认状态为 `pending`。
- CLI `tx add` / `tx update` 和 beancount 导入支持在渠道名后加 `*` 表示 `pending`，加 `√` 表示 `verified`，无后缀表示 `default`。
- beancount 导出时 `channel_path` metadata 改用 CLI 的 `->` / `&` 文本格式，并附加 `*` / `√` 后缀表达状态。
- beancount 导入时解析上述文本格式的 `channel_path`。
- commodity 导出时使用数据库中的 `created_at` 作为 `commodity` 指令日期，不再固定 `1970-01-01`。
- `tx reconcile` 命令语义调整为将链路节点标记为 `verified`。

## Capabilities

### New Capabilities

（无全新能力，均为现有能力增强）

### Modified Capabilities

- `channel-path`: 数据模型从 `reconciled: bool` 改为 `status` 三态；CLI 语法新增 `*` / `√` 后缀；第三方导入默认 `pending`。
- `beancount-export`: `channel_path` metadata 输出为 CLI 文本格式；commodity 日期使用 `created_at`。
- `beancount-import`: 解析 CLI 文本格式的 `channel_path`；根据后缀设置链路状态。
- `bill-import`: 支付宝等第三方适配器导入的 channel_paths 状态设为 `pending`。
- `transaction-api`: TransactionDto / PostingDto 中链路节点字段从 `reconciled: bool` 改为 `status: String`。

## Impact

- 数据库 schema：`channel_paths` 表字段变更，需要迁移旧数据。
- `accounting` core types：`ChannelPath` / `ChannelPathNode` 新增 `ChannelPathStatus` 枚举。
- `accounting-sql` repo：所有读写 `channel_paths` 的查询更新。
- `accounting-service`：ImportService、TransactionService 的链路构造与对账接口更新。
- `accounting-api`：DTO 与 `build_account_type_map` 之外的链路字段序列化更新。
- `accounting-cli`：`tx add` / `tx update` 解析器支持后缀；`tx reconcile` 语义调整；输出表格显示状态。
- `accounting-beancount`：generator 与 parser 的 `channel_path` 格式变更，commodity 日期来源变更。
- `accounting-web`：前端链路展示从 `reconciled` 改为 `status`。
