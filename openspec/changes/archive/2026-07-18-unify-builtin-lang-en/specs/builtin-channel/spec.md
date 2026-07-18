# builtin-channel (delta)

## MODIFIED Requirements

### Requirement: 内置渠道种子数据
系统 SHALL 在种子数据中创建一个内置渠道实体（`is_system=1`），并在 `channel_names` 中为其按受支持语言（en、zh-CN）插入系统名字（`is_system=1`，英文 `Alipay`、中文 `支付宝`），分别设为对应语言的显示名。种子内容与建库显示语言无关（见 `entity-names-i18n`、`builtin-data-english-storage`）。

#### Scenario: 英文环境初始化
- **WHEN** 以 `lang="en"` 初始化数据库
- **THEN** 存在一个 `is_system=1` 的渠道，其英文显示名为 "Alipay"

#### Scenario: 中文环境初始化
- **WHEN** 以 `lang="zh"` 初始化数据库
- **THEN** 同一个内置渠道实体同时拥有中文系统名 "支付宝" 并设为中文显示名，数据库渠道内容与英文建库无结构差异
