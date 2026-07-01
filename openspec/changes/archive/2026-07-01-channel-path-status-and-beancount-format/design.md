## Context

当前系统：

- `channel_paths` 表用 `reconciled: INTEGER`（0/1）记录环节是否已对账。
- beancount 导出时把 `channel_paths` 序列化为 JSON metadata：`channel_path: '[{"position":0,"channel":"支付宝","reconciled":true}]'`。
- beancount 导出的 `commodity` 指令日期硬编码为 `1970-01-01`。
- `tx add --channel` 使用 `->` 表示链路、`&` 表示末级并行，例如 `淘宝 -> 支付宝 -> 花呗 & 建行卡`。

用户期望：

1. 从数据库导出的 commodity 日期应是数据库中的 `created_at`。
2. `channel_path` metadata 应复用 CLI 的 `->` / `&` 语法，便于人工阅读和编辑。
3. 链路状态扩展为三档：默认、待校验、已校验；第三方导入为待校验，CLI/beancount 可用后缀 `*` / `√` 控制。

## Goals / Non-Goals

**Goals:**

- 将 `channel_paths` 的 `reconciled` 改为 `status` 三态：`default`、`pending`、`verified`。
- 第三方导入（支付宝等）生成的 channel_paths 状态为 `pending`。
- CLI 和 beancount 支持用 `*` / `√` 后缀显式设置 `pending` / `verified`，无后缀为 `default`。
- beancount 导入/导出使用文本格式 `channel_path` metadata，兼容 `->` / `&` / `*` / `√`。
- commodity 导出日期使用 `commodities.created_at`。

**Non-Goals:**

- 不改 commodity 的 `created_at` 生成逻辑（仍由数据库默认填充）。
- 不修改 `channel_paths` 的位置、外键、级联等已有结构。
- 不引入新的 beancount directive，仅在 metadata 层面表达状态。
- 不要求旧数据能完美区分“第三方导入的 pending”与“CLI 创建的 default”；迁移规则统一处理。

## Decisions

### 1. 用整数枚举存储状态

- **选择**：`channel_paths.status INTEGER NOT NULL DEFAULT 0`，对应 `0=default`、`1=pending`、`2=verified`。
- **理由**：SQLite 整数枚举查询高效，与现有 `reconciled INTEGER` 风格一致；对外 Rust 层使用 `ChannelPathStatus` 枚举保证类型安全。
- **替代**：用 TEXT 存储字符串。更易读但占用更大、索引效率略低；当前规模下差异可忽略，但 INTEGER 与现有代码风格更一致。

### 2. 链路状态语义

- `default`：普通链路，无特殊校验含义（对应原 `reconciled=false` 中由 CLI/手动创建的部分）。
- `pending`：需要人工校验，主要来自第三方导入。
- `verified`：已确认无误（对应原 `reconciled=true`）。

`tx reconcile` 命令从“切换对账标记”改为“将节点标记为 `verified`”；提供 `--unset` 时可回到 `default`。

### 3. CLI / beancount 文本语法

在原有 `->` / `&` 语法基础上，允许每个渠道名后 optionally 加 `*` 或 `√`（紧跟名称，中间无空格）：

```
淘宝 -> 支付宝* -> 花呗&建行卡√
```

解析规则：

- 从末尾剥离一个后缀字符，若是 `*` 则状态 `pending`，若是 `√` 则 `verified`，否则 `default`。
- 渠道名本身不得包含 `*` 或 `√`；验证渠道存在时按剥离后的名称查找。
- `&` 仅允许在最后一级使用，与现有规则一致。

### 4. beancount 导入与导出的状态映射

- 导出：`default` 不加后缀，`pending` 加 `*`，`verified` 加 `√`。
- 导入：解析后缀后按上述规则映射回 `ChannelPathStatus`。
- 向后兼容：导入旧备份中的 JSON `channel_path` 时，仍尝试解析；`reconciled=true` 映射为 `verified`，`reconciled=false` 映射为 `default`。

### 5. 第三方导入默认 pending

- `ImportService` 在构造 `ChannelPathNode` 时显式设置 `status = pending`。
- 支付宝适配器本身不感知状态，状态由 `ImportService` 统一决定，便于未来其他第三方来源复用同一规则。

### 6. Commodity 日期来源

- `BCommodity` 增加 `created_at: Option<NaiveDate>`。
- 导出时通过 `commodity_created_at_map`（或扩展 `commodity_list`）获取每个 commodity 的 `created_at`。
- 若 `created_at` 缺失，回退到 `1970-01-01` 以保持现有测试不变，同时记录警告或测试覆盖缺失场景。

### 7. 数据迁移

- 迁移脚本将 `reconciled = 1` 的行更新为 `status = 2`（verified）。
- `reconciled = 0` 的行更新为 `status = 0`（default）。
- 这意味着旧第三方导入数据会变为 `default` 而非 `pending`，这是可接受的，因为旧数据无法区分来源；新导入才会标记 `pending`。

## Risks / Trade-offs

- **[Risk] 旧 beancount 备份中的 JSON channel_path 可能包含 `reconciled`，需要兼容解析。** → Mitigation：parser 先尝试新文本格式，失败再回退 JSON；JSON 中 `reconciled=true` → `verified`，`false` → `default`。
- **[Risk] 渠道名中已存在带 `*` 或 `√` 的字符导致解析歧义。** → Mitigation：channel name validation 已禁止 `->` 和 `&`，本次继续禁止 `*` 和 `√`。
- **[Risk] 前端、CLI、API 多处依赖 `reconciled` 布尔字段，迁移工作量大。** → Mitigation：按 proposal 中列出的模块分任务逐步替换；保持 `verified` 与旧 `reconciled=true` 在展示上可互换。
- **[Risk] 旧 `tx reconcile` 行为是 toggle，改为 set-to-verified 可能让用户困惑。** → Mitigation：CLI help 明确说明；必要时保留 toggle 语义（当前节点已是 verified 则降为 default）。

## Migration Plan

1. 添加 `channel_paths.status` 列并填充（SQLite `ALTER TABLE` + `UPDATE`）。
2. 删除 `channel_paths.reconciled` 列（或在应用层忽略）。
3. 更新 Rust 枚举、repo 函数、service 调用、API DTO、CLI 解析、beancount generator/parser。
4. 更新相关测试与 spec。
5. 验证：导入支付宝 CSV → 检查 channel_paths 状态为 pending；CLI 设置 `√` 后导出 → beancount 文件中出现 `√` 后缀；再导入后状态为 verified。

## Open Questions

- `tx reconcile` 是否保留 toggle 行为（verified ↔ default），还是仅设为 verified？
- beancount 导出时 commodity 的 `created_at` 若缺失，是回退 1970 还是使用交易最早日期？
