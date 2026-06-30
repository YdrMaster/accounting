# bill-import

## Purpose

账单导入功能——从外部渠道 App 导出的账单文件批量导入交易，通过适配器模式支持多渠道，无映射交易使用 `Assets:Import:<channel>` / `Income:Import:<channel>` / `Expenses:Import:<channel>` fallback 账户并标记待处理 Tag。

## Requirements

### Requirement: 账单适配器 trait
系统 SHALL 提供 `BillAdapter` trait，定义统一的账单解析接口。trait 方法接受 `&[u8]`（原始文件字节）和 `ImportContext`（补充上下文），返回 `Iterator<Item = Result<BillEntry, AdaptError>>`。trait 中 SHALL 不出现任何具体文件格式解析库的类型。适配器内部自行决定如何解析输入字节。

#### Scenario: 支付宝适配器解析账单
- **WHEN** 调用 AlipayAdapter 的 `parse` 方法，传入支付宝导出的账单文件字节和 ImportContext
- **THEN** 返回一个迭代器，每次 `next()` 产出一个 `BillEntry`，包含日期、描述、金额、分类等信息

#### Scenario: 适配器名称
- **WHEN** 调用任意适配器的 `name()` 方法
- **THEN** 返回该适配器的标识字符串（如 `"alipay"`）

### Requirement: 支付宝适配器按 `&` 拆分收/付款方式

支付宝适配器 SHALL 在解析 `收/付款方式` 字段时，按 `&` 字符拆分为多个部分；每个非空部分各自生成一个 `role=Asset` 的 `BillPosting`。第一个部分承担该交易全部资产侧金额，其余部分金额为 `Decimal::ZERO`。空字段或拆分后全部为空时，仍回退为使用渠道名作为单一 Asset Posting 的 `category`。

#### Scenario: 收/付款方式含两个部分

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡(江苏银行)&超划算"`，交易金额为 `4.80`，方向为支出
- **THEN** 产出的 `BillEntry` 包含三个 `BillPosting`：
  - `{role=IncomeExpense, category="餐饮美食", amount=+4.80}`
  - `{role=Asset, category="蚂蚁宝藏信用卡(江苏银行)", amount=-4.80}`
  - `{role=Asset, category="超划算", amount=0.00}`

#### Scenario: 收/付款方式含三个部分

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡(江苏银行)&茶咖自由卡&超划算"`
- **THEN** 第一个部分 `"蚂蚁宝藏信用卡(江苏银行)"` 承担全部资产侧金额，其余两个部分生成金额为 `0.00` 的 Asset Posting

#### Scenario: 退款交易中收/付款方式含 `&`

- **WHEN** 适配器解析一笔退款账单，其 `收/付款方式` 为 `"招商银行信用卡分期&招商银行满减"`，退款金额为 `3158.00`
- **THEN** 第一个部分 `"招商银行信用卡分期"` 的 Asset Posting 金额为 `+3158.00`，其余部分金额为 `0.00`

#### Scenario: 收/付款方式为空

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为空字符串
- **THEN** 生成一个 `role=Asset` 的 `BillPosting`，其 `category` 为 `ImportContext` 中的渠道名（如 `"支付宝"`），金额按原有规则计算

#### Scenario: 收/付款方式无 `&`

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡"` 且不含 `&`
- **THEN** 行为与改动前一致，仅生成一个 `role=Asset` 的 `BillPosting`

### Requirement: BillEntry 数据结构
系统 SHALL 定义 `BillEntry` 结构体作为适配器输出的标准格式，包含 `date_time`、`description`、`kind`、`postings: Vec<BillPosting>`、`tags: Vec<String>` 字段。`BillPosting` SHALL 使用 `role: PostingRole` 和 `category: String` 替代原有的 `account_path: String`，并保留 `commodity_symbol: String`、`amount: Decimal`、`is_reimbursable: bool` 字段。`PostingRole` 为枚举类型，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。

#### Scenario: BillPosting 使用 role + category
- **WHEN** 适配器解析一行支付宝账单 "美团外卖 -35.00 餐饮美食 蚂蚁宝藏信用卡"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="餐饮美食", amount=+35}` 和 `{role=Asset, category="蚂蚁宝藏信用卡", amount=-35}`

#### Scenario: 收入方向的 BillPosting
- **WHEN** 适配器解析一行收入账单 "工资 +100.00 余额宝"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="工资", amount=-100}` 和 `{role=Asset, category="余额宝", amount=+100}`

### Requirement: ImportContext 补充上下文
系统 SHALL 定义 `ImportContext` 结构体，包含 `member_id`、`channel_id`、`commodity_id` 字段，在调用适配器时传入，为解析提供运行时补充信息。

#### Scenario: 传入 member_id 和 channel_id
- **WHEN** 用户指定 `--member 1 --source alipay` 进行导入
- **THEN** ImportContext 包含 member_id=MemberId(1)、channel_id 对应支付宝渠道

#### Scenario: 默认商品为 CNY
- **WHEN** 用户未指定 commodity
- **THEN** ImportContext 的 commodity_id 默认为 CNY 对应的 ID

