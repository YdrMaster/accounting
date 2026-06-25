## MODIFIED Requirements

### Requirement: 适配器与渠道自动关联
当用户指定 `--source` 选择适配器类型时，系统 SHALL 同时确定渠道（Channel）并设置到 ImportContext 中。适配器对应的渠道 SHALL 为系统内置渠道（`is_system=1`），在数据库初始化时自动创建。

#### Scenario: 选择支付宝适配器时自动设置渠道
- **WHEN** 用户指定 `--source alipay` 或 `--source 支付宝`
- **THEN** 系统查找名为 "支付宝"（中文环境）或 "Alipay"（英文环境）的内置 Channel，将其 channel_id 设入 ImportContext，导入交易的 channel_path 为 [支付宝]

#### Scenario: 渠道不存在时报错
- **WHEN** 指定的来源对应的内置渠道在系统中不存在（数据库未正确初始化）
- **THEN** 系统返回错误，提示"渠道不存在，请检查数据库初始化"
