# built-in-tags-exclude

## Purpose

定义用于排除交易统计的内置系统标签，包括不计收支和不计预算两个独立标签，以及按语言存储的多语言系统名字种子数据。

## Requirements

### Requirement: 内置标签不计收支
系统 SHALL 在种子数据中新增 `不计收支`（英文 `exclude-from-income-statement`）系统标签。带有此标签的交易不计入收支统计。

#### Scenario: 种子数据包含不计收支标签
- **WHEN** 数据库初始化完成
- **THEN** tags 表中存在 is_system=1、name 为 `exclude-from-income-statement` 的记录

#### Scenario: 不计收支标签为系统标签
- **WHEN** 查看不计收支标签的 is_system 字段
- **THEN** is_system = true，标签不可删除

### Requirement: 内置标签不计预算
系统 SHALL 在种子数据中新增 `不计预算`（英文 `exclude-from-budget`）系统标签。带有此标签的交易不计入预算统计。

#### Scenario: 种子数据包含不计预算标签
- **WHEN** 数据库初始化完成
- **THEN** tags 表中存在 is_system=1、name 为 `exclude-from-budget` 的记录

#### Scenario: 不计预算标签为系统标签
- **WHEN** 查看不计预算标签的 is_system 字段
- **THEN** is_system = true，标签不可删除

### Requirement: 两个标签互相独立
`不计收支` 和 `不计预算` 标签 SHALL 互相独立。标记一个标签不自动应用另一个标签。一个交易可同时带两个标签、只带一个、或不带。

#### Scenario: 只标记不计收支
- **WHEN** 交易只有"不计收支"标签
- **THEN** 该交易不计入收支统计，但仍计入预算统计

#### Scenario: 只标记不计预算
- **WHEN** 交易只有"不计预算"标签
- **THEN** 该交易不计入预算统计，但仍计入收支统计

#### Scenario: 同时标记两个标签
- **WHEN** 交易同时有"不计收支"和"不计预算"标签
- **THEN** 该交易同时不计入收支统计和预算统计

### Requirement: 内置标签多语言系统名字种子
系统 SHALL 在种子数据中创建 `exclude-from-income-statement` 和 `exclude-from-budget` 两个内置标签实体（`is_system=1`），并在 `tag_names` 中为每个实体按受支持语言（en、zh-CN）插入系统名字（英文 `exclude-from-income-statement`/`不计收支`、`exclude-from-budget`/`不计预算`），分别设为对应语言的显示名。种子内容与建库显示语言无关（见 `entity-names-i18n`）。

#### Scenario: 英文环境初始化
- **WHEN** 使用英文 locale 初始化数据库
- **THEN** 两个内置标签实体的英文显示名分别为 `exclude-from-income-statement` 和 `exclude-from-budget`

#### Scenario: 中文环境初始化
- **WHEN** 使用中文 locale 初始化数据库
- **THEN** 同一批内置标签实体的中文显示名分别为 `不计收支` 和 `不计预算`，数据库标签内容与英文建库无结构差异