### Requirement: 标准根账户下的 Import fallback
系统不再提供 `Import` 系统根账户。无映射时，`Asset` 角色分录 SHALL 落到 `Assets:Import:<来源>:<分类>`，`IncomeExpense` 角色金额为正（支出）或退款（category 为 "退款" / "Refund"）时 SHALL 落到 `Expenses:Import:<来源>:<分类>`，`IncomeExpense` 角色金额为负（收入）时 SHALL 落到 `Income:Import:<来源>:<分类>`，由 service 层通过 `ensure_cascading` 自动创建。

#### Scenario: 无映射时支出侧自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, amount>0, category="餐饮美食"` 且 (member_id, channel_id, "Expenses:餐饮美食") 无映射
- **THEN** service 层自动创建 Expenses → Import → 支付宝 → 餐饮美食 四级账户

#### Scenario: 无映射时收入侧自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, amount<0, category="工资"` 且 (member_id, channel_id, "Income:工资") 无映射
- **THEN** service 层自动创建 Income → Import → 支付宝 → 工资 四级账户

#### Scenario: 无映射时资产侧自动创建子账户
- **WHEN** 适配器输出 `role=Asset, category="蚂蚁宝藏信用卡"` 且 (member_id, channel_id, "Assets:蚂蚁宝藏信用卡") 无映射
- **THEN** service 层自动创建 Asset → Import → 支付宝 → 蚂蚁宝藏信用卡 四级账户

#### Scenario: 无映射时退款自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, category="退款"` 且 (member_id, channel_id, "Expenses:退款") 无映射
- **THEN** service 层自动创建 Expenses → Import → 支付宝 → 退款 四级账户

#### Scenario: 相同路径不重复创建
- **WHEN** 两条 BillEntry 都使用 `role=IncomeExpense, amount>0, category="餐饮美食"` 且均无映射
- **THEN** 系统只创建一次 `Expenses:Import:支付宝:餐饮美食` 账户，两条 Posting 指向同一个 AccountId

### Requirement: 待处理系统 Tag
系统 SHALL 新增 `待处理` 系统 Tag（`is_system=true`），所有通过导入创建的交易 SHALL 自动添加此 Tag。用户确认交易后 SHALL 手动移除此 Tag。

#### Scenario: 导入交易自动标记待处理
- **WHEN** 通过 `import` 命令导入一批交易
- **THEN** 每笔交易的 tag 列表中包含 "待处理" Tag

#### Scenario: 用户确认后移除待处理 Tag
- **WHEN** 用户审查并确认一笔导入的交易
- **THEN** 用户移除该交易的 "待处理" Tag，交易仍保留在 Import 账户下（账户移动是独立操作）

#### Scenario: 按待处理 Tag 筛选交易
- **WHEN** 用户执行 `tx list --tag 待处理`
- **THEN** 返回所有标记了 "待处理" Tag 的交易，包括导入的和用户手动标记的

### Requirement: ImportService 编排导入流程
系统 SHALL 在 `accounting-service` crate 中由 `ImportService` 编排完整导入流程：确保待处理 Tag 存在 → 选择适配器 → 调用适配器解析 → 迭代 BillEntry → 对每个 BillPosting 查询账户映射表决定目标账户（有映射用映射账户，无映射走 `Assets:Import:<channel>` / `Income:Import:<channel>` / `Expenses:Import:<channel>` fallback）→ 调用 `TransactionService::submit` → 收集 TransactionId → 返回批次结果。

#### Scenario: 有映射时使用映射账户
- **WHEN** (member_id=1, channel_id=1, "Expenses:餐饮美食") 存在映射 → AccountId(42)
- **THEN** 该 BillPosting 的 account_id 为 42，不创建 Import 子账户

