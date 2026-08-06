# budget-view

## MODIFIED Requirements

### Requirement: 预算列表展示

预算页 SHALL 从 GET /api/budgets/statuses 获取全部预算表的执行情况（含各预算当前周期状态），以卡片展示：名称、周期类型（或一次性+截止日期）、执行情况环形（ProgressRing）。环形规则：未超支时绿色弧显示已用百分比、环内显示剩余金额；超支时红色环、环内显示「超支 + 金额」。已失效（查询日晚于 deadline）的预算 SHALL 显示「已失效」徽标。

#### Scenario: 加载预算列表

- **WHEN** 用户切换到预算页
- **THEN** 系统调用批量执行情况端点，展示名称、周期类型与执行情况环形

#### Scenario: 超支预算环形为红色

- **WHEN** 某预算某账户实际支出超过限额导致整体超支
- **THEN** 该预算卡片环形为红色，环内显示超支金额

#### Scenario: 已失效预算标注

- **WHEN** 列表中存在 deadline 早于当天的预算
- **THEN** 该卡片显示「已失效」徽标

#### Scenario: 无预算表

- **WHEN** 没有任何预算表
- **THEN** 显示空状态提示

### Requirement: 预算执行情况

点击预算卡片 SHALL 内联展开执行情况明细：各账户的限额、实际金额、剩余、百分比；一次性预算不显示周期区间，周期预算显示当前周期起止日期。再次点击收起。

#### Scenario: 查看执行情况

- **WHEN** 用户点击一个预算表
- **THEN** 展开区展示各账户的限额、实际金额、剩余、百分比

#### Scenario: 超支显示

- **WHEN** 某账户实际支出超过限额
- **THEN** 该账户显示为超支状态（红色标记）

#### Scenario: 一次性预算无周期区间

- **WHEN** 用户展开 period 为 null 的预算
- **THEN** 不显示周期起止日期，仅显示各账户执行情况

### Requirement: 创建预算表

预算页 SHALL 支持通过抽屉创建新预算表，字段：名称、周期类型（daily/weekly-sun/weekly-mon/monthly/yearly/一次性）、deadline（日期，可选）、限额列表（每行账户+金额，账户选择器仅显示 Expenses 子树账户）。币种固定为 CNY。

#### Scenario: 打开创建抽屉

- **WHEN** 用户点击新建预算按钮
- **THEN** 底部滑出抽屉，包含名称、周期类型（含一次性）、deadline、限额列表字段

#### Scenario: 提交创建

- **WHEN** 用户填写完整并确认
- **THEN** 系统调用 POST /api/budgets 创建预算表，成功后刷新列表

#### Scenario: 创建一次性预算

- **WHEN** 用户选择周期为「一次性」并填写 deadline 后提交
- **THEN** 创建 period 为 null、deadline 为所选日期的预算表

#### Scenario: 账户选择器仅显示支出账户

- **WHEN** 用户在限额行打开账户选择器
- **THEN** 仅 Expenses 根账户子树的账户可选

### Requirement: 编辑预算表

预算页 SHALL 支持通过抽屉编辑已有预算表，预填当前数据（period 为 null 时显示为一次性、含 deadline 与限额列表）。

#### Scenario: 打开编辑抽屉

- **WHEN** 用户点击预算表编辑按钮
- **THEN** 底部滑出抽屉，预填充预算表当前数据

#### Scenario: 提交更新

- **WHEN** 用户修改后确认
- **THEN** 系统调用 PUT /api/budgets/:id 更新预算表
