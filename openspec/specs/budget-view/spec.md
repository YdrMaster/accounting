# budget-view

预算页面——展示预算表列表、执行情况，支持预算表 CRUD 操作。

## Requirements

### Requirement: 预算列表展示
预算页 SHALL 从 GET /api/budgets 获取预算表列表并展示。

#### Scenario: 加载预算列表
- **WHEN** 用户切换到预算页
- **THEN** 系统调用 API 获取所有预算表，展示名称、周期类型

#### Scenario: 无预算表
- **WHEN** 没有任何预算表
- **THEN** 显示空状态提示

### Requirement: 预算执行情况
预算页 SHALL 展示选中预算表的执行情况。

#### Scenario: 查看执行情况
- **WHEN** 用户选择一个预算表
- **THEN** 系统调用 GET /api/budgets/:id/status 获取执行情况，展示各账户的限额、实际金额、剩余、百分比

#### Scenario: 超支显示
- **WHEN** 某账户实际支出超过限额
- **THEN** 该账户显示为超支状态（红色标记）

### Requirement: 创建预算表
预算页 SHALL 支持通过抽屉创建新预算表。

#### Scenario: 打开创建抽屉
- **WHEN** 用户点击新建预算按钮
- **THEN** 底部滑出抽屉，包含名称、周期类型、币种、限额列表字段

#### Scenario: 提交创建
- **WHEN** 用户填写完整并确认
- **THEN** 系统调用 POST /api/budgets 创建预算表，成功后刷新列表

### Requirement: 编辑预算表
预算页 SHALL 支持通过抽屉编辑已有预算表。

#### Scenario: 打开编辑抽屉
- **WHEN** 用户点击预算表编辑按钮
- **THEN** 底部滑出抽屉，预填充预算表当前数据

#### Scenario: 提交更新
- **WHEN** 用户修改后确认
- **THEN** 系统调用 PUT /api/budgets/:id 更新预算表

### Requirement: 删除预算表
预算页 SHALL 支持删除预算表。

#### Scenario: 确认删除
- **WHEN** 用户点击删除按钮
- **THEN** 弹出确认对话框，确认后调用 DELETE /api/budgets/:id

#### Scenario: 删除成功
- **WHEN** API 删除成功
- **THEN** 预算表从列表移除
