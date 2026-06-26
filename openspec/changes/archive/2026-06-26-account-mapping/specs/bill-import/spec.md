## MODIFIED Requirements

### Requirement: BillEntry 数据结构
系统 SHALL 定义 `BillEntry` 结构体作为适配器输出的标准格式，包含 `date_time`、`description`、`kind`、`postings: Vec<BillPosting>`、`tags: Vec<String>` 字段。`BillPosting` SHALL 使用 `role: PostingRole` 和 `category: String` 替代原有的 `account_path: String`，并保留 `commodity_symbol: String`、`amount: Decimal`、`is_reimbursable: bool` 字段。`PostingRole` 为枚举类型，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。

#### Scenario: BillPosting 使用 role + category
- **WHEN** 适配器解析一行支付宝账单 "美团外卖 -35.00 餐饮美食 蚂蚁宝藏信用卡"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="餐饮美食", amount=+35}` 和 `{role=Asset, category="蚂蚁宝藏信用卡", amount=-35}`

#### Scenario: 收入方向的 BillPosting
- **WHEN** 适配器解析一行收入账单 "工资 +100.00 余额宝"
- **THEN** 产出的 BillEntry 包含两个 BillPosting：`{role=IncomeExpense, category="工资", amount=-100}` 和 `{role=Asset, category="余额宝", amount=+100}`

### Requirement: Import 系统根账户
系统 SHALL 新增 `Import` 系统根账户（`is_system=true`），作为所有导入交易的 Posting 容器。适配器 SHALL 按 `Import:<来源>:收支:<分类>` 和 `Import:<来源>:资产:<付款方式>` 格式生成 fallback 账户路径，由 service 层通过 `ensure_cascading` 自动创建。

#### Scenario: Import 根账户在初始化时创建
- **WHEN** 数据库初始化（seed data）
- **THEN** `Import` 根账户存在，`is_system=true`，`parent_id=NULL`

#### Scenario: 无映射时收支侧自动创建子账户
- **WHEN** 适配器输出 `role=IncomeExpense, category="餐饮美食"` 且 (member_id, channel_id, "收支:餐饮美食") 无映射
- **THEN** service 层自动创建 Import → 支付宝 → 收支 → 餐饮美食 四级账户

#### Scenario: 无映射时资产侧自动创建子账户
- **WHEN** 适配器输出 `role=Asset, category="蚂蚁宝藏信用卡"` 且 (member_id, channel_id, "资产:蚂蚁宝藏信用卡") 无映射
- **THEN** service 层自动创建 Import → 支付宝 → 资产 → 蚂蚁宝藏信用卡 四级账户

#### Scenario: 相同路径不重复创建
- **WHEN** 两条 BillEntry 都使用 `role=IncomeExpense, category="餐饮美食"` 且均无映射
- **THEN** 系统只创建一次 `Import:支付宝:收支:餐饮美食` 账户，两条 Posting 指向同一个 AccountId

### Requirement: ImportService 编排导入流程
系统 SHALL 在 `accounting-service` crate 中由 `ImportService` 编排完整导入流程：确保 Import 根账户和待处理 Tag 存在 → 选择适配器 → 调用适配器解析 → 迭代 BillEntry → 对每个 BillPosting 查询账户映射表决定目标账户（有映射用映射账户，无映射走 Import fallback）→ 调用 `TransactionService::submit` → 收集 TransactionId → 返回批次结果。

#### Scenario: 有映射时使用映射账户
- **WHEN** (member_id=1, channel_id=1, "收支:餐饮美食") 存在映射 → AccountId(42)
- **THEN** 该 BillPosting 的 account_id 为 42，不创建 Import 子账户

#### Scenario: 无映射时使用 Import fallback 账户
- **WHEN** (member_id=1, channel_id=1, "收支:餐饮美食") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Import:支付宝:收支:餐饮美食` 对应的 AccountId

#### Scenario: 部分映射部分 fallback
- **WHEN** 收支侧有映射但资产侧无映射
- **THEN** 收支侧 Posting 使用映射账户，资产侧 Posting 使用 Import fallback 账户

#### Scenario: 无论映射是否完全，交易仍挂待处理 Tag
- **WHEN** 一条 BillEntry 的所有 Posting 都有映射
- **THEN** 该交易仍自动添加 "待处理" Tag，与无映射时行为一致

#### Scenario: 返回批次交易 ID 列表
- **WHEN** 导入完成
- **THEN** 返回 `Vec<TransactionId>`，包含本次导入创建的所有交易 ID

#### Scenario: skip-on-error 策略
- **WHEN** 导入过程中单行解析错误
- **THEN** 跳过该行继续处理，最终汇总输出成功数和错误详情

## ADDED Requirements

### Requirement: PostingRole 枚举
系统 SHALL 在 `accounting` crate 中定义 `PostingRole` 枚举，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。`IncomeExpense` 的映射 key 前缀为 `"收支"`，Import fallback 路径段为 `"收支"`；`Asset` 的映射 key 前缀为 `"资产"`，Import fallback 路径段为 `"资产"`。

#### Scenario: 生成映射 key
- **WHEN** role = IncomeExpense, category = "餐饮美食"
- **THEN** 映射 key 为 "收支:餐饮美食"

#### Scenario: 生成 Import fallback 路径
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡", 渠道名 = "支付宝", import_root = "Import"
- **THEN** fallback 路径为 "Import:支付宝:资产:蚂蚁宝藏信用卡"