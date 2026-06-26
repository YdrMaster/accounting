# built-in-tags-exclude

## Purpose

定义用于排除交易统计的内置系统标签，包括不计收支和不计预算两个独立标签，以及中英文双语种子数据。

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

### Requirement: 中英文双语种子数据
系统 SHALL 在中英文种子数据中各插入 2 条内置标签记录，共 4 条。

#### Scenario: 英文种子数据
- **WHEN** 使用英文 locale 初始化数据库
- **THEN** tags 表包含 `exclude-from-income-statement` 和 `exclude-from-budget` 两条 is_system=1 记录

#### Scenario: 中文种子数据
- **WHEN** 使用中文 locale 初始化数据库
- **THEN** tags 表包含 `不计收支` 和 `不计预算` 两条 is_system=1 记录