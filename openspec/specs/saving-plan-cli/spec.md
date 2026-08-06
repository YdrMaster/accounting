# saving-plan-cli

## Purpose

攒钱计划命令行接口——为攒钱计划系统提供统一的 CLI 操作入口，覆盖攒钱计划的创建、列出、查看、更新和删除，让用户无需直接操作底层服务即可完成攒钱目标管理；同时规范 `--period`、`--deadline`、`--account` 等参数的取值与格式，保证命令行输入的一致性和可校验性。错误信息 SHALL 支持 i18n 本地化。

## Requirements

### Requirement: saving-plan create 命令
CLI SHALL 提供 `saving-plan create` 子命令，接受 `--name`（计划名称）、`--period`（可选，缺省为一次性/无节奏）、`--deadline`（可选，YYYY-MM-DD）、`--commodity`（币种符号）、`--target`（目标金额）和多个 `--account`（账户路径，可重复指定）参数。

#### Scenario: 创建一次性攒钱计划
- **WHEN** 执行 `saving-plan create --name "旅行基金" --deadline 2026-09-30 --commodity CNY --target 5000 --account Assets:Alipay --account Assets:WeChat`
- **THEN** 创建 period=None 的攒钱计划并显示新 SavingPlan ID

#### Scenario: 创建循环攒钱计划
- **WHEN** 执行 `saving-plan create --name "房租备用金" --period monthly --commodity CNY --target 6000 --account Assets:Bank:CMB`
- **THEN** 创建 period=Monthly 的攒钱计划并显示新 SavingPlan ID

#### Scenario: 无效周期参数
- **WHEN** 执行 `saving-plan create --period biweekly`
- **THEN** 显示本地化错误信息，提示可选值

#### Scenario: 无效 deadline 格式
- **WHEN** 执行 `saving-plan create --deadline 2026/09/30`
- **THEN** 显示本地化错误信息，提示日期格式为 YYYY-MM-DD

### Requirement: saving-plan list 命令
CLI SHALL 提供 `saving-plan list` 子命令，以表格形式列出所有攒钱计划的 ID、名称、周期、截止日期、目标金额、币种和满足率（基于全局资金分配）。

#### Scenario: 列出攒钱计划
- **WHEN** 执行 `saving-plan list`
- **THEN** 以表格输出 ID、名称、周期、截止日期、目标金额、币种、满足率列

#### Scenario: 共享账户的计划满足率不同
- **WHEN** 计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,E} 目标 2000），A 3000、B 1000、E 500
- **THEN** list 中计划 1 满足率为 100，计划 2 满足率为 75

### Requirement: saving-plan show 命令
CLI SHALL 提供 `saving-plan show` 子命令，接受计划名称参数和可选 `--date` 参数（默认当天），显示攒钱计划状态：目标金额、当前余额、缺口、是否达标、满足率、每账户分配明细（余额/被更早计划占用/本计划分配）、当前周期区间。过期计划 SHALL 显示「已失效」标注；未达标计划 SHALL 显示缺口提醒标注。

#### Scenario: 显示攒钱计划状态
- **WHEN** 执行 `saving-plan show "旅行基金"`
- **THEN** 显示目标金额、当前余额、缺口、是否达标、满足率；period 非空时同时显示当前周期区间

#### Scenario: 显示每账户分配明细
- **WHEN** 计划 1（{A,B} 目标 3000）先于计划 2（{A,E} 目标 2000），A 3000、B 1000、E 500，执行 `saving-plan show "计划1"`
- **THEN** 输出包含 A 的分配明细（余额 3000、被占用 0、本计划分配 2000）和 B 的分配明细（余额 1000、被占用 0、本计划分配 1000）

#### Scenario: 显示指定日期的状态
- **WHEN** 执行 `saving-plan show "旅行基金" --date 2026-08-15`
- **THEN** 显示截至 2026-08-15 的攒钱计划状态

