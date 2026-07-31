# budget-cli

## Purpose

预算命令行接口——为预算系统提供统一的 CLI 操作入口，覆盖预算表的创建、列出、查看、更新和删除，让用户无需直接操作底层服务即可完成预算管理；同时规范 `--period`、`--limit` 等参数的取值与格式，保证命令行输入的一致性和可校验性。

## Requirements

### Requirement: budget create 命令
CLI SHALL 提供 `budget create` 子命令，接受 `--name`（预算表名称）、`--period`（周期类型，可选，缺省表示一次性预算）、`--deadline`（截止日期 YYYY-MM-DD，可选）、`--commodity`（币种符号）和多个 `--limit`（账户路径:金额）参数。

#### Scenario: 创建月度预算
- **WHEN** 执行 `budget create --name "月度生活" --period monthly --commodity CNY --limit Expenses:Food:2000 --limit Expenses:Transport:500`
- **THEN** 创建预算表并显示新 Budget ID

#### Scenario: 创建一次性预算
- **WHEN** 执行 `budget create --name "旅行预算" --deadline 2026-09-30 --commodity CNY --limit Expenses:Travel:8000`（不提供 --period）
- **THEN** 创建 period 为空、deadline 为 2026-09-30 的预算表

#### Scenario: 无效周期参数
- **WHEN** 执行 `budget create --period biweekly`
- **THEN** 显示错误信息，提示可选值

### Requirement: budget list 命令
CLI SHALL 提供 `budget list` 子命令，以表格形式列出所有预算表的 ID、名称、周期和币种。

#### Scenario: 列出预算表
- **WHEN** 执行 `budget list`
- **THEN** 以表格输出 ID、Name、Period、Commodity 列

### Requirement: budget show 命令
CLI SHALL 提供 `budget show` 子命令，接受预算表名称参数和可选 `--date` 参数（默认当天），显示预算执行情况。已失效（查询日晚于 deadline）的预算 SHALL 显示「已失效」标注；一次性预算不显示周期区间。

#### Scenario: 显示当月预算执行情况
- **WHEN** 执行 `budget show "月度生活"`
- **THEN** 显示预算周期范围和各账户的 limit/actual/remaining/percentage，超支项标注 ⚠

#### Scenario: 显示指定日期的预算执行情况
- **WHEN** 执行 `budget show "月度生活" --date 2025-12-15`
- **THEN** 显示 2025 年 12 月的预算执行情况

#### Scenario: 显示已失效预算
- **WHEN** 执行 `budget show "旅行预算"` 且当前日期晚于其 deadline
- **THEN** 显示执行情况并标注「已失效」

#### Scenario: 预算名称不存在
- **WHEN** 执行 `budget show "不存在的预算"`
- **THEN** 返回错误 "预算表 '不存在的预算' 不存在"

### Requirement: budget update 命令
CLI SHALL 提供 `budget update` 子命令，接受预算表名称参数和可选的 `--name`、`--period`、`--deadline`、`--commodity`、`--limit` 参数。提供 `--limit` 时替换所有限额。`--commodity` 使用币种符号。`--deadline none` SHALL 清除截止日期。`--period once` SHALL 将预算置为一次性（period 置空）。

#### Scenario: 更新预算表名称
- **WHEN** 执行 `budget update "月度生活" --name "月度家庭"`
- **THEN** 预算表名称已更新

#### Scenario: 替换限额
- **WHEN** 执行 `budget update "月度生活" --limit Expenses:Food:3000`
- **THEN** 旧限额全部删除，仅剩 1 条新限额

#### Scenario: 更新预算币种
- **WHEN** 执行 `budget update "月度生活" --commodity USD`
- **THEN** 预算表币种更新为 USD 对应币种

#### Scenario: 设置与清除 deadline
- **WHEN** 执行 `budget update "月度生活" --deadline 2026-12-31`，随后执行 `budget update "月度生活" --deadline none`
- **THEN** 第一次 deadline 设为 2026-12-31，第二次 deadline 被清除

#### Scenario: --period once 清回一次性
- **WHEN** 对一个 period=Monthly 的预算执行 `budget update "月度生活" --period once`
- **THEN** 预算 period 置为 None，show 不再显示周期区间

#### Scenario: 预算名称不存在
- **WHEN** 执行 `budget update "不存在的预算" --name "新名称"`
- **THEN** 返回错误 "预算表 '不存在的预算' 不存在"

### Requirement: budget delete 命令
CLI SHALL 提供 `budget delete` 子命令，接受预算表名称参数，删除预算表及所有限额。

#### Scenario: 删除预算表
- **WHEN** 执行 `budget delete "月度生活"`
- **THEN** 预算表和所有限额均已删除

#### Scenario: 删除不存在的预算表
- **WHEN** 执行 `budget delete "不存在的预算"`
- **THEN** 返回错误 "预算表 '不存在的预算' 不存在"

### Requirement: --period 参数值
`--period` 参数 SHALL 接受以下值：`daily`、`weekly-sun`、`weekly-mon`、`monthly`、`yearly`、`once`。`once` 表示一次性预算（period 为 None）；缺省时 period 同样为 None。

#### Scenario: weekly-mon 对应 WeeklyFromMonday
- **WHEN** 使用 `--period weekly-mon`
- **THEN** 创建的预算表 period 为 FinancePeriod::WeeklyFromMonday

#### Scenario: once 表示一次性
- **WHEN** 使用 `--period once`
- **THEN** 创建的预算表 period 为 None

### Requirement: --limit 参数格式
`--limit` 参数 SHALL 接受 `<账户路径>:<金额>` 格式，如 `Expenses:Food:2000`。CLI 内部通过账户路径查找账户 ID。

#### Scenario: 路径查找成功
- **WHEN** 使用 `--limit Expenses:Food:2000`
- **THEN** 查找 "Expenses" → "Food" 的账户 ID，关联金额 2000

#### Scenario: 路径不存在
- **WHEN** 使用 `--limit Expenses:NotExist:100`
- **THEN** 显示错误"账户不存在"