# Delta: bill-import

## ADDED Requirements

### Requirement: 支付宝适配器按交易状态判定退款
支付宝适配器 SHALL 以 `交易状态` 列精确匹配 `"退款成功"` 判定退款行，不再使用 `交易分类 == "退款"` 的分类名嗅探。判定为退款的行，其 `role=IncomeExpense` 的 BillPosting `is_refund` SHALL 为 `true`，金额为负（负支出），`role=Asset` 的 BillPosting 金额为正；`is_refund` 与分类名无关。

#### Scenario: 分类为"退款"的退款行
- **WHEN** 适配器解析一行交易状态为 `"退款成功"`、交易分类为 `"退款"`、收/支为 `"不计收支"` 的账单
- **THEN** 收支侧 BillPosting 的 `is_refund=true`，金额为负

#### Scenario: 分类为真实消费分类的退款行
- **WHEN** 适配器解析一行交易状态为 `"退款成功"`、交易分类为 `"餐饮美食"`、收/支为 `"不计收支"` 的账单
- **THEN** 收支侧 BillPosting 的 `is_refund=true`，金额为负，category 仍为 `"餐饮美食"`

#### Scenario: 交易成功行不判定为退款
- **WHEN** 适配器解析一行交易状态为 `"交易成功"` 的账单
- **THEN** 收支侧 BillPosting 的 `is_refund=false`，金额方向按收/支列规则计算

### Requirement: 支付宝适配器兼容多种日期格式
支付宝适配器 SHALL 兼容以下交易时间格式：短横线带秒（`2026-06-30 19:29:03`）、短横线无秒、斜杠带秒、斜杠无秒且月日不补零（`2026/7/25 11:43`，新版导出格式）。

#### Scenario: 新版斜杠无秒日期
- **WHEN** 适配器解析一行交易时间为 `"2026/7/21 19:00"` 的账单
- **THEN** 该行正常解析，date_time 为 2026-07-21 19:00:00

#### Scenario: 旧版短横线带秒日期
- **WHEN** 适配器解析一行交易时间为 `"2026-06-30 19:29:03"` 的账单
- **THEN** 该行正常解析，行为与改动前一致

## MODIFIED Requirements

### Requirement: BillEntry 数据结构
系统 SHALL 定义 `BillEntry` 结构体作为适配器输出的标准格式，包含 `date_time`、`description`、`kind`、`postings: Vec<BillPosting>`、`tags: Vec<String>` 字段。`BillPosting` SHALL 使用 `role: PostingRole` 和 `category: String` 替代原有的 `account_path: String`，并保留 `commodity_symbol: String`、`amount: Decimal`、`is_reimbursable: bool` 字段。`BillPosting` SHALL 包含 `is_refund: bool` 字段，由适配器显式标记该分录是否为退款事件（见「支付宝适配器按交易状态判定退款」）。`PostingRole` 为枚举类型，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。

#### Scenario: BillPosting 使用 role + category
- **WHEN** 适配器解析一行支付宝账单 "美团外卖 -35.00 餐饮美食 蚂蚁宝藏信用卡"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="餐饮美食", amount=+35, is_refund=false}` 和 `{role=Asset, category="蚂蚁宝藏信用卡", amount=-35}`

#### Scenario: 收入方向的 BillPosting
- **WHEN** 适配器解析一行收入账单 "工资 +100.00 余额宝"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="工资", amount=-100, is_refund=false}` 和 `{role=Asset, category="余额宝", amount=+100}`

### Requirement: 标准根账户下的 Import fallback
系统不再提供 `Import` 系统根账户。无映射时，`Asset` 角色分录 SHALL 落到 `Assets:Import:<来源>:<分类>`，`IncomeExpense` 角色金额为正（支出）或 `is_refund=true`（退款，负支出）时 SHALL 落到 `Expenses:Import:<来源>:<分类>`，`IncomeExpense` 角色金额为负（收入）且非退款时 SHALL 落到 `Income:Import:<来源>:<分类>`，由 service 层通过 `ensure_cascading` 自动创建。

