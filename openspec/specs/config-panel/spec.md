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
渠道 tab SHALL 将渠道显示为卡片列表。每张卡片在折叠状态下 SHALL 显示渠道名称。展开时，卡片 SHALL 显示名称、描述和关联账户，每个字段都有编辑控件。同时只 SHALL 展开一张卡片。卡片头部操作位 SHALL 按渠道类型显示不同按钮：用户渠道（`is_system=false`）显示删除按钮；关联了导入适配器的渠道（`has_import_adapter=true`）显示导入按钮；其余渠道（无适配器的系统渠道）不显示任何按钮。

#### Scenario: 列出折叠的渠道
- **WHEN** 渠道 tab 激活时
- **THEN** 每个渠道显示为卡片，用户渠道显示名称和删除按钮，适配器渠道显示名称和导入按钮，无适配器的系统渠道仅显示名称

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
渠道 tab SHALL 支持通过行内输入添加新渠道和删除用户创建的渠道。系统内置渠道（`is_system=true`）的卡片 SHALL NOT 显示删除按钮；其改名、描述与关联账户编辑保持开放。

#### Scenario: 添加渠道
- **WHEN** 用户在输入框中输入名称并点击"添加"或按回车
- **THEN** 通过 `POST /api/channels` 创建新渠道，列表刷新

#### Scenario: 删除渠道
- **WHEN** 用户点击用户创建渠道（`is_system=false`）上的删除按钮（×）
- **THEN** 通过 `DELETE /api/channels/{id}` 删除渠道，列表刷新

#### Scenario: 内置渠道不显示删除按钮
- **WHEN** 渠道 tab 显示一个 `is_system=true` 的渠道卡片
- **THEN** 该卡片（折叠与展开状态）均不显示删除按钮

#### Scenario: 删除正在使用的渠道
- **WHEN** 用户尝试删除被交易引用的渠道
- **THEN** 后端返回错误，UI 显示错误消息

### Requirement: 渠道导入规则配置
展开的渠道卡片 SHALL 仅在渠道关联了导入适配器时（`has_import_adapter=true`，即渠道的任一语言名字能匹配某个内置账单适配器）显示"导入规则"区块，用于管理该渠道的账户映射。普通用户渠道 SHALL NOT 显示该区块。区块 SHALL 包含成员切换器（列出所有成员，默认选中第一个）、当前 (成员, 渠道) 的映射列表（每行显示 category、目标账户名称和删除按钮），以及添加行（分类文本输入、账户选择控件、添加按钮）。

#### Scenario: 适配器渠道显示导入规则区块
- **WHEN** 用户展开关联了导入适配器的渠道（如内置渠道"支付宝"）的卡片
- **THEN** 卡片展开区显示"导入规则"区块

#### Scenario: 普通渠道不显示导入规则区块
- **WHEN** 用户展开未关联导入适配器的用户渠道的卡片
- **THEN** 卡片展开区不显示"导入规则"区块

#### Scenario: 切换成员加载映射
- **WHEN** 用户在导入规则区块中切换成员
- **THEN** 通过 `GET /api/mappings?member_id=<id>&channel_id=<渠道id>` 重新加载并显示该 (成员, 渠道) 的映射列表

#### Scenario: 添加映射
- **WHEN** 用户输入分类（如 `Expenses:餐饮美食`）、选择目标账户并点击"添加"
- **THEN** 通过 `PUT /api/mappings` 设置映射，映射列表刷新显示新记录

#### Scenario: 删除映射
- **WHEN** 用户点击映射行上的删除按钮（×）
- **THEN** 通过 `DELETE /api/mappings` 删除映射，列表刷新

#### Scenario: 映射目标账户显示名称
- **WHEN** 映射列表渲染一行映射
- **THEN** 目标账户按 `account_id` 从账户数据解析显示名称，账户已不存在时显示原始 ID

### Requirement: 渠道卡片导入账单
关联了导入适配器的渠道卡片 SHALL 提供两种导入触发方式：点击卡片头部的"导入"按钮打开系统文件选择框；将文件拖拽到卡片上（拖拽悬停时卡片 SHALL 高亮提示）后松手。任一方式选中文件后 SHALL 先弹出成员确认对话框，用户确认后才通过 `POST /api/channels/{id}/import?member_id=<成员id>`（body 为文件原始字节）执行导入。导入进行中 SHALL 显示 loading 状态并忽略重复的拖放与点击。无适配器的渠道卡片 SHALL NOT 响应文件拖放。

#### Scenario: 点击导入按钮选择文件
- **WHEN** 用户点击适配器渠道卡片的"导入"按钮，在文件选择框中选中一个账单文件
- **THEN** 弹出成员确认对话框，确认后立即以该文件字节为 body 发起导入请求

#### Scenario: 拖拽文件到卡片
- **WHEN** 用户将账单文件拖到适配器渠道卡片上并松手
- **THEN** 拖拽悬停期间卡片高亮，松手后弹出成员确认对话框，确认后立即以该文件发起导入请求

#### Scenario: 导入中忽略重复触发
- **WHEN** 一次导入请求尚未完成，用户再次拖放文件或点击导入按钮
- **THEN** 不发起新的导入请求

#### Scenario: 无适配器渠道不响应拖放
- **WHEN** 用户将文件拖到无适配器的渠道卡片上
- **THEN** 卡片不高亮，松手后不发起任何请求

### Requirement: 导入成员确认对话框
文件选定或拖入后、发起导入请求前，SHALL 弹出成员确认对话框。对话框 SHALL 包含成员下拉框（默认选中第一个成员，可改选）和确认、取消按钮。确认 SHALL 以下拉框选中的成员作为 `member_id` 发起导入；取消 SHALL 放弃本次导入且不发起任何请求。

#### Scenario: 确认导入
- **WHEN** 用户在成员确认对话框中保持或改选成员后点击确认
- **THEN** 以选中成员作为 `member_id` 发起导入请求

#### Scenario: 取消导入
- **WHEN** 用户在成员确认对话框中点击取消
- **THEN** 对话框关闭，不发起任何导入请求

#### Scenario: 默认成员
- **WHEN** 成员确认对话框打开
- **THEN** 成员下拉框默认选中成员列表的第一个成员

### Requirement: 导入结果摘要反馈
导入完成后 SHALL 显示摘要 toast：成功时显示"导入 N 条，跳过 M 条"；存在跳过记录时 SHALL 支持展开查看逐行原因（行号 + 错误描述）。导入失败（渠道不支持、解析失败、服务错误）时 SHALL 显示错误 toast。toast SHALL 在数秒后自动消失。

#### Scenario: 全部成功的摘要
- **WHEN** 导入完成，`imported=320`、`skipped=0`
- **THEN** toast 显示"导入 320 条，跳过 0 条"，不提供展开入口

#### Scenario: 含跳过记录的摘要
- **WHEN** 导入完成，`imported=318`、`skipped=2` 且 `errors` 含两条记录
- **THEN** toast 显示"导入 318 条，跳过 2 条"，展开后列出两行的行号与原因

#### Scenario: 导入失败
- **WHEN** 导入请求返回错误（如文件无法解析）
- **THEN** 显示错误 toast，展示失败原因
