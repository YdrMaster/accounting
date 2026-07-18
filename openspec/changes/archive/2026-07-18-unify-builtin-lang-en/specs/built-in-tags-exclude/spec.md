# built-in-tags-exclude (delta)

## RENAMED Requirements

- FROM: `### Requirement: 中英文双语种子数据`
  TO: `### Requirement: 内置标签多语言系统名字种子`

## MODIFIED Requirements

### Requirement: 内置标签多语言系统名字种子
系统 SHALL 在种子数据中创建 `exclude-from-income-statement` 和 `exclude-from-budget` 两个内置标签实体（`is_system=1`），并在 `tag_names` 中为每个实体按受支持语言（en、zh-CN）插入系统名字（英文 `exclude-from-income-statement`/`不计收支`、`exclude-from-budget`/`不计预算`），分别设为对应语言的显示名。种子内容与建库显示语言无关（见 `entity-names-i18n`）。

#### Scenario: 英文环境初始化
- **WHEN** 使用英文 locale 初始化数据库
- **THEN** 两个内置标签实体的英文显示名分别为 `exclude-from-income-statement` 和 `exclude-from-budget`

#### Scenario: 中文环境初始化
- **WHEN** 使用中文 locale 初始化数据库
- **THEN** 同一批内置标签实体的中文显示名分别为 `不计收支` 和 `不计预算`，数据库标签内容与英文建库无结构差异
