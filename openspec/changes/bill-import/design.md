## Context

当前系统只能通过 CLI/API 手动逐笔创建交易。真实使用场景中，用户需要从支付宝、微信、银行等渠道 App 导出的账单文件批量导入月度账单，单月动辄数百行。手动录入不可行。

项目技术栈：Rust (Edition 2024)，SQLite (sqlx)，axum Web 框架，clap CLI。已有 `Import` 根账户和 `待处理` 系统 Tag 的需求。channel_paths 机制已实现，支持交易链路。

现有相关文件：
- 核心模型：`accounting/src/channel.rs`、`accounting/src/transaction.rs`、`accounting/src/posting.rs`
- 交易服务：`accounting-service/src/transaction_service.rs`
- 账户服务：`accounting-service/src/account_service.rs`
- Tag 服务：`accounting-service/src/tag_service.rs`
- Schema seed data：`accounting-sql/src/schema.rs`
- CLI 注册：`accounting-cli/src/cmd/mod.rs`

## Goals / Non-Goals

**Goals:**
- 新增 `accounting-import` crate，实现适配器 trait 和具体渠道适配器（首批支付宝）
- 支持从渠道 App 导出的账单文件批量导入交易（首批支付宝 CSV）
- 导入的交易 Posting 全部挂在 `Import:<来源>:<分类>` 根账户下，不污染用户定义的账户体系
- 使用 `待处理` 系统 Tag 标记导入的未确认交易
- 适配器 trait 输入 `&[u8]` + `ImportContext`，输出 `Iterator<Item = BillEntry>`
- 适配器内部自行决定文件格式解析方式，trait 不耦合具体解析库
- 导入批次返回 `Vec<TransactionId>`，仅在内存中不落盘
- skip-on-error 策略：单行解析错误不中断整批导入

**Non-Goals:**
- 不做自定义映射表（"支付宝:餐饮美食 → Expenses:Food"），留待未来扩展
- 不做交易去重（同一笔交易出现在多个渠道账单中的情况），留待未来扩展
- 不做退款行自动关联原始消费的 `linked_posting_id`，由用户自行建立关联
- 不做导入进度条或回调（CLI 批量导入完成后汇总输出即可）
- 不做 Web UI 的导入功能（先做 CLI，未来可复用 service 层）
- 首批只实现支付宝适配器，微信/银行等留待后续迭代
- 不修改现有 `Transaction` 结构体（用 Tag 替代 pending bool 字段）

## Decisions

### D1: 新增独立 crate `accounting-import`，而非放在现有 crate 中

**选择**：新增 `accounting-import` crate，仅依赖 `accounting`（核心模型），不依赖 `accounting-sql` 或 `accounting-service`。

**替代方案**：
- 在 `accounting-service` 中实现所有导入逻辑：耦合度高，文件解析依赖会传染到 service crate，且适配器逻辑与业务逻辑混杂。
- 在 `accounting` 核心模型 crate 中定义适配器 trait：核心 crate 应保持纯数据模型，不应引入 I/O 依赖。

**理由**：独立 crate 允许适配器层独立测试（输入字节 + context → 输出 BillEntry），不依赖数据库。依赖方向清晰：`accounting-cli` → `accounting-service` → `accounting-import` → `accounting`。文件格式解析依赖被隔离在 import crate 内部，各适配器按需引入自己的解析库。

### D2: 适配器 trait 输入 `&[u8]`，不耦合具体文件格式

**选择**：`BillAdapter::parse(&self, data: &[u8], ctx: &ImportContext)` —— 适配器内部自行决定如何解析输入字节，trait 签名中不出现任何具体文件格式解析库的类型。

**替代方案**：
- 输入具体解析库的类型：直接耦合该库，其他格式适配器无法复用。
- 输入 `Box<dyn Read + Seek>`：对 CSV/纯文本解析不友好（不需要 Seek），且生命周期复杂。

**理由**：`&[u8]` 是最通用的输入形式——CSV 适配器用 `std::str::from_utf8` 解析，未来其他格式适配器可根据需要选择解析方式。trait 保持简洁，不引入泛型或生命周期参数。各适配器对文件格式的具体处理方式是实现细节，不在设计中限定。

### D3: Import 根账户 + 系统 Tag，而非 Transaction.pending bool 字段

**选择**：新增 `Import` 系统根账户（`is_system=true`）和 `待处理` 系统 Tag（`is_system=true`），导入交易的 Posting 全挂在 `Import:<来源>:<分类>` 下，并自动添加 `待处理` Tag。

