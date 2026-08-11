# Proposal: user-facing-text-i18n

## Why

仓库存在系统性的多语言泄漏：多处用户可见文本未走 `t!`(Rust)/`t(`(前端)，导致英文 session 下出现中文、中文 session 下出现英文，甚至英壳中芯的混合串。已确认泄漏覆盖全部三面（CLI 输出、API JSON 响应体、Web 渲染文本），且其中几处的 i18n 与状态码/客户端分支判定逻辑耦合（`msg.contains("中文字面")`），是比单纯漏翻更深的工程债。现有规格 `cli-message-i18n` 已为 CLI 面立了标杆，但 API/Web 面无规格、且 CLI 面的错误类型覆盖不全（仅 `AdaptError`/`ImportError`，不含 `BudgetError`/`SavingPlanError`/`FinancePeriod`）。本次把用户可见文本的多语言一致性提升到与 `cli-message-i18n` 同等的可测、可证水准。

## What Changes

- **结构化错误 + 边界本地化**：`BudgetError`、`SavingPlanError`、`ImportError`、`AdaptError`/`RowErrorDetail` 的 `Display` 不再产出预格式化文案；改为在 service→API 边界按变体映射到 `t!(...)`(参照 `accounting-cli/src/cmd/import.rs:format_import_error` 已有范式)。`RowErrorDetail::Other` 携带结构化数据而非预字符串化的 `AccountingError.to_string()`。
- ****BREAKING** —— 状态码判定去耦合**：API handler `budget.rs`/`saving_plan.rs` 的 `map_error` 由 `msg.contains("预算表不存在")` 改为按错误变体(`BudgetError::BudgetNotFound`/`SavingPlanError::PlanNotFound`)判定 404 vs 400。`account_rename` 的 `contains("system root account")` 同样改为按 repo 层类型化错误判定。前端随之去掉 `SERVER_BAD_CODE = '验证码错误'` 字面匹配。
- **FinancePeriod 双职拆分**：`FinancePeriod::Display` 当前同时是 CLI 用户标签与配置文件往返键（`config/service.rs:213` 写 / `:396` 读）——拆分为稳定 machine key（snake-case，复用 `dto::to_period_string` 或新增 `config_key()`）+ `t!` 包装的 CLI 标签；`parse_budget_period` 接受新 key 并向后兼容旧 PascalCase。
- **API handler 散落 `format!` 文案统一**：`transaction.rs`(11 处英文)、`report.rs`/`budget.rs`/`saving_plan.rs`/`dto.rs`(中英混杂)的硬编码 `format!` 改为 `t!(..., locale=lang)`；`Lang` 透传到错误返回路径。
- **认证消息结构化（**BREAKING**）**：`accounting-auth` 的 `MSG_BAD_CREDENTIALS`/`MSG_BAD_TOTP`/`MSG_RATE_LIMITED` 等改为 JSON 返回结构化 `code` 字段 + `t!` 本地化 `message`；客户端按 `code` 分支而非本地化串。`internal()` 的 `"服务器内部错误"` 同样走 `t!`。
- **raw sqlite 错误统一 generic 化**：API 面大批 `.map_err(|e| e.to_string())?` 改为输出 `t!("database_error")` 通用本地化文案，原始 detail 仅记服务端日志。
- **CLI table 表头本地化**：`budget list`/`saving_plan list` 手写表头改 `t!`；`#[derive(Tabled)]` 字段名表头作为机器契约标识符保留，不入本次范围。
- **Web 错误管道**：前端模板本身已干净；`{{ store.error }}`/`alertDialog(t('…saveFailed', {message}))` 的泄漏由服务端修复根治，前端仅去 `LoginView` 的 `SERVER_BAD_CODE` 耦合。
- **范围排除**（明确标注）：`accounting-cli/docs/`、`auth-admin` 二进制（运维工具）、日志/Debug/tracing、SQL 字面量、`#[derive(Tabled)]` 字段名表头、`account_type` 等 machine-contract 标识符不入本次。

## Capabilities

### New Capabilities
- `api-message-i18n`: API JSON 响应体中所有用户可见文本（含错误 message、`errors[].detail`、human-readable 字段）SHALL 可按请求语言本地化；错误 SHALL 结构化（code + 本地化 message），状态码判定 SHALL 不依赖本地化字面量。
- `web-text-i18n`: accounting-web 渲染到用户的文本 SHALL 经 vue-i18n 本地化；SHALL NOT 以本地化串作为跨边界 machine contract。前端错误展示依赖服务端返回的本地化 message 与结构化 code。

### Modified Capabilities
- `cli-message-i18n`: 错误类型本地化覆盖扩展——`BudgetError`、`SavingPlanError` SHALL 经 `t!` 本地化（当前规格仅列 `AdaptError`/`ImportError`）；`FinancePeriod` 的用户可见标签 SHALL 可本地化且与其作为配置文件机器键的稳定标识解耦。

## Impact

- 受影响代码：`accounting/src/{budget,saving_plan,finance_period,error}.rs`、`accounting-service/src/{import_service,import/mod,report/budget,report/saving_plan,config/service}.rs`、`accounting-api/src/handlers/{budget,saving_plan,transaction,report,account,channel}.rs` 与 `dto.rs`、`accounting-auth/src/api/{handlers,middleware}.rs`、`accounting-cli/src/cmd/{budget,saving_plan,account,import}.rs`、`accounting-web/src/views/LoginView.vue`。
- 受影响规格：新建 `api-message-i18n`、`web-text-i18n`；扩展 `cli-message-i18n`。
- locale 资源：`accounting/locales`、`accounting-api/locales`、`accounting-cli/locales`、`accounting-web/src/i18n` 需新增对应 key（budget/saving_plan/import/auth/period 等）。
- **BREAKING**：`/api/auth/*` 响应在错误分支增加 `code` 字段（原仅 `error`）；早期客户端按 `error` 字面量分支的逻辑需迁移到 `code`。`channel.rs import` 错误体由单一 `import_failed` 模板改为按变体的 `t!`。配置文件 budget `period` 字段由 PascalCase 迁移到 snake-case（保留旧格式向后兼容读取）。
- 测试：参照 `accounting/src/error.rs` 的 `test_error_display_en/zh` 范式，为本地化的 `Display`/边界映射补 locale 双语测试。
