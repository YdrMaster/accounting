## Context

当前 `accounting-cli` 的命令参数大量依赖数据库内部 ID。用户在命令行中必须先 `list` 出 ID，再复制到下一个命令，体验不自然。例如 `member delete 3`、`budget show 2`、`import --member 1`。

同时，`tx add --channel` 的现有语法基于隐式位置和 `:` 分隔符，既不易读也容易与账户路径的 `:` 混淆。用户希望用更直观的 `->` 链式语法表达资金流向，末级多个渠道用 `&` 分隔。

本设计将 CLI 的实体定位参数统一改为自然键，同时为交易链路引入新的字符串语法。

## Goals / Non-Goals

**Goals:**
- 用户可以使用人类可读的自然键操作 CLI：`成员名`、`账户路径`、`渠道名`、`币种符号`、`预算名`。
- 统一自然键解析逻辑，避免每个命令重复写查找代码。
- 将 `tx add --channel` 的语法改为 `->` 链式 + 末级 `&` 分隔。
- 保证 schema 层自然键唯一（member、budget），从数据库层面消除歧义。
- 更新相关文档和计划文件。

**Non-Goals:**
- 不改动交易/分录自身的 `tx show/delete/update` 和 `tx reconcile`（这些实体没有稳定自然键）。
- 不改动 API/Web 层的接口（本次只改 CLI）。
- 不引入交互式选择器或模糊匹配（自然键必须精确命中）。
- 不为 ID 保留 fallback 参数（彻底替换）。

## Decisions

### Decision: 新增 `cmd/resolver.rs` 统一解析自然键
**Rationale**: 目前各命令自己调用 SQL 查询或构造 ID，重复且错误信息不一致。集中到一个 resolver 模块后，所有命令共享同一套“不存在/不唯一/空值”的错误处理。

解析器职责：
- `resolve_member(db, name)` → 查询 `members` 按 `name`，返回 `MemberId`。
- `resolve_account(db, path)` → 调用 `account_get_by_name`，返回 `AccountId`。
- `resolve_channel(db, name)` → 查询 `channels` 按 `name`，返回 `ChannelId`。
- `resolve_commodity(db, symbol)` → 调用 `commodity_get_by_symbol`，返回 `CommodityId`。
- `resolve_budget(db, name)` → 查询 `budgets` 按 `name`，返回 `BudgetId`。

每个函数返回的错误文本形如 `成员 'Alice' 不存在`，不暴露内部 ID。

### Decision: `members.name` 和 `budgets.name` 加 `UNIQUE`
**Rationale**: 只有保证唯一，名称才能作为稳定的定位键。否则用户写 `member delete Alice` 时若存在两个 Alice，行为不可预期。

实现方式：
- 在 `accounting-sql/src/schema.rs` 的建表语句中为 `members.name` 和 `budgets.name` 增加 `UNIQUE`。
- 在 repo 层新增 `member_get_by_name`、`budget_get_by_name`。
- 在 service 层暴露按名称查询接口供 CLI resolver 使用。

### Decision: 账户操作统一使用完整路径
**Rationale**: 账户天然是树形结构，`UNIQUE(parent_id, name)` 已经保证完整路径唯一。用户写 `Assets:Cash` 比记 ID 直观得多。

具体命令变化：
- `account show/close/reopen/balance <PATH>`
- `account add <PATH>` 使用 `AccountService::create_cascading`，自动按需创建中间父级，移除 `--parent-id`。
- `tx list --account <PATH>`
- `budget --limit <PATH>:<AMOUNT>` 已使用路径，保持不变。

### Decision: `tx add --channel` 语法改为 `->` 链式 + 末级 `&`
**Rationale**: `->` 直观表示资金流向；`&` 在 shell 中需要引号，但与账单导入里“收/付款方式”用 `&` 拆分多个支付方式的语义一致，末级多个渠道表示并行支付方式。

