## MODIFIED Requirements

### Requirement: 业务错误可被结构化以支持本地化
适配器错误（`AdaptError`）、import service 致命错误（`ImportError`）、预算校验错误（`BudgetError`）、攒钱计划校验错误（`SavingPlanError`）SHALL 携带结构化数据而非预格式化的固定语言字符串。CLI SHALL 将每个变体映射到一个翻译 key，并按当前语言环境渲染。CLI 错误信息 SHALL 经 `t!` 生成，或来自某个已经在共享 crate 中完成本地化的类型错误（见 `api-message-i18n`）。

#### Scenario: 适配器行错误
- **WHEN** 支付宝适配器解析某一行失败
- **THEN** 它返回结构化的 `AdaptError::Row { row, detail }`，CLI 使用行号和 detail 数据渲染本地化消息

#### Scenario: 致命导入错误
- **WHEN** `ImportService::import` 因来源不受支持而失败
- **THEN** 它返回 `ImportError::UnsupportedSource { source }`，CLI 渲染本地化消息，例如 "Unsupported source: alipay"

#### Scenario: 预算校验错误本地化
- **WHEN** 预算校验失败（如空名称、限额账户非支出子树）
- **THEN** CLI 按当前语言环境渲染本地化错误，不输出预格式化的中文文案

#### Scenario: 攒钱计划校验错误本地化
- **WHEN** 攒钱计划校验失败（如空名称、账户非资产子树）
- **THEN** CLI 按当前语言环境渲染本地化错误，不输出预格式化的中文文案

### Requirement: 周期类型的用户标签可本地化且与配置机器键解耦
财务周期类型（`FinancePeriod`）的用户可见标签 SHALL 可经 `t!` 按语言环境渲染。该标签 SHALL 与其在配置文件中作为机器往返键的稳定标识相互独立：配置文件的读写 SHALL 使用稳定的不随语言变化的键，CLI 展示 SHALL 使用本地化标签。

#### Scenario: 周期标签随语言渲染
- **WHEN** CLI 在 `lang=zh-CN` 下展示预算的周期列
- **THEN** 周期名以中文本地化标签呈现，而非英文标识

#### Scenario: 配置文件键不随语言变化
- **WHEN** 配置被导出再导入
- **THEN** 周期字段以稳定的语言无关键持久化与往返，与界面语言无关
