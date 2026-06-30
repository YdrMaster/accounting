## 为什么

记账 CLI 目前在大量用户可见信息里硬编码了中文，尤其是 import 链路（`accounting-service/src/import/alipay.rs`、`import_service.rs` 以及 `accounting-cli/src/cmd/import.rs`）。这导致英文环境下无法使用，也与项目已有的 i18n 基础设施（`rust-i18n`、各 crate 的 locales）相冲突。需要一次性补齐，使所有 CLI 可见文本都能在 CLI 层完成翻译。

## 改动内容

- 将 import 适配器错误（`AdaptError`）改为结构化枚举，不再携带已经格式化好的中文字符串。
- 在 import service 中新增 `ImportError` 枚举，用于表达致命的导入失败。
- 在 `accounting-cli/src/cmd/import.rs` 中使用 `accounting-cli/locales` 翻译所有 import 相关文案。
- 扫清其他 CLI 命令（`budget`、`report`、`mapping`、`beancount`、`resolver`、`tx`）中剩余的中文硬编码字符串，全部移到 `accounting-cli/locales`。
- 保持 `accounting` crate 中已有的 `AccountingError` Display 翻译不变（它已被 API 和 CLI 共享）。
- 更新依赖中文字串断言的测试，改为断言枚举变体或显式设置 locale。

## 能力

### 新增能力

- `cli-message-i18n`：所有 CLI 用户可见输出（包括 import 摘要、错误详情、命令反馈）都能在 CLI 层完成本地化和翻译。

### 修改的能力

- `bill-import`：适配器和 import service 的错误以结构化数据返回，由 CLI 负责本地化。解析行为和支持格式不变。

## 影响范围

- `accounting-cli/src/cmd/*.rs` —— 所有用户可见字符串改为 `t!` 调用。
- `accounting-cli/locales/{en,zh-CN}.yaml` —— 新增 CLI 输出和 import 错误的翻译 key。
- `accounting-service/src/import/mod.rs` —— `AdaptError` 改为结构化枚举。
- `accounting-service/src/import/alipay.rs` —— 适配器产生结构化错误。
- `accounting-service/src/import_service.rs` —— 新增 `ImportError` 枚举；`ImportService::import` 返回该类型。
- 涉及 `accounting-cli` 和 `accounting-service` 中依赖中文展示文本的测试。
