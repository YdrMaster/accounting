# saving-plan-cli

## MODIFIED Requirements

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
