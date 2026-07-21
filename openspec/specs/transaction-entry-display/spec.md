# transaction-entry-display

## Purpose

交易分录展开显示功能——定义账单列表页中交易卡片的展示与交互规则。支持检测纯导入账单并简化其显示内容，点击卡片可展开/折叠完整分录列表（带平滑动画），并提供双击、左滑编辑与右滑删除等手势入口，改善账单浏览与操作体验。

## Requirements

### Requirement: 纯导入账单检测
系统 SHALL 提供 `isPureImport(tx)` 检测函数，当账单的所有分录的账户路径都包含 `:Import:` 段时返回 true。

#### Scenario: 纯导入账单
- **WHEN** 账单的所有分录账户路径都包含 `:Import:`
- **THEN** `isPureImport(tx)` 返回 true

#### Scenario: 正常账单
- **WHEN** 账单至少有一个分录的账户路径不包含 `:Import:`
- **THEN** `isPureImport(tx)` 返回 false

### Requirement: 纯导入账单显示
对于纯导入账单，UI SHALL 隐藏收支账户名称、金额显示和资产账户名称，仅保留备注（description）、成员（member_name）和标签（tags）。

#### Scenario: 纯导入账单不显示金额
- **WHEN** 账单被判定为纯导入账单
- **THEN** 不显示 tx-amount 区域（金额显示为 ¥0.00 的部分）

#### Scenario: 纯导入账单不显示收支账户
- **WHEN** 账单被判定为纯导入账单
- **THEN** 不显示 tx-top 中的收支账户名称（ie-accounts）

#### Scenario: 纯导入账单不显示资产账户
- **WHEN** 账单被判定为纯导入账单
- **THEN** 不显示 tx-bottom 中的资产账户名称（asset-accounts）

#### Scenario: 纯导入账单保留基本信息
- **WHEN** 账单被判定为纯导入账单
- **THEN** 仍然显示 description、member_name 和 tags

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

### Requirement: 展开状态独立控制
每个账单的展开/折叠状态 SHALL 独立控制，互不影响。

#### Scenario: 多个账单独立展开
- **WHEN** 用户展开账单 A，然后展开账单 B
- **THEN** 账单 A 和账单 B 的分录列表同时显示

### Requirement: 展开折叠动画
分录区域的展开和折叠 SHALL 使用平滑的 CSS 过渡动画。

#### Scenario: 展开动画
- **WHEN** 分录区域从折叠变为展开
- **THEN** 使用 max-height 过渡实现平滑展开动画，持续时间约 0.3 秒

#### Scenario: 折叠动画
- **WHEN** 分录区域从展开变为折叠
- **THEN** 使用 max-height 过渡实现平滑折叠动画，持续时间约 0.3 秒

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
