# Spec Delta: saving-plan-view

## MODIFIED Requirements

### Requirement: 计划列表展示

攒钱计划页 SHALL 从 GET /api/saving-plans/statuses 获取全部计划状态（按检查点升序），以卡片展示：名称、周期或截止日期、目标金额、满足率环形（ProgressRing）、状态徽标。环形颜色规则：satisfaction 为 100 显示绿色，小于 100 显示黄色，已失效计划显示灰色并带「已失效」徽标。`satisfaction` 显示为归一化百分比（如 75 而非 75.00）。卡片状态徽标 SHALL 按分配口径判定：`satisfaction >= 100` 时显示「已达标」，否则显示缺口徽标，缺口金额为分配口径 `target_amount − allocated`；MUST NOT 用账面口径 `met`/`gap` 判定卡片徽标（共享账户时各计划账面余额相同，会导致所有计划同时显示「已达标」且缺口金额失真）。

#### Scenario: 加载计划列表

- **WHEN** 用户切换到攒钱计划页
- **THEN** 系统调用批量状态端点，按检查点顺序展示计划卡片与满足率环形

#### Scenario: 共享账户计划满足率不同

- **WHEN** 计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,E} 目标 2000），A 3000、B 1000、E 500
- **THEN** 计划 1 环形显示 100（绿色），计划 2 环形显示 75（黄色）

#### Scenario: 共享账户时徽标各自独立

- **WHEN** 计划 1 satisfaction 为 100 而计划 2 satisfaction 小于 100（即使两计划 met 均为 true）
- **THEN** 计划 1 显示「已达标」徽标，计划 2 显示缺口徽标而非「已达标」

#### Scenario: 已失效计划置灰

- **WHEN** 列表中存在 deadline 早于当天的计划
- **THEN** 该卡片环形为灰色并显示「已失效」徽标

#### Scenario: 无攒钱计划

- **WHEN** 没有任何攒钱计划
- **THEN** 显示空状态提示
