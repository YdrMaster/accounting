## MODIFIED Requirements

### Requirement: 分录展开显示
所有账单 SHALL 支持点击展开查看完整分录列表，并支持通过双击或滑动手势触发编辑和删除操作。分录列表直接展示分录条目，不显示"分录："标题行。

#### Scenario: 点击展开分录
- **WHEN** 用户点击账单卡片
- **THEN** 在账单卡片下方展开显示该账单的所有分录

#### Scenario: 再次点击折叠分录
- **WHEN** 用户点击已展开的账单卡片
- **THEN** 分录列表折叠隐藏

#### Scenario: 分录显示内容
- **WHEN** 分录列表展开
- **THEN** 每条分录显示账户名（短名）、商品代码和金额，金额正值显示绿色，负值显示红色

## ADDED Requirements

### Requirement: 交易卡片编辑入口
交易卡片 SHALL 支持通过双击（桌面端）或左滑（移动端）触发编辑操作。

#### Scenario: 双击编辑
- **WHEN** 用户在桌面端双击交易卡片
- **THEN** 打开交易表单覆盖层，预填充该交易数据

#### Scenario: 左滑编辑
- **WHEN** 用户在交易卡片上左滑超过阈值
- **THEN** 打开交易表单覆盖层，预填充该交易数据

### Requirement: 交易卡片删除入口
交易卡片 SHALL 支持通过右滑触发删除操作。

#### Scenario: 右滑删除
- **WHEN** 用户在交易卡片上右滑超过阈值（60px）
- **THEN** 弹出确认对话框

#### Scenario: 确认删除
- **WHEN** 用户在确认对话框中点击删除
- **THEN** 调用 DELETE /api/transactions/:id，成功后从列表移除该交易

#### Scenario: 取消删除
- **WHEN** 用户在确认对话框中点击取消
- **THEN** 对话框关闭，交易保持不变
