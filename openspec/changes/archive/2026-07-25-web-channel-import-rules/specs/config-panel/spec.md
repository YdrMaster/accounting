# config-panel 增量

## MODIFIED Requirements

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

## ADDED Requirements

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
