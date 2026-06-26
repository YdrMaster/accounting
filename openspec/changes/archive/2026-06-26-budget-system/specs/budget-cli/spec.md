## ADDED Requirements

### Requirement: budget create 命令
CLI SHALL 提供 `budget create` 子命令，接受 `--name`（预算表名称）、`--period`（周期类型）、`--commodity`（币种 ID）和多个 `--limit`（账户路径:金额）参数。

#### Scenario: 创建月度预算
- **WHEN** 执行 `budget create --name "月度生活" --period monthly --commodity 1 --limit Expenses:Food:2000 --limit Expenses:Transport:500`
- **THEN** 创建预算表并显示新 Budget ID

#### Scenario: 无效周期参数
- **WHEN** 执行 `budget create --period biweekly`
- **THEN** 显示错误信息，提示可选值

### Requirement: budget list 命令
CLI SHALL 提供 `budget list` 子命令，以表格形式列出所有预算表的 ID、名称、周期和币种。

#### Scenario: 列出预算表
- **WHEN** 执行 `budget list`
- **THEN** 以表格输出 ID、Name、Period、Commodity 列

### Requirement: budget show 命令
CLI SHALL 提供 `budget show` 子命令，接受 budget_id 参数和可选 `--date` 参数（默认当天），显示预算执行情况。

#### Scenario: 显示当月预算执行情况
- **WHEN** 执行 `budget show 1`
- **THEN** 显示预算周期范围和各账户的 limit/actual/remaining/percentage，超支项标注 ⚠

#### Scenario: 显示指定日期的预算执行情况
- **WHEN** 执行 `budget show 1 --date 2025-12-15`
- **THEN** 显示 2025 年 12 月的预算执行情况

### Requirement: budget update 命令
CLI SHALL 提供 `budget update` 子命令，接受 budget_id 参数和可选的 `--name`、`--period`、`--commodity`、`--limit` 参数。提供 `--limit` 时替换所有限额。

#### Scenario: 更新预算表名称
- **WHEN** 执行 `budget update 1 --name "月度家庭"`
- **THEN** 预算表名称已更新

#### Scenario: 替换限额
- **WHEN** 执行 `budget update 1 --limit Expenses:Food:3000`
- **THEN** 旧限额全部删除，仅剩 1 条新限额

### Requirement: budget delete 命令
CLI SHALL 提供 `budget delete` 子命令，接受 budget_id 参数，删除预算表及所有限额。

#### Scenario: 删除预算表
- **WHEN** 执行 `budget delete 1`
- **THEN** 预算表和所有限额均已删除

### Requirement: --period 参数值
`--period` 参数 SHALL 接受以下值：`daily`、`weekly-sun`、`weekly-mon`、`monthly`、`yearly`。

#### Scenario: weekly-mon 对应 WeeklyFromMonday
- **WHEN** 使用 `--period weekly-mon`
- **THEN** 创建的预算表 period 为 BudgetPeriod::WeeklyFromMonday

### Requirement: --limit 参数格式
`--limit` 参数 SHALL 接受 `<账户路径>:<金额>` 格式，如 `Expenses:Food:2000`。CLI 内部通过账户路径查找账户 ID。

#### Scenario: 路径查找成功
- **WHEN** 使用 `--limit Expenses:Food:2000`
- **THEN** 查找 "Expenses" → "Food" 的账户 ID，关联金额 2000

#### Scenario: 路径不存在
- **WHEN** 使用 `--limit Expenses:NotExist:100`
- **THEN** 显示错误"账户不存在"