#### Scenario: 过期计划显示已失效标注
- **WHEN** 对 deadline 早于查询日的攒钱计划执行 `saving-plan show`
- **THEN** 输出包含「已失效」标注

#### Scenario: 未达标计划显示缺口提醒
- **WHEN** 当前余额低于目标金额
- **THEN** 输出包含未达标提醒标注

#### Scenario: 计划名称不存在
- **WHEN** 执行 `saving-plan show "不存在的计划"`
- **THEN** 返回本地化错误 "攒钱计划 '不存在的计划' 不存在"

### Requirement: saving-plan update 命令
CLI SHALL 提供 `saving-plan update` 子命令，接受计划名称参数和可选的 `--name`、`--period`、`--deadline`、`--commodity`、`--target`、`--account` 参数。未指定的项 SHALL 沿用旧值；提供 `--account` 时 SHALL 替换整个账户集合。`--commodity` 使用币种符号。`--period once` SHALL 将计划置为一次性（period 置空）。

#### Scenario: 更新计划名称
- **WHEN** 执行 `saving-plan update "旅行基金" --name "欧洲旅行基金"`
- **THEN** 计划名称已更新，其余字段保持不变

#### Scenario: 替换账户集合
- **WHEN** 执行 `saving-plan update "旅行基金" --account Assets:Bank:CMB`
- **THEN** 旧账户关联全部删除，账户集合仅剩 Assets:Bank:CMB

#### Scenario: 更新目标金额
- **WHEN** 执行 `saving-plan update "旅行基金" --target 8000`
- **THEN** 目标金额更新为 8000，其余字段保持不变

#### Scenario: --period once 清回一次性
- **WHEN** 对一个 period=Monthly 的计划执行 `saving-plan update "房租备用金" --period once`
- **THEN** 计划 period 置为 None，show 不再显示周期区间

#### Scenario: 计划名称不存在
- **WHEN** 执行 `saving-plan update "不存在的计划" --name "新名称"`
- **THEN** 返回本地化错误 "攒钱计划 '不存在的计划' 不存在"

### Requirement: saving-plan delete 命令
CLI SHALL 提供 `saving-plan delete` 子命令，接受计划名称参数，删除攒钱计划及所有账户关联。

#### Scenario: 删除攒钱计划
- **WHEN** 执行 `saving-plan delete "旅行基金"`
- **THEN** 攒钱计划和所有账户关联均已删除

#### Scenario: 删除不存在的计划
- **WHEN** 执行 `saving-plan delete "不存在的计划"`
- **THEN** 返回本地化错误 "攒钱计划 '不存在的计划' 不存在"

### Requirement: --period 参数值
`--period` 参数 SHALL 接受以下值：`daily`、`weekly-sun`、`weekly-mon`、`monthly`、`yearly`、`once`，与 budget 命令一致。`once` 表示一次性/无节奏（period 为 None）；缺省时 period 同样为 None。

#### Scenario: weekly-mon 对应 WeeklyFromMonday
- **WHEN** 使用 `--period weekly-mon`
- **THEN** 创建的攒钱计划 period 为 FinancePeriod::WeeklyFromMonday

#### Scenario: once 表示一次性
- **WHEN** 使用 `--period once`
- **THEN** 创建的攒钱计划 period 为 None

### Requirement: --account 参数格式
`--account` 参数 SHALL 接受账户路径，如 `Assets:Alipay`，可重复指定多次。CLI 内部通过账户路径查找账户 ID，账户 MUST 位于 Assets 根账户子树内。

#### Scenario: 路径查找成功
- **WHEN** 使用 `--account Assets:Alipay`
- **THEN** 查找 "Assets" → "Alipay" 的账户 ID 并加入账户集合

#### Scenario: 路径不存在
- **WHEN** 使用 `--account Assets:NotExist`
- **THEN** 显示本地化错误"账户不存在"

#### Scenario: 非资产账户被拒绝
- **WHEN** 使用 `--account Expenses:Food`
- **THEN** 显示本地化错误，提示攒钱计划账户必须是资产账户