#### Scenario: 无映射时支出侧自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, amount>0, category="餐饮美食"` 且 (member_id, channel_id, "Expenses:餐饮美食") 无映射
- **THEN** service 层自动创建 Expenses → Import → 支付宝 → 餐饮美食 四级账户

#### Scenario: 无映射时收入侧自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, amount<0, is_refund=false, category="工资"` 且 (member_id, channel_id, "Income:工资") 无映射
- **THEN** service 层自动创建 Income → Import → 支付宝 → 工资 四级账户

#### Scenario: 无映射时资产侧自动创建子账户
- **WHEN** 适配器输出 `role=Asset, category="蚂蚁宝藏信用卡"` 且 (member_id, channel_id, "Assets:蚂蚁宝藏信用卡") 无映射
- **THEN** service 层自动创建 Asset → Import → 支付宝 → 蚂蚁宝藏信用卡 四级账户

#### Scenario: 无映射时退款自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, is_refund=true, category="退款"` 且 (member_id, channel_id, "Expenses:退款") 无映射
- **THEN** service 层自动创建 Expenses → Import → 支付宝 → 退款 四级账户

#### Scenario: 无映射时真实分类退款落到对应 Expenses 子账户
- **WHEN** 适配器输出 `role=IncomeExpense, is_refund=true, category="餐饮美食"` 且 (member_id, channel_id, "Expenses:餐饮美食") 无映射
- **THEN** service 层自动创建 Expenses → Import → 支付宝 → 餐饮美食 四级账户，分录金额为负

#### Scenario: 相同路径不重复创建
- **WHEN** 两条 BillEntry 都使用 `role=IncomeExpense, amount>0, category="餐饮美食"` 且均无映射
- **THEN** 系统只创建一次 `Expenses:Import:支付宝:餐饮美食` 账户，两条 Posting 指向同一个 AccountId

### Requirement: PostingRole 枚举
系统 SHALL 在 `accounting` crate 中定义 `PostingRole` 枚举，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。`Asset` 的映射 key 前缀为 `"Assets"`。`IncomeExpense` 角色按以下规则生成映射 key 和 fallback 路径（`is_refund` 来自 BillPosting 的显式标记，系统 SHALL NOT 按分类名嗅探退款）：
- `is_refund=true` 时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`，金额为负。
- `is_refund=false` 且 amount > 0 时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`。
- `is_refund=false` 且 amount < 0 时，映射 key 前缀为 `"Income"`，fallback 路径为 `Income:Import:<channel>:<category>`。

#### Scenario: 生成支出侧映射 key
- **WHEN** role = IncomeExpense, amount > 0, is_refund = false, category = "餐饮美食"
- **THEN** 映射 key 为 "Expenses:餐饮美食"

#### Scenario: 生成收入侧映射 key
- **WHEN** role = IncomeExpense, amount < 0, is_refund = false, category = "工资"
- **THEN** 映射 key 为 "Income:工资"

#### Scenario: 生成退款侧映射 key
- **WHEN** role = IncomeExpense, is_refund = true, category = "退款"
- **THEN** 映射 key 为 "Expenses:退款"

#### Scenario: 真实分类退款生成对应分类的 Expenses 映射 key
- **WHEN** role = IncomeExpense, is_refund = true, category = "餐饮美食"
- **THEN** 映射 key 为 "Expenses:餐饮美食"，与被误判为正支出时的 key 一致，既有映射直接生效

#### Scenario: 生成资产侧映射 key
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡"
- **THEN** 映射 key 为 "Assets:蚂蚁宝藏信用卡"

#### Scenario: 生成资产侧 Import fallback 路径
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Assets:Import:支付宝:蚂蚁宝藏信用卡"

#### Scenario: 生成支出侧 Import fallback 路径
- **WHEN** role = IncomeExpense, amount > 0, is_refund = false, category = "餐饮美食", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Expenses:Import:支付宝:餐饮美食"

#### Scenario: 生成退款侧 Import fallback 路径
- **WHEN** role = IncomeExpense, is_refund = true, category = "退款", 渠道名 = "支付宝"
- **THEN** fallback 路径为 "Expenses:Import:支付宝:退款"
