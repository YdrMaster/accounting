## ADDED Requirements

### Requirement: 交易表单覆盖层
系统 SHALL 提供覆盖整个 pane 区域的交易表单覆盖层，用于创建和编辑交易。覆盖层从 pane 顶部滑入，包含表单字段和分录列表。

#### Scenario: 打开新建表单
- **WHEN** 用户点击交易页或日历页的新建按钮
- **THEN** 交易表单覆盖层滑入，显示空白表单，日期默认为当前日期时间

#### Scenario: 打开编辑表单
- **WHEN** 用户通过双击或左滑交易卡片触发编辑
- **THEN** 交易表单覆盖层滑入，表单预填充该交易的所有字段（日期、备注、成员、标签、渠道链路、分录）

#### Scenario: 关闭覆盖层
- **WHEN** 用户点击取消按钮或关闭按钮
- **THEN** 覆盖层滑出关闭，表单数据丢弃

### Requirement: 交易表单字段
交易表单 SHALL 包含以下字段：日期时间、备注（可选）、成员选择、标签（可选多选）、渠道链路（可选）、分录列表。

#### Scenario: 必填字段
- **WHEN** 用户提交交易
- **THEN** 系统验证日期时间已填写、至少有两笔有效分录

#### Scenario: 备注字段
- **WHEN** 用户输入备注
- **THEN** 备注保存为交易描述（description）

#### Scenario: 成员选择
- **WHEN** 表单加载
- **THEN** 成员选择默认为当前用户（GET /api/me 返回的成员）

### Requirement: 分录管理
交易表单 SHALL 支持动态添加和删除分录行。每行分录包含账户选择、币种、金额、报销标记。

#### Scenario: 添加分录
- **WHEN** 用户点击"添加分录"按钮
- **THEN** 新增一行分录，币种默认 CNY，金额自动填入配平值（= -(已有分录金额之和)）

#### Scenario: 删除分录
- **WHEN** 用户点击分录行的删除按钮
- **THEN** 该行分录被移除

#### Scenario: 最少两笔分录
- **WHEN** 表单中有效分录少于两笔
- **THEN** 确认按钮禁用

### Requirement: 自动配平
添加新分录时，系统 SHALL 自动计算配平金额，使所有分录金额之和为零。

#### Scenario: 添加配平分录
- **WHEN** 已有分录金额分别为 -100 和 -50，用户点击添加分录
- **THEN** 新分录金额自动填入 150

#### Scenario: 已平衡时添加
- **WHEN** 已有分录金额之和为 0，用户点击添加分录
- **THEN** 新分录金额字段为空

### Requirement: 借贷平衡校验
提交按钮 SHALL 在所有分录金额之和不为零时禁用。

#### Scenario: 不平衡时禁用
- **WHEN** 分录金额之和不为 0
- **THEN** 确认按钮置灰禁用，不显示额外提示文字

#### Scenario: 平衡时可提交
- **WHEN** 分录金额之和为 0 且至少有两条有效分录
- **THEN** 确认按钮可用

### Requirement: 账户选择覆盖层
分录行的账户选择 SHALL 点击后打开覆盖层，显示现有账户网格。选中账户后高亮显示，点击确认按钮关闭覆盖层。

#### Scenario: 打开账户选择
- **WHEN** 用户点击分录行的账户选择触发器
- **THEN** 账户选择覆盖层打开，显示按类型分组的账户网格

#### Scenario: 选中账户
- **WHEN** 用户在账户网格中点击一个账户
- **THEN** 该账户高亮选中，覆盖层不关闭

#### Scenario: 确认选择
- **WHEN** 用户点击确认按钮
- **THEN** 覆盖层关闭，分录行显示选中的账户名称

#### Scenario: 不弹编辑抽屉
- **WHEN** 用户在账户选择覆盖层中点击账户
- **THEN** 不弹出账户编辑抽屉（与账户页行为不同）

### Requirement: 标签输入
标签字段 SHALL 使用标签输入控件，支持从已有标签选择和输入新标签名称。

#### Scenario: 选择已有标签
- **WHEN** 用户在标签输入框中搜索
- **THEN** 显示匹配的已有标签列表供选择

#### Scenario: 创建新标签
- **WHEN** 用户输入不存在的标签名称并提交
- **THEN** 后端自动创建该标签（POST /api/transactions 已支持自动创建）

### Requirement: 渠道链路输入
渠道链路 SHALL 使用分级标签输入控件。每一级为一个槽位，前几级单选，最后一级支持多选。

#### Scenario: 添加第一级渠道
- **WHEN** 用户点击添加渠道
- **THEN** 显示第一级槽位，从渠道列表选择一个渠道

#### Scenario: 添加下一级渠道
- **WHEN** 用户在已有槽位后点击添加
- **THEN** 新增一级槽位，原最后一级变为单选

#### Scenario: 最后一级多选
- **WHEN** 用户在最后一级槽位添加多个渠道
- **THEN** 多个渠道以标签 chips 形式显示在同一级

#### Scenario: 提交渠道链路
- **WHEN** 用户提交交易
- **THEN** 渠道链路序列化为 ChannelPathNodeRequest 数组，同一 position 的多个渠道生成多条记录

### Requirement: 提交创建交易
表单提交 SHALL 调用 POST /api/transactions 创建交易。

#### Scenario: 创建成功
- **WHEN** 用户填写完整表单并点击确认
- **THEN** 系统调用 API 创建交易，成功后关闭覆盖层并刷新交易列表

#### Scenario: 创建失败
- **WHEN** API 返回错误
- **THEN** 覆盖层内显示错误信息，表单数据保留

### Requirement: 提交更新交易
编辑模式提交 SHALL 调用 PUT /api/transactions/:id 更新交易。

#### Scenario: 更新成功
- **WHEN** 用户修改表单后点击确认
- **THEN** 系统调用 API 更新交易，成功后关闭覆盖层并刷新交易列表

### Requirement: 删除交易
交易卡片 SHALL 支持删除操作。

#### Scenario: 右滑删除
- **WHEN** 用户在交易卡片上右滑超过阈值
- **THEN** 弹出确认对话框，确认后调用 DELETE /api/transactions/:id

#### Scenario: 删除成功
- **WHEN** API 删除成功
- **THEN** 交易从列表中移除，显示成功提示
