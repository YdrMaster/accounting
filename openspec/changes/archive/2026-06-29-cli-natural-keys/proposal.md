## Why

当前 CLI 大量命令要求用户传入数据库内部 ID（如 `member delete <ID>`、`budget show <ID>`、`import --member <ID>`）。这些 ID 对用户不友好，需要查询后才能使用，容易出错。用户日常操作应使用人类可读的自然键（名称、路径、符号），只有交易/分录这类没有稳定自然键的实体才保留 ID。

## What Changes

- **BREAKING** 将 CLI 中所有实体定位参数从数据库 ID 改为自然键：
  - `member delete <NAME>`（`members.name` 加 `UNIQUE`）
  - `account show/close/reopen/balance <PATH>`，如 `Assets:Cash`
  - `account add <PATH>` 移除 `--parent-id`，路径已隐含父级
  - `tx add --member <NAME> --channel <PATH_EXPR>`
  - `tx list --account <PATH> --member <NAME> --channel <NAME>`
  - `mapping set/list/delete --member <NAME>`
  - `budget show/update/delete <NAME>`（`budgets.name` 加 `UNIQUE`）
  - `budget create/update --commodity <SYMBOL>`
  - `import --member <NAME>`
  - `report cashflow --commodity <SYMBOL>`
- **BREAKING** 重做 `tx add --channel` 语法：移除隐式位置和 `:` 分隔，改为 `->` 链式语法，末级多个渠道用 `&` 分隔。例如 `淘宝->支付宝->花呗&建行卡`。
- 在 `accounting-cli` 中新增统一的 natural key resolver 模块，集中处理名称/路径/符号到 ID 的解析，并提供清晰的人类可读错误。
- 在 schema 层为 `members.name` 和 `budgets.name` 增加 `UNIQUE` 约束，确保自然键唯一。
- 在 SQL/repo/service 层补充 `member_get_by_name` 和 `budget_get_by_name`。
- 交易/分录自身的 `tx show/delete/update` 和 `tx reconcile <PATH_ID>` 继续保留 ID（无自然键）。
- `list`/`show` 输出继续显示 ID 列，方便脚本和交易相关操作引用，但用户不再需要记忆或输入它们。
- 更新 `accounting-cli/README.md` 和 `plan/cli-design.md` 中的命令示例。

## Capabilities

### New Capabilities

- `cli-natural-key-resolution`: 定义 CLI 使用自然键解析为内部 ID 的通用规则，包括解析失败/重复/不存在时的错误格式，以及渠道名称中禁止出现 `->` 和 `&` 等分隔符的约束。

### Modified Capabilities

- `channel-path`: 更新 CLI 创建/更新交易链路时的渠道序列语法，从 `:` 分隔的位置/ID 模式改为 `->` 链式 + 末级 `&` 分隔模式。
- `budget-cli`: `budget show/update/delete` 参数从 budget_id 改为预算表名称；`budget create/update --commodity` 从币种 ID 改为币种符号。
- `bill-import`: CLI `import --member` 参数从成员 ID 改为成员名称。
- `account-mapping`: CLI `mapping set/list/delete` 的 `--member` 参数从成员 ID 改为成员名称。

## Impact

- **accounting-cli**: 大量命令参数类型和解析逻辑变更，新增 `cmd/resolver.rs`。
- **accounting-sql**: schema 新增两处 `UNIQUE`；需要处理已有数据库中重复 `members.name` 或 `budgets.name` 的迁移/报错；新增 `member_get_by_name`、`budget_get_by_name` repo 函数。
- **accounting-service**: 在 `MemberService` / `BudgetService` 暴露按名称查询接口。
- **文档**: `accounting-cli/README.md`、`plan/cli-design.md` 需要同步更新。
- **用户体验**: 命令更直观，但旧脚本会失效（彻底替换，无 ID fallback）。
