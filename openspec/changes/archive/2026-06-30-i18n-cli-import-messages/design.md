## 上下文

项目已经使用 `rust-i18n` 并按 crate 维护 YAML 语言文件。`accounting/src/error.rs` 对核心 `AccountingError` 变体进行了翻译，许多 CLI 命令也已经使用 `t!` 输出成功信息。然而，import 链路（`accounting-service/src/import/alipay.rs`、`import/mod.rs`、`import_service.rs`）以及若干 CLI 命令（`budget`、`report`、`mapping`、`beancount`、`resolver`）仍然在 `format!` / `println!` 中直接嵌入中文。用户希望一次性、干净地解决这个问题，让 CLI 拥有最终的用户可见文本。

## 目标 / 非目标

**目标：**
- 所有由 CLI 命令产生的用户可见字符串都可以通过 `accounting-cli/locales` 翻译。
- Import 适配器和 service 错误携带结构化数据，CLI 可以据此进行本地化。
- 保持 `accounting` crate 中现有 `AccountingError` 翻译不变。
- 更新测试，使其不再断言特定的中文字符串。

**非目标：**
- 改变支持的导入格式或解析行为。
- 新增 `zh-CN` 和 `en` 之外的其他语言。
- 翻译内部调试日志、注释或测试断言信息。
- 为 API/服务端实现按请求切换语言。

## 决策

1. **服务层返回结构化错误，CLI 负责翻译。**
   - `AdaptError` 改为结构化枚举（`Encoding`、`Row { row, detail }`）。
   - `ImportError` 枚举覆盖致命导入错误（`UnsupportedSource`、`ChannelNotFound`、`CnyCommodityNotFound`、`ImportRootNotFound`、`Parse`、`Database`）。
   - `accounting-cli/src/cmd/import.rs` 对每个变体匹配并调用 `t!`。
   - 理由：保持 service crate 与语言无关，符合用户“到 CLI 层翻译”的要求。

2. **保留 `AccountingError` 在 `accounting` crate 的翻译。**
   - `AccountingError` 是 API 和 CLI 共享的域错误。把它搬到 CLI 会让 API 依赖 CLI 的 locale 文件或导致 key 重复。
   - 理由：最小化影响范围；核心错误语义应靠近领域层。

3. **使用现有的 `rust-i18n` `t!` 宏模式。**
   - 不引入新依赖；与 `tx.rs`、`account.rs` 等现有代码保持一致。
   - 理由：最少意外，复用现有工具和语言文件。

4. **`ImportResult` 携带实际的待处理标签名。**
   - `ImportService::import` 返回 `pending_tag_name: Option<String>`，这样 CLI 提示 `tx list --tag <name>` 可以使用实际标签名（根据 seed 语言可能是“待处理”或 “pending”）。
   - 理由：避免在 CLI 中硬编码中文标签名。

5. **测试断言枚举变体或显式设置 locale。**
   - `ImportError` / `AdaptError` 的 service 测试检查变体。
   - 必须验证措辞的展示测试显式调用 `rust_i18n::set_locale`。
   - 理由：展示字符串是表现层问题，不是业务逻辑。

## 风险 / 权衡

- **服务错误变体与 CLI 翻译 key 的耦合更紧。** 新增 `AdaptError` / `ImportError` 变体时需要同时更新 CLI 的映射。  
  → 缓解：让变体与用户可见的失败模式保持一致，并在本设计中记录。

- **`rust_i18n` 的 locale 是全局状态。** CLI 是单进程，因此没有问题；API 无法按请求切换语言。  
  → 缓解：本次变更不在范围内。

- **涉及文件多、改动机械量大。** 容易漏掉某个字符串或引入拼写错误。  
  → 缓解：按命令拆分为独立任务；最后运行完整 workspace 测试和 clippy。

- **现有 `bill-import` 规约场景引用了中文错误文本。** delta spec 必须更新这些场景，对应测试也可能需要调整。  
  → 缓解：将规约场景改为期望本地化/结构化行为，并在同一变更中更新测试。

## 迁移计划

无需运行时迁移。已有数据库不受影响。变更完成后，以英文运行 CLI（`--lang en` 或 `LANG=en_US.UTF-8`）时，所有涉及的命令都会输出英文。

## 待解决问题

无 —— 范围和方法已由用户确认。