#### Scenario: 无映射时使用 Import fallback 账户
- **WHEN** (member_id=1, channel_id=1, "Expenses:餐饮美食") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Expenses:Import:支付宝:餐饮美食` 对应的 AccountId

#### Scenario: 部分映射部分 fallback
- **WHEN** 收支侧有映射但资产侧无映射
- **THEN** 收支侧 Posting 使用映射账户，资产侧 Posting 使用 Import fallback 账户

#### Scenario: 无论映射是否完全，交易仍挂待处理 Tag
- **WHEN** 一条 BillEntry 的所有 Posting 都有映射
- **THEN** 该交易仍自动添加 "待处理" Tag，与无映射时行为一致

#### Scenario: 返回批次交易 ID 列表
- **WHEN** 导入完成
- **THEN** 返回 `Vec<TransactionId>`，包含本次导入创建的所有交易 ID

#### Scenario: 批次 ID 列表不落盘
- **WHEN** 导入完成并返回 TransactionId 列表
- **THEN** 该列表仅在内存中，不写入数据库任何表

#### Scenario: skip-on-error 策略
- **WHEN** 导入过程中单行解析错误
- **THEN** 跳过该行继续处理，最终汇总输出成功数和错误详情

### Requirement: skip-on-error 策略
导入过程中，单行解析错误 SHALL 不中断整批导入。系统 SHALL 以结构化方式记录错误（包含行号和错误类型/数据），继续处理后续行，最终由 CLI 根据当前语言环境汇总输出成功数和错误详情。

#### Scenario: 跳过格式错误的行
- **WHEN** 第 5 行金额字段为空、第 18 行日期格式无效
- **THEN** 系统跳过这两行并继续处理其他行，英文环境下 CLI 报告 "Imported 23 transactions, skipped 2 (row 5: amount parse failed; row 18: date parse failed)"

#### Scenario: 所有行均成功
- **WHEN** 所有行解析成功
- **THEN** 英文环境下 CLI 报告 "Imported N transactions, skipped 0"

### Requirement: CLI import 子命令
系统 SHALL 在 CLI 中新增 `import` 子命令，接受 `--file <路径>`、`--source <来源>`、`--member <NAME>` 等参数，执行导入并输出结果摘要和交易 ID 列表。所有输出文本 SHALL 通过 `accounting-cli` 的 locale 文件进行本地化。

#### Scenario: 基本导入命令
- **WHEN** 用户执行 `accounting import --source alipay --member Alice --file bill.csv`
- **THEN** 系统读取文件，选择支付宝适配器，执行导入，输出本地化的摘要信息和交易 ID 列表

#### Scenario: 成员名称不存在
- **WHEN** 用户执行 `accounting import --source alipay --member Bob --file bill.csv` 且成员 Bob 不存在
- **THEN** 系统返回本地化错误，英文环境下为 "Member not found: Bob"

#### Scenario: 不支持的来源
- **WHEN** 用户指定 `--source unknown_provider`
- **THEN** 系统返回本地化错误，英文环境下为 "Unsupported source: unknown_provider"

#### Scenario: 文件不存在
- **WHEN** 用户指定的文件路径不存在
- **THEN** 系统返回本地化的文件读取错误

### Requirement: 适配器与渠道自动关联
当用户指定 `--source` 选择适配器类型时，系统 SHALL 同时确定渠道（Channel）并设置到 ImportContext 中。适配器对应的渠道 SHALL 为系统内置渠道（`is_system=1`），在数据库初始化时自动创建。

#### Scenario: 选择支付宝适配器时自动设置渠道
- **WHEN** 用户指定 `--source alipay` 或 `--source 支付宝`
- **THEN** 系统查找名为 "支付宝"（中文环境）或 "Alipay"（英文环境）的内置 Channel，将其 channel_id 设入 ImportContext，导入交易的 channel_path 为 [支付宝]

#### Scenario: 渠道不存在时报错
- **WHEN** 指定的来源对应的内置渠道在系统中不存在（数据库未正确初始化）
- **THEN** 系统返回错误，提示"渠道不存在，请检查数据库初始化"

### Requirement: 适配器注册机制
系统 SHALL 提供适配器注册机制，使得 `ImportService` 可根据 `source` 名称查找对应的适配器实例。内置适配器通过 `builtin_adapters()` 函数提供。

#### Scenario: 查找内置适配器
- **WHEN** 调用 `find_adapter("alipay", &builtin_adapters())`
- **THEN** 返回 AlipayAdapter 实例的引用

#### Scenario: 查找不存在的适配器
- **WHEN** 调用 `find_adapter("unknown", &builtin_adapters())`
- **THEN** 返回 None

### Requirement: PostingRole 枚举
系统 SHALL 在 `accounting` crate 中定义 `PostingRole` 枚举，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。`Asset` 的映射 key 前缀为 `"Assets"`。`IncomeExpense` 角色按以下规则生成映射 key 和 fallback 路径：
- category 为退款（"退款" 或 "Refund"）时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`，金额为负。
- category 非退款且 amount > 0 时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`。
- category 非退款且 amount < 0 时，映射 key 前缀为 `"Income"`，fallback 路径为 `Income:Import:<channel>:<category>`。

#### Scenario: 生成支出侧映射 key
- **WHEN** role = IncomeExpense, amount > 0, category = "餐饮美食"
- **THEN** 映射 key 为 "Expenses:餐饮美食"

#### Scenario: 生成收入侧映射 key
- **WHEN** role = IncomeExpense, amount < 0, category = "工资"
- **THEN** 映射 key 为 "Income:工资"

#### Scenario: 生成退款侧映射 key
- **WHEN** role = IncomeExpense, category = "退款"
- **THEN** 映射 key 为 "Expenses:退款"

#### Scenario: 生成资产侧映射 key
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡"
- **THEN** 映射 key 为 "Assets:蚂蚁宝藏信用卡"

#### Scenario: 生成资产侧 Import fallback 路径
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Assets:Import:支付宝:蚂蚁宝藏信用卡"

#### Scenario: 生成支出侧 Import fallback 路径
- **WHEN** role = IncomeExpense, amount > 0, category = "餐饮美食", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Expenses:Import:支付宝:餐饮美食"

#### Scenario: 生成退款侧 Import fallback 路径
- **WHEN** role = IncomeExpense, category = "退款", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Expenses:Import:支付宝:退款"