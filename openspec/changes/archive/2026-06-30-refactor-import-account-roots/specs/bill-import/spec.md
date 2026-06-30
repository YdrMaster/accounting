# bill-import

## MODIFIED Requirements

### Requirement: ImportService 编排导入流程
系统 SHALL 在 `accounting-service` crate 中由 `ImportService` 编排完整导入流程：选择适配器 → 调用适配器解析 → 迭代 BillEntry → 对每个 BillPosting 查询账户映射表决定目标账户（有映射用映射账户，无映射走标准根账户下的 Import fallback）→ 调用 `TransactionService::submit` → 收集 TransactionId → 返回批次结果。

#### Scenario: 有映射时使用映射账户
- **WHEN** (member_id=1, channel_id=1, "Expenses:餐饮美食") 存在映射 → AccountId(42)
- **THEN** 该 BillPosting 的 account_id 为 42，不创建 Import 子账户

#### Scenario: 无映射时使用 Asset fallback 账户
- **WHEN** (member_id=1, channel_id=1, "Assets:蚂蚁宝藏信用卡") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Assets:Import:Alipay:蚂蚁宝藏信用卡` 对应的 AccountId

#### Scenario: 无映射时使用 Expenses fallback 账户
- **WHEN** (member_id=1, channel_id=1, "Expenses:餐饮美食") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Expenses:Import:Alipay:餐饮美食` 对应的 AccountId

#### Scenario: 无映射时使用 Income fallback 账户
- **WHEN** (member_id=1, channel_id=1, "Income:工资") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Income:Import:Alipay:工资` 对应的 AccountId

#### Scenario: 无映射时使用 Refund fallback 账户
- **WHEN** (member_id=1, channel_id=1, "Expenses:退款") 不存在映射
- **THEN** 该 BillPosting 的 account_id 为 `Expenses:Import:Alipay:退款` 对应的 AccountId

#### Scenario: 部分映射部分 fallback
- **WHEN** 支出侧有映射但资产侧无映射
- **THEN** 支出侧 Posting 使用映射账户，资产侧 Posting 使用 `Assets:Import:Alipay:<付款方式>` fallback 账户

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

### Requirement: PostingRole 枚举
系统 SHALL 在 `accounting` crate 中定义 `PostingRole` 枚举，包含 `IncomeExpense`（收支侧）和 `Asset`（资产侧）两个变体。`Asset` 的映射 key 前缀为 `"Asset"`。`IncomeExpense` 角色按以下规则生成映射 key 和 fallback 路径：
- category 为退款（"退款" 或 "Refund"）时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`，金额为负。
- category 非退款且 amount > 0 时，映射 key 前缀为 `"Expenses"`，fallback 路径为 `Expenses:Import:<channel>:<category>`。
- category 非退款且 amount < 0 时，映射 key 前缀为 `"Income"`，fallback 路径为 `Income:Import:<channel>:<category>`。

#### Scenario: 生成资产侧映射 key
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡"
- **THEN** 映射 key 为 "Assets:蚂蚁宝藏信用卡"

#### Scenario: 生成支出侧映射 key
- **WHEN** role = IncomeExpense, amount > 0, category = "餐饮美食"
- **THEN** 映射 key 为 "Expenses:餐饮美食"

#### Scenario: 生成收入侧映射 key
- **WHEN** role = IncomeExpense, amount < 0, category = "工资"
- **THEN** 映射 key 为 "Income:工资"

#### Scenario: 生成退款侧映射 key
- **WHEN** role = IncomeExpense, category = "退款"
- **THEN** 映射 key 为 "Expenses:退款"

#### Scenario: 生成资产侧 Import fallback 路径
- **WHEN** role = Asset, category = "蚂蚁宝藏信用卡", 渠道名 = "Alipay"
- **THEN** fallback 路径为 "Assets:Import:Alipay:蚂蚁宝藏信用卡"

#### Scenario: 生成支出侧 Import fallback 路径
- **WHEN** role = IncomeExpense, amount > 0, category = "餐饮美食", 渠道名 = "Alipay"
- **THEN** fallback 路径为 "Expenses:Import:Alipay:餐饮美食"

#### Scenario: 生成收入侧 Import fallback 路径
- **WHEN** role = IncomeExpense, amount < 0, category = "工资", 渠道名 = "Alipay"
- **THEN** fallback 路径为 "Income:Import:Alipay:工资"

#### Scenario: 生成退款侧 Import fallback 路径
- **WHEN** role = IncomeExpense, category = "退款", 渠道名 = "Alipay"
- **THEN** fallback 路径为 "Expenses:Import:Alipay:退款"

## REMOVED Requirements

### Requirement: Import 系统根账户
系统 SHALL 新增 `Import` 系统根账户（`is_system=true`），作为所有导入交易的 Posting 容器。适配器 SHALL 按 `Import:<来源>:收支:<分类>` 和 `Import:<来源>:资产:<付款方式>` 格式生成 fallback 账户路径，由 service 层通过 `ensure_cascading` 自动创建。

#### Scenario: Import 根账户在初始化时创建
- **WHEN** 数据库初始化（seed data）
- **THEN** 不再创建 `Import` 根账户

**Reason**: `Import` 类型不是 beancount 标准账户类型，导出时需折中映射为 `Equity:Import:`，兼容性差。

**Migration**: 新数据库使用 `Assets:Import:<channel>`、`Income:Import:<channel>`、`Expenses:Import:<channel>` 作为导入容器；旧 `导入:` 数据不迁移。
