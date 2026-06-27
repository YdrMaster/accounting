## Context

本系统是一个基于 Rust + SQLite 的复式记账应用，数据存储在二进制 SQLite 文件中，缺乏可读的备份和迁移手段。系统已有 `config export/import` 命令处理配置数据（YAML 格式），但账目数据（交易、账户、分录等）尚无导出能力。

Beancount 是成熟的纯文本复式记账格式，其账户路径（`:` 分隔）与本系统天然兼容。通过 beancount 格式实现账目导入导出，可以获得可读、可 diff、可版本管理的数据备份能力。

现有代码结构：
- `accounting` — 核心领域模型（Transaction, Posting, Account, Commodity, Channel, ChannelPath, Tag, Attachment, Member 等）
- `accounting-sql` — SQLite 持久化层（SqliteDatabase 提供所有 CRUD 接口）
- `accounting-service` — 业务逻辑层（含 config 导入导出、账单导入等）
- `accounting-cli` — CLI 层（clap 子命令结构，已有 `config export/import` 模式可参考）
- `accounting-api` — HTTP API 层

## Goals / Non-Goals

**Goals:**
- 实现 `accounting-cli beancount export` — 将数据库全量账目导出为 beancount 文本文件 + 附件目录
- 实现 `accounting-cli beancount import` — 从 beancount 文件导入账目到数据库，通过 `internal_id` metadata 重建内部引用关系
- 导出覆盖：Commodity、Account（含类型/账单日/还款日/关闭状态）、Member、Channel、Transaction（含 Posting/冲减关系/渠道链路/成员/标签/附件）
- 移除 `Posting.description` 字段（误加）
- 支持 round-trip：导出后导入应能无损恢复所有账目数据

**Non-Goals:**
- 不导出配置数据（AccountMapping、Budget）— 已有独立 YAML 机制
- 不做 HTTP API 端点
- 不支持 beancount 全部特性（balance assertion、pad、price directive 等）
- 不做增量导入（每次导入为全量覆盖或追加，不做 diff 合并）

## Decisions

### 决策 1: 新增 `accounting-beancount` crate 作为核心解析/序列化层

**选择**: 新建 `accounting-beancount` crate，负责 beancount 文本的解析和生成。

**理由**: 将 beancount 格式处理逻辑与 CLI 解耦，便于测试和复用。`accounting-cli` 只负责参数解析和调用 service，`accounting-beancount` 负责格式转换。

**替代方案**: 直接在 `accounting-cli` 中实现 — 但会导致 CLI 层过于臃肿，且无法被其他 crate（如未来的 API 层）复用。

### 决策 2: Beancount 文本生成采用手写模板，不依赖外部 beancount 库

**选择**: 手写 beancount 文本生成器（格式化输出），手写简易解析器（导入时）。

**理由**: Rust 生态中没有成熟的 beancount 解析库。beancount 文本格式相对简单（日期 + 指令 + metadata），手写生成器可控性更强。解析器只需处理本系统导出的格式，不需要支持 beancount 全部语法。

**替代方案**: 引入外部 beancount 解析库 — 生态不成熟，且本系统只需处理自己导出的格式。

### 决策 3: 导出格式 — metadata 承载本系统独有概念

**选择**: 使用 beancount metadata（`key: value` 缩进语法）承载本系统独有数据。

具体映射规则：

| 本系统概念 | Beancount 承载方式 |
|---|---|
| Transaction.description | narration（payee 留空 `""`） |
| Transaction.kind | txn metadata: `kind: "normal"` / `"refund"` / `"reimbursement"` |
| Transaction.member_id | txn metadata: `member: "成员名"` |
| ChannelPath[] | txn metadata: `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'` (JSON) |
| Posting.is_reimbursable | posting metadata: `reimbursable: FALSE` / `TRUE` |
| Posting.linked_posting_id | txn metadata: `reversal_of: '{"posting_id": N, "target_posting_id": M}'` (JSON) |
| Account.billing_day | account open metadata: `billing_day: 15` |
| Account.repayment_day | account open metadata: `repayment_day: 5` |
| AccountType | account open metadata: `account_type: "Import"` |
| 所有实体 internal_id | metadata: `internal_id: N` |
| Attachment | `document` 指令 + 外部文件（`attachments/` 子目录） |
| Tag | `#hashtag`（直接映射） |

