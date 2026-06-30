# cli-message-i18n

## ADDED Requirements

### Requirement: 所有 CLI 用户可见输出均可本地化
所有由 `accounting-cli` 命令因用户操作而打印的字符串 SHALL 可通过 `accounting-cli/locales` 进行翻译。CLI SHALL 使用 `rust-i18n` 的 `t!` 宏按当前语言环境组装最终文本。

#### Scenario: 成功操作反馈
- **WHEN** CLI 命令完成一项用户操作（例如创建交易或导入文件）
- **THEN** 成功或摘要信息通过 `t!` 生成，使用 `accounting-cli/locales` 中定义的 key

#### Scenario: 错误反馈
- **WHEN** CLI 命令遇到错误
- **THEN** 展示给用户的错误信息通过 `t!` 生成，或来自某个已经在共享 crate 中完成本地化的类型错误

### Requirement: Import 错误可被结构化以支持本地化
适配器错误（`AdaptError`）和 import service 致命错误（`ImportError`）SHALL 携带结构化数据而非预格式化的中文字符串。CLI SHALL 将每个变体映射到一个翻译 key，并按当前语言环境渲染。

#### Scenario: 适配器行错误
- **WHEN** 支付宝适配器解析某一行失败
- **THEN** 它返回结构化的 `AdaptError::Row { row, detail }`，CLI 使用行号和 detail 数据渲染本地化消息

#### Scenario: 致命导入错误
- **WHEN** `ImportService::import` 因来源不受支持而失败
- **THEN** 它返回 `ImportError::UnsupportedSource { source }`，CLI 渲染本地化消息，例如 "Unsupported source: alipay"

### Requirement: 英文兜底
如果当前语言环境缺少某个翻译 key，系统 SHALL 回退到 `accounting-cli/locales/en.yaml` 中定义的英文翻译。

#### Scenario: 缺少语言项
- **WHEN** 当前语言环境不包含请求的 key
- **THEN** 显示 `accounting-cli/locales/en.yaml` 中的英文文本