**替代方案**：
- `Transaction.pending: bool` 字段：需要修改 schema、模型、API、CLI，且语义单一，不如 Tag 灵活。
- 不使用 Import 根账户，直接放在用户真实账户下：会污染用户账户体系，大量自动创建的子账户难以管理。

**理由**：Tag 方案复用现有基础设施（`transaction_tags` 表、`TransactionFilter.tag_ids` 筛选），不修改 schema。用户修正后只需去除 Tag 即可，无需专门的账户移动命令。Import 根账户完全隔离导入数据，用户可以逐步审查后手动重新分类（未来可通过映射表自动化）。`pending` 语义不仅限于导入——用户也可手动标记未确认交易。

### D4: 全挂 Import 下（两侧 Posting 都在 Import 根账户下）

**选择**：导入交易的所有 Posting（支出侧和资产侧）都挂在 `Import:<来源>:...` 下。

**替代方案**：
- 资产侧用真实账户（`Assets:支付宝余额`），支出侧挂 Import：资产侧余额更准确，但需要预先知道或自动创建真实资产账户，且 `Channel.account_id` 关联的账户可能不存在。

**理由**：全挂 Import 下实现最简单——适配器只需关心来源和分类，不需要了解用户的真实账户体系。完全隔离意味着导入不会影响任何真实账户余额，用户审查后手动重新分类。未来可通过自定义映射表优化，让资产侧直接写入真实账户。

### D5: skip-on-error 策略

**选择**：单行解析错误不中断整批导入，记录错误行号和原因，最终汇总输出。

**替代方案**：
- atomic（一行错全部回滚）：用户被迫修复每一个小问题后重新导入，体验差。
- stop-on-first-error：与 atomic 类似，但至少知道第一个错误在哪。

**理由**：账单文件中常有空行、格式异常行、不相关的表头行等，这些不应阻断整批导入。skip-on-error 策略让用户一次导入就能看到整体情况，再针对性处理少量问题行。

### D6: 批次 ID 列表仅在内存中，不落盘

**选择**：`ImportService::import()` 返回 `Vec<TransactionId>`，不写入数据库。CLI 输出后即丢弃。

**替代方案**：
- 新增 `import_batches` 表记录每次导入：增加 schema 复杂度，且当前无明确需求回溯历史导入批次。
- 将交易 ID 列表写入文件：比落盘表轻量，但当前 CLI 输出已满足需求。

**理由**：导入后用户通过 `tx list --tag 待处理` 即可定位所有导入的未确认交易，批次 ID 列表仅为方便即时查看。无需持久化。未来如需批次管理，可在那时增加。

### D7: `BillEntry` 使用字符串账户路径，不使用 `AccountId`

**选择**：`BillPosting.account_path: String`（如 `"Import:支付宝:餐饮美食"`）和 `BillPosting.commodity_symbol: String`，在 service 层解析为真实 ID。

**理由**：`accounting-import` crate 不依赖 `accounting-sql`，无法查询数据库获取 `AccountId`。字符串路径是最自然的表达方式，service 层可通过 `ensure_cascading` 查找或创建对应账户。

### D8: 适配器实现聚焦支付宝 CSV，其他格式留待扩展

**选择**：首批只实现支付宝适配器，使用 CSV 格式。其他渠道（微信、银行等）及其他文件格式留待后续迭代。

**理由**：支付宝 CSV 是最典型、最容易获取的账单来源，先集中精力把 adapter trait + ImportService + CLI 的完整链路跑通。文件格式的具体解析细节是适配器内部实现层面的事，不需要在设计中硬性约定。各适配器按需引入自己的解析依赖。

## Risks / Trade-offs

- **[Import 根账户可能积累大量子账户]** → 每个 `来源:分类` 组合都会创建一个子账户，长期积累后 Import 下可能有上百个子账户。但这是隔离设计的必要代价，未来可通过映射表减少手动操作。
- **[适配器对文件格式敏感]** → 各渠道 App 的导出格式可能随版本变化，适配器需要持续维护。设计上通过独立 crate 隔离变更影响，各适配器内部可做格式版本检测。
- **[退款关联未自动处理]** → 退款行导入为独立交易，不自动关联原始消费。用户需手动在 UI/CLI 中建立 `linked_posting_id` 关系。这是已知的非目标。
- **[交易去重未处理]** → 同一笔交易可能出现在多个渠道账单中（淘宝→支付宝→银行卡），当前不检测去重。这是已知的非目标，留待未来通过自动化判断处理。

## Open Questions

- 支付宝/微信等具体适配器需要解析哪些列？需要在实现阶段拿到真实的导出文件样本后确定。首批聚焦支付宝 CSV，具体列映射在实现时决定。