解析逻辑：
1. 用正则 `\s*->\s*` 将输入切分为 segment。
2. 最后一个 segment 再用 `\s*&\s*` 切分为多个 channel name。
3. 其它 segment 只能有一个 channel name。
4. 每个 name trim 后检查非空，并不包含 `->` 或 `&`。
5. 按 segment 索引生成 `position`；末级多个 name 共享同一最大 `position`。

示例：
- `支付宝` → `[(0, 支付宝)]`
- `淘宝 -> 支付宝 -> 建行卡` → `[(0, 淘宝), (1, 支付宝), (2, 建行卡)]`
- `淘宝 -> 支付宝 -> 花呗 & 建行卡` → `[(0, 淘宝), (1, 支付宝), (2, 花呗), (2, 建行卡)]`

### Decision: 输出继续显示 ID
**Rationale**: 虽然用户不需要输入 ID，但交易相关操作（`tx show/delete/update`、`tx reconcile`）仍依赖 ID。list/show 输出保留 ID 列可以让用户复制粘贴进行这些操作，也方便 JSON 消费和调试。

## Risks / Trade-offs

- **[Risk] 现有数据库存在重复 member/budget 名称，加 UNIQUE 会失败。**
  → **Mitigation**: schema 初始化时加 UNIQUE；对于已存在的数据库，在启动或初始化阶段检测重复并给出明确错误，提示用户手动合并或重命名。不自动静默合并，避免数据丢失。

- **[Risk] 彻底替换 ID 参数会破坏现有脚本。**
  → **Mitigation**: 这是有意的 BREAKING change。在 proposal 和 changelog 中明确标注，无 ID fallback。

- **[Risk] 渠道名禁止 `->`/`&` 可能与用户已有渠道名冲突。**
  → **Mitigation**: 该限制只影响新增/修改渠道；已有非法名称的渠道在解析时可能失败，需要在迁移或导入时处理。可在 channel 创建/更新时增加校验并给出清晰错误。

- **[Risk] 账户路径和 posting 格式都使用 `:`，新手容易混淆。**
  → **Mitigation**: 保持现状，`:` 作为账户层级分隔符已经是 Beancount 风格用户的常识； posting 中用 `:` 分隔账户路径和商品/金额，二者不会冲突。

- **[Risk] `&` 在 shell 中是后台运行符，用户容易忘记加引号。**
  → **Mitigation**: 文档和示例中始终展示带引号的写法；解析器在收到单个 token 时也能工作，因为 CLI 会把它当做一个参数。

## Migration Plan

1. **Schema 变更**: 更新 `accounting-sql/src/schema.rs`，给 `members.name` 和 `budgets.name` 加 `UNIQUE`。
2. **已有数据检查**: 在 `initialize` 或首次启动时查询是否存在重复名称，若存在则报错并列出重复项，要求用户处理。
3. **Repo/Service 新增**: 实现 `member_get_by_name`、`budget_get_by_name` 及对应 service 接口。
4. **CLI Resolver**: 新增 `cmd/resolver.rs`。
5. **CLI 命令替换**: 按 proposal 列表逐个修改命令参数和解析逻辑。
6. **Channel path 解析器重写**: 替换 `tx.rs` 中的 `parse_channel_paths`。
7. **文档更新**: 更新 `accounting-cli/README.md` 和 `plan/cli-design.md`。
8. **测试**: 新增 resolver、channel path、重复名检测等测试。

Rollback: 由于这是代码层面的接口变更，回滚即恢复旧版本参数。数据层的 UNIQUE 约束一旦加上不会回滚，但不影响旧代码运行（旧代码只是不再使用这些命令）。

## Open Questions

- 是否需要为 `member rename` / `budget rename` 提供命令？如果自然键是名称，rename 会影响脚本，但当前似乎尚未暴露 rename 功能。
- `tx list --channel` 过滤语义是“链路中包含该渠道”还是“链路首节点是该渠道”？目前按 channel_id 过滤的实现是包含任意位置，建议保持此语义。
- 是否需要在输出中将 `channel_id` 替换为 `channel_name`（`tx show` 的链路表格）？从“用户不需要知道 ID”的角度，建议同步替换。
