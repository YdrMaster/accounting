# 配置面板

## Purpose

配置面板是统一管理记账基础数据的配置界面，覆盖成员、渠道、标签三类数据的增删改查，以及渠道与账户的关联设置。它以底部抽屉覆盖层形式从页面头部打开，让用户无需跳转页面即可维护基础配置，解决基础数据分散、管理入口不统一的问题。

## Requirements

### Requirement: 配置面板入口
系统 SHALL 在 PageSwitcher 头部显示齿轮图标按钮，位于导航箭头右侧。点击该按钮 SHALL 打开配置面板。

#### Scenario: 打开配置面板
- **WHEN** 用户点击 PageSwitcher 中的齿轮图标
- **THEN** 配置面板作为底部抽屉覆盖层打开

#### Scenario: 关闭配置面板
- **WHEN** 用户点击抽屉头部的关闭按钮（×）
- **THEN** 配置面板关闭，底层页面恢复交互

### Requirement: 配置面板覆盖层行为
配置面板 SHALL 渲染为底部抽屉覆盖层，覆盖整个视口，带半透明背景。抽屉内容 SHALL 从底部滑入，`max-height: 66vh`。背景 SHALL 在面板打开时阻止与底层内容的所有交互。

#### Scenario: 覆盖层阻止底层交互
- **WHEN** 配置面板打开时
- **THEN** 点击背景区域不会触发底层页面的任何操作

#### Scenario: 抽屉动画
- **WHEN** 配置面板打开时
- **THEN** 抽屉内容从底部平滑滑入

### Requirement: Tab 导航
配置面板 SHALL 包含三个 tab：成员、渠道、标签。默认 SHALL 选中第一个 tab。点击 tab SHALL 切换显示内容。

#### Scenario: 默认 tab 选中
- **WHEN** 配置面板打开时
- **THEN** 成员 tab 处于激活状态并显示其内容

#### Scenario: 切换 tab
- **WHEN** 用户点击渠道 tab
- **THEN** 显示渠道管理内容

### Requirement: 成员管理
成员 tab SHALL 显示所有成员的列表。每个成员条目 SHALL 显示成员名称和删除按钮。面板 SHALL 支持通过行内输入框添加新成员，并通过行内编辑重命名成员。

#### Scenario: 列出成员
- **WHEN** 成员 tab 激活时
- **THEN** 所有成员以列表形式显示，包含名称和删除按钮

#### Scenario: 添加成员
- **WHEN** 用户在输入框中输入名称并点击"添加"或按回车
- **THEN** 通过 `POST /api/members` 创建新成员，列表刷新

#### Scenario: 重命名成员
- **WHEN** 用户点击成员名称，编辑文本，按回车或失去焦点
- **THEN** 通过 `PUT /api/members/{id}` 重命名成员，列表更新

#### Scenario: 删除成员
- **WHEN** 用户点击成员上的删除按钮（×）
- **THEN** 通过 `DELETE /api/members/{id}` 删除成员，列表刷新

### Requirement: 标签管理
标签 tab SHALL 显示所有标签的列表。每个标签条目 SHALL 显示标签名称、描述（如有）和删除按钮。面板 SHALL 支持添加新标签和通过行内编辑重命名标签。

#### Scenario: 列出标签
- **WHEN** 标签 tab 激活时
- **THEN** 所有标签以名称、描述和删除按钮显示

#### Scenario: 添加标签
- **WHEN** 用户在输入框中输入名称并点击"添加"或按回车
- **THEN** 通过 `POST /api/tags` 创建新标签，列表刷新

#### Scenario: 重命名标签
- **WHEN** 用户点击标签名称，编辑文本，按回车或失去焦点
- **THEN** 通过 `PUT /api/tags/{id}` 重命名标签，列表更新

#### Scenario: 删除标签
- **WHEN** 用户点击非系统标签上的删除按钮（×）
- **THEN** 通过 `DELETE /api/tags/{id}` 删除标签，列表刷新

### Requirement: 可展开卡片的渠道管理
渠道 tab SHALL 将渠道显示为卡片列表。每张卡片在折叠状态下 SHALL 显示渠道名称。展开时，卡片 SHALL 显示名称、描述和关联账户，每个字段都有编辑控件。同时只 SHALL 展开一张卡片。

#### Scenario: 列出折叠的渠道
- **WHEN** 渠道 tab 激活时
- **THEN** 每个渠道显示为卡片，仅显示名称和删除按钮

#### Scenario: 展开渠道卡片
- **WHEN** 用户点击折叠的渠道卡片
- **THEN** 卡片展开，显示名称（可编辑）、描述（可编辑）和关联账户（带选择按钮）

#### Scenario: 折叠渠道卡片
- **WHEN** 用户点击展开的渠道卡片头部
- **THEN** 卡片折叠回仅显示名称

#### Scenario: 同时只展开一张卡片
- **WHEN** 一张渠道卡片展开，用户点击另一张卡片
- **THEN** 之前展开的卡片折叠，新卡片展开

### Requirement: 渠道关联账户
展开的渠道卡片 SHALL 显示关联账户名称（如无则显示"未关联"）。"选择"按钮 SHALL 打开 AccountPickerOverlay 选择账户。选择账户 SHALL 更新渠道的关联账户。

#### Scenario: 选择关联账户
- **WHEN** 用户在展开的渠道卡片中点击"选择"
- **THEN** AccountPickerOverlay 打开

#### Scenario: 账户已选择
- **WHEN** 用户从选择器中选择一个账户
- **THEN** 通过 `PUT /api/channels/{id}` 更新渠道的 account_id，卡片显示新账户名称

### Requirement: 渠道添加和删除
渠道 tab SHALL 支持通过行内输入添加新渠道和删除渠道。

#### Scenario: 添加渠道
- **WHEN** 用户在输入框中输入名称并点击"添加"或按回车
- **THEN** 通过 `POST /api/channels` 创建新渠道，列表刷新

#### Scenario: 删除渠道
- **WHEN** 用户点击渠道上的删除按钮（×）
- **THEN** 通过 `DELETE /api/channels/{id}` 删除渠道，列表刷新

#### Scenario: 删除正在使用的渠道
- **WHEN** 用户尝试删除被交易引用的渠道
- **THEN** 后端返回错误，UI 显示错误消息