**理由**: metadata 是 beancount 的标准扩展机制，fava 等工具会忽略不认识的 metadata key，不会报错。JSON 值用于承载结构化数据（如 channel_path 数组）。

### 决策 4: AccountType::Import 的根账户映射

**选择**: 导出时将 `Import:xxx` 映射为 `Equity:Import:xxx`，通过 `account_type: "Import"` metadata 保留原始类型。导入时根据 metadata 还原。

**理由**: Beancount 只认 5 种标准根账户（Assets/Liabilities/Equity/Income/Expenses）。Import 类型在语义上最接近 Equity（导入类账户用于平衡账目）。映射到 Equity 下可保持 beancount 工具链兼容性。

### 决策 5: ID 重建策略

**选择**: 导出时在每个实体的 metadata 中记录 `internal_id`。导入时建立 `old_id → new_id` 映射表，用于重连所有外键引用（account_id, commodity_id, member_id, channel_id, posting_id 等）。

**理由**: 这是 beancount 生态做 round-trip 转换的标准做法。数据库自增 ID 在导入目标库中不可预测，必须通过映射表重连。

### 决策 6: 附件处理

**选择**: 导出时将附件二进制数据写入 `<导出目录>/attachments/` 子目录，文件名格式为 `<attachment_id>_<原始文件名>`。使用 beancount `document` 指令引用。导入时反向读取文件并存入数据库。

**理由**: Beancount 的 `document` 指令是处理附件的标准方式。外部文件存储避免 beancount 文本膨胀，且便于人工管理。

### 决策 7: Posting.description 移除

**选择**: 从 `accounting::posting::Posting` 结构体中移除 `description` 字段，同步修改数据库 schema、repo 层、API 层、CLI 层。

**理由**: 用户确认该字段为误加。分录不需要独立的描述字段，交易级别的 description 已足够。

### 决策 8: CLI 命令结构

**选择**: 新增 `beancount` 子命令组，包含 `export` 和 `import` 子命令。

```
accounting-cli <db> beancount export <output-dir>
accounting-cli <db> beancount import <input-file>
```

**理由**: 与现有 `config export/import` 模式一致。`export` 接收目录路径（因为需要写入附件子目录），`import` 接收文件路径。

### 决策 9: 导入时的账户处理

**选择**: 导入时如果账户不存在则自动创建（使用 `account_get_or_create_by_path`）。Commodity 不存在时自动创建（使用 `commodity_upsert_by_symbol`）。Member 不存在时自动创建（使用 `member_get_or_create_by_name`）。Channel 不存在时自动创建（使用 `channel_upsert_by_name`）。

**理由**: 全量导入场景下，目标库可能是空库或已有部分数据。upsert 策略避免手动预创建所有基础数据。

### 决策 10: 导入时的交易去重

**选择**: 不做去重。每次导入的交易都作为新交易插入。如果 `internal_id` metadata 存在且匹配到已有交易，则跳过该交易（视为重复导入）。

**理由**: 简单的去重策略足以应对重复导入场景。基于 `internal_id` 的去重比基于内容 hash 的去重更可靠。

## Risks / Trade-offs

- **[手写解析器的健壮性]** → 本系统只需解析自己导出的格式，不需要通用 beancount 解析器。导出时严格控制格式，导入时按固定格式解析。如果用户手动编辑了 beancount 文件导致格式不符，解析器报错退出即可。

- **[metadata JSON 值的可读性]** → channel_path 和 reversal_of 使用 JSON 编码，在 beancount 文件中不如原生语法直观。但这是承载结构化数据的必要妥协，且不影响 beancount 工具链处理。

- **[Posting.description 移除的破坏性]** → 这是 BREAKING 变更，会影响现有数据库 schema 和所有引用该字段的代码。需要通过 schema 迁移处理。由于是内部项目且字段确认误加，风险可控。

- **[导入覆盖策略]** → 当前设计为追加模式（不覆盖已有数据）。如果用户需要完全替换，需要先手动清空数据库再导入。未来可考虑增加 `--replace` 选项。

- **[beancount 文件大小]** → 大量附件的 document 引用会使文件变大，但文本本身仍可控。二进制数据存储在外部文件中，不影响 beancount 文本的可读性。
