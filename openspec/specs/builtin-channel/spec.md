# builtin-channel

## Purpose

系统内置渠道——标记为 `is_system=1` 的渠道，在数据库初始化时自动创建，提供导入适配器的默认渠道关联。

## Requirements

### Requirement: 渠道系统内置标记
`channels` 表 SHALL 包含 `is_system` 字段（`INTEGER NOT NULL DEFAULT 0`），标识系统内置渠道。领域模型 `Channel` SHALL 包含 `is_system: bool` 字段。

#### Scenario: 内置渠道标记
- **WHEN** 数据库初始化种子数据
- **THEN** 内置渠道的 `is_system` 为 1，用户创建的渠道 `is_system` 为 0

#### Scenario: Channel 领域模型包含 is_system
- **WHEN** 从数据库加载一个内置渠道
- **THEN** `Channel.is_system` 为 `true`

### Requirement: 内置渠道种子数据
系统 SHALL 在种子数据中创建一个内置渠道实体（`is_system=1`），并在 `channel_names` 中为其按受支持语言（en、zh-CN）插入系统名字（`is_system=1`，英文 `Alipay`、中文 `支付宝`），分别设为对应语言的显示名。种子内容与建库显示语言无关（见 `entity-names-i18n`、`builtin-data-english-storage`）。

#### Scenario: 英文环境初始化
- **WHEN** 以 `lang="en"` 初始化数据库
- **THEN** 存在一个 `is_system=1` 的渠道，其英文显示名为 "Alipay"

#### Scenario: 中文环境初始化
- **WHEN** 以 `lang="zh"` 初始化数据库
- **THEN** 同一个内置渠道实体同时拥有中文系统名 "支付宝" 并设为中文显示名，数据库渠道内容与英文建库无结构差异

### Requirement: 内置渠道删除保护
系统 SHALL 禁止删除 `is_system=1` 的渠道，返回错误提示。

#### Scenario: 尝试删除内置渠道
- **WHEN** 用户尝试删除 `is_system=1` 的渠道
- **THEN** 系统返回错误，提示"系统内置渠道不可删除"

#### Scenario: 删除用户创建的渠道
- **WHEN** 用户删除 `is_system=0` 的渠道
- **THEN** 删除成功

### Requirement: 适配器对应内置渠道
所有适配器（通过 `builtin_adapters()` 注册）对应的渠道 SHALL 为系统内置渠道。适配器的 `names()` 返回列表中的第一个名称 SHALL 与内置渠道的英文名称一致。

#### Scenario: Alipay 适配器对应内置渠道
- **WHEN** 查看 AlipayAdapter 的 `names()` 返回列表
- **THEN** 第一个名称为 "alipay"，对应内置渠道（英文 "Alipay"，中文 "支付宝"）
