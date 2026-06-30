## 1. 结构化 import 适配器错误

- [x] 1.1 重构 `accounting-service/src/import/mod.rs` 中的 `AdaptError`，从基于字符串的变体改为结构化枚举（`Encoding { source }`、`Row { row, detail }`），并新增 `RowErrorDetail` 覆盖 `MissingColumn`、`AmountParse`、`DateParse`、`ClosedTransaction`。
- [x] 1.2 更新 `accounting-service/src/import/alipay.rs`，在编码失败、列缺失、金额/日期解析失败、交易关闭等场景下构造结构化 `AdaptError` 变体，不再生成预格式化的中文字符串。
- [x] 1.3 更新 `accounting-service/src/import/mod.rs` 和 `accounting-service/src/import/alipay.rs` 中的单元测试，使其断言新的结构化变体而非中文展示文本。

## 2. 结构化 import service 致命错误

- [x] 2.1 在 `accounting-service/src/import_service.rs` 中引入 `ImportError` 枚举，覆盖 `UnsupportedSource { source }`、`ChannelNotFound { source }`、`CnyCommodityNotFound`、`ImportRootNotFound`、`Parse { source }`、`Database { source }`。
- [x] 2.2 将 `ImportService::import` 的返回类型改为 `Result<ImportResult, ImportError>`，并把现有的 `AccountingError` 构造映射为新的 `ImportError` 变体。
- [x] 2.3 为 `ImportResult` 添加 `pending_tag_name: Option<String>`，并根据 `resolve_pending_tag_id` 的实际结果填充，使 CLI 提示能使用数据库当前语言的标签名。
- [x] 2.4 更新 `import_service.rs` 中的测试，使其断言 `ImportError` 变体而非中文错误子串。

## 3. 本地化 CLI import 命令

- [x] 3.1 在 `accounting-cli/src/cmd/import.rs` 中，将每个 `ImportError` 变体通过 `t!` 映射为翻译后的字符串，并包装到合适的 `AccountingError` 载体中。
- [x] 3.2 在 `accounting-cli/src/cmd/import.rs` 中新增辅助函数，将结构化的 `AdaptError` 每个变体映射为 `t!` 调用，并在打印跳过的行时使用它。
- [x] 3.3 替换 `accounting-cli/src/cmd/import.rs` 中所有硬编码中文（摘要、待处理标签提示、交易 ID 列表），改为使用 `t!` 调用并结合 `ImportResult` 字段格式化。

## 4. 扫清 CLI 剩余中文

- [x] 4.1 本地化 `accounting-cli/src/cmd/budget.rs` 中的成功信息和错误字符串。
- [x] 4.2 本地化 `accounting-cli/src/cmd/report.rs` 中的报表标题和信息。
- [x] 4.3 本地化 `accounting-cli/src/cmd/mapping.rs` 中的映射命令输出。
- [x] 4.4 本地化 `accounting-cli/src/cmd/beancount.rs` 中的输出和错误。
- [x] 4.5 本地化 `accounting-cli/src/cmd/resolver.rs` 中的错误字符串。
- [x] 4.6 本地化 `accounting-cli/src/cmd/tx.rs` 中尚未使用 `t!` 的剩余中文。

## 5. 补充翻译 key

- [x] 5.1 将任务 3 和任务 4 所需的所有英文 key 添加到 `accounting-cli/locales/en.yaml`。
- [x] 5.2 将对应的中文 key 添加到 `accounting-cli/locales/zh-CN.yaml`。

## 6. 验证与清理

- [x] 6.1 运行 `cargo fmt --all`。
- [x] 6.2 运行 `cargo check --workspace` 并修复编译错误。
- [x] 6.3 运行 `cargo test --workspace` 并更新仍依赖硬编码中文展示文本的测试。
- [x] 6.4 运行 `cargo clippy --workspace -- -D warnings` 并解决所有警告。
