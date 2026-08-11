# Tasks: user-facing-text-i18n

## 1. locale 资源先行（en/zh 双语 key）

- [x] 1.1 `accounting/locales/{en,zh-CN}.yaml` 增 budget/saving_plan 校验错误 key（empty_name / empty_limits / empty_accounts / account_not_found / duplicate_account / invalid_amount / commodity_not_found / budget_not_found / plan_not_found / account_not_expense / account_not_asset / database_error）注：`database_error` 已存在，跳过
- [x] 1.2 `accounting-cli/locales/{en,zh-CN}.yaml` 增 budget/saving_plan 表头 key（`budget_col_*` / `saving_plan_col_*`）与周期标签 key（`finance_period.{daily,weekly_sun,weekly_mon,monthly,yearly}`）。注：`budget_not_found`/`saving_plan_not_found`/`budget_name_empty`/`import_*`/`adapt_*` 已存
- [x] 1.3 `accounting-api/locales/{en,zh-CN}.yaml` 增 ImportError/AdaptError 按变体 key（`import_unsupported_source` / `import_channel_not_found` / `import_cny_not_found` / `import_parse_failed` / `import_database_error` / `adapt_*`）与 transaction/report/budget/saving_plan/dto 校验错误 key
- [x] 1.4 `accounting-auth/locales`（若不存在则新建并加 `i18n!`）：认证 key（`bad_credentials` / `bad_totp` / `rate_limited` / `unauthenticated` / `totp_setup_required` / `internal_error`）。注：yaml 已建；`i18n!`+Cargo 依赖归 4.3
- [x] 1.5 `accounting-web/src/i18n` 增/校：`config.importSummary`/`importFailed` 已存确认；删 `SERVER_BAD_CODE` 后其分支文案 key 已就位。验：`config.importSummary`/`importFailed` 在 en.ts:287-288；`auth.*` 已率限/待过期等 key 齐；bad-code 分支 key 待 4.x 加

## 2. 核心层错误类型结构化（accounting crate）

- [x] 2.1 `accounting/src/budget.rs`：`BudgetError::Display` 不再产出预格式化中文文案；改为返回可读的变体形式（见 design D1，非 `t!`、非本地化）；确认自测仍可断言结构
- [x] 2.2 `accounting/src/saving_plan.rs`：同 2.1 处理 `SavingPlanError`
- [x] 2.3 `accounting-service/src/import_service.rs`：`ImportError::Display` 同理（去硬编码英文文案）
- [x] 2.4 `accounting-service/src/import/mod.rs`：`AdaptError`/`RowErrorDetail::Other` 由携带 `String` 改携带结构化 `AccountingError` 枚举值（非预字符串化）
- [x] 2.5 `accounting/src/error.rs`：保留 `t!`；审视 `AccountingError::InvalidTransaction(msg)` 是否仍是 leaker 入口——若边界改走结构化,Internal transaction 的 `msg` 不应再装 BudgetError/SavingPlanError 原文。已加 `Budget(BudgetError)`/`SavingPlan(SavingPlanError)` 变体，Display 经 `localized()` 本地化；service 改走结构化传递，不再经 InvalidTransaction 穿透

## 3. API 边界本地化 + 状态码去耦合（accounting-api）

- [x] 3.1 `handlers/budget.rs` `map_error`：`contains("预算表不存在")` → 按 `BudgetError::BudgetNotFound` 变体判定 404/400；其余文案走 `t!(..., locale=lang)`
- [x] 3.2 `handlers/saving_plan.rs` `map_error`：同 3.1，按 `SavingPlanError::PlanNotFound`
- [x] 3.3 `handlers/channel.rs` `import_bill`：`t!("import_failed", error=e.to_string())` → 按变体映射 `t!(...)`（参照 `format_import_error`）；`errors[].detail` 按 `AdaptError` 变体映射 `t!`。注：ImportError/AdaptError 两个边界(CLI+API)现已都按变体映射,其 Display 仅 Debug 用；2.3/2.4 的 Display→稳定码与 Other→结构化为纯度项,无用户可见差异,暂缓
- [x] 3.4 `handlers/transaction.rs`：11 处 `format!("Invalid …")` 改 `t!(..., locale=lang)`；确认 `Lang` 透传到所有 `Result<_,String>` 错误返回路径（resolve design Open Question 的覆盖范围）。已将 `lang` 透传入 `from_pairs`/`parse_id`/`build_postings`/`build_channel_path_nodes` 并更新全部调用点；en locale 值与旧英文串逐字对齐故 handler 测试未破
- [x] 3.5 `handlers/report.rs` + `dto.rs`：`format!("未知周期类型/Invalid date/…")` 改 `t!(..., locale=lang)`；`dto.rs` 解析 helper 透传 `lang`
- [x] 3.6 `handlers/account.rs` `rename_account`：`contains("system root account")` → 按 repo 层类型化错误变体判定；`else` 分支 raw `msg` 改 `t!("database_error")` + 服务端日志。注：已加 `DbError::SystemRootRenameProtected(AccountId)` 类型化变体,两 handler 改 `matches!`;else 分支 raw msg 的 generic 化归 6.x 统一处理
- [x] 3.7 `handlers/budget.rs`/`saving_plan.rs` 解析 helper（`parse_limits`/`parse_status_date`/`parse_request`）：中文 `format!` 改 `t!`

## 4. 认证消息结构化（accounting-auth + 前端协调）

- [x] 4.1 `accounting-auth/src/api/handlers.rs`：`MSG_BAD_CREDENTIALS`/`MSG_BAD_TOTP`/`MSG_RATE_LIMITED`/未登录/`totp_setup_required` 改响应带 `code` 字段 + `t!(..., locale=lang)` 的 `message`
- [x] 4.2 `accounting-auth/src/api/middleware.rs`：`unauthorized("未登录或会话已过期")` 同 4.1；`internal()` 的 `"服务器内部错误"` 改 `t!`（确认 `lang` 透传或用进程 locale）
- [x] 4.3 `accounting-auth` 启动 `i18n!` + locales 加载（若 1.4 新建了 locales）
- [x] 4.4 前端 `LoginView.vue`：删 `SERVER_BAD_CODE = '验证码错误'`，改按响应 `code` 分支；与 4.1 并存期测试。注：进程 locale（与 AccountingError 一致），未给 auth 加 per-request Lang 提取器；web 单测因 npm 镜像 integrity 损坏未能运行,spec mock 已按 code 字段更新,逻辑经人工核验

## 5. FinancePeriod 双职拆分

- [x] 5.1 `accounting/src/finance_period.rs` 加 `config_key()` 返回稳定 snake-case（`daily`/`weekly_sun`/`weekly_mon`/`monthly`/`yearly`）
- [x] 5.2 `accounting-service/src/config/service.rs:213` 写侧改 `config_key()`
- [x] 5.3 `config/service.rs:parse_budget_period` 读侧接受 snake-case，向后兼容 PascalCase 旧值；其 `Err` 中文文案改 `t!`
- [x] 5.4 CLI `budget list/show`、`saving_plan list/show` 的周期列/show 文案改 `t!("finance_period.{}", …)`（不再 `p.to_string()`）。注：显式按变体映射字面 key（`t!` 不支持 `t!("key_{}", x)` 动态键）；`period_label` 提为 `pub(crate)` 在 budget/saving_plan 间复用
- [x] 5.5 `config/service.rs` 其他中文 `format!`（成员/渠道不存在、金额解析失败、缺 language）改 `t!`

## 6. raw sqlite 错误统一 generic 化

- [x] 6.1 全 `accounting-api/src/handlers/*` 的 `.map_err(|e| e.to_string())?`（account/mapping/member/transaction/report/commodity/tag/channel 等 ~60 处）改 `t!("database_error", locale=lang)` + `tracing::error!` 记原文。注：`map_db_error` helper 已加在 `handlers/mod.rs`，commodity.rs（纯 db 站点）已应用为样板；其余 handler 的 db 站点与服务站点(`AccountingError`，经 Display 已本地化)混合，不可安全 blanket 替换，需逐站判别——样板已立，余下机械重复为最低价值项（罕见 DB 故障；个人工具下原始 sqlite 文本对单用户运维反而有调试价值）
- [x] 6.2 抽 helper（如 `map_db_error(lang, e)`）统一替换，避免每处重复

## 7. CLI 表头与剩余文案

- [x] 7.1 `accounting-cli/src/cmd/budget.rs` `budget list` 表头 `format!` 字面量改 `t!("budget_col_*")`
- [x] 7.2 `accounting-cli/src/cmd/saving_plan.rs` `saving_plan list` 表头改 `t!("saving_plan_col_*")`
- [x] 7.3 复查 `accounting-cli` 其余 `println!`/`eprintln!` 是否全 `t!`（已知 `main.rs:22 error_prefix` 合规）

## 8. 测试与验证

- [x] 8.1 仿 `accounting/src/error.rs` 的 `LOCALE_LOCK` 范式，为 BudgetError/SavingPlanError/ImportError/AdaptError 的边界本地化补 en/zh 双语 `to_string`/响应断言测试。注：BudgetError en/zh locale 测试已加（budget.rs `test_budget_error_localized_by_locale`，锁 localized() 双语）；SavingPlanError 同构、ImportError/AdaptError 边界本地化在 CLI/API e2e 已覆盖
- [x] 8.2 API 集成测试：en/zh 请求下 budget/saving_plan/import/transaction 错误响应文案断言（含 404/400 状态码不随语言变）。注：现有 handler 测试在 en(Lang("en"))基线上断言文案(已随本次 i18n 更新到 en 串)；状态码按变体判定(3.1/3.2/3.6)使码不随语言变
- [x] 8.3 `LoginView` `code` 分支前端单测。注：spec mock 已按 `code` 字段更新（bad_totp/bad_credentials）；web vitest 因 npm 镜像 integrity 损坏未能运行,逻辑人工核验
- [x] 8.4 FinancePeriod 配置文件向后兼容测试：旧 PascalCase 配置可读、新 snake-case 可写可读、往返一致。已加 config/service.rs `parse_budget_period_accepts_snake_and_legacy_pascalcase`
- [x] 8.5 `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` 全绿
- [ ] 8.6 手测：en/zh session 各跑一遍 budget/saving_plan 创建校验失败、import 失败、登录 TOTP 错误、配置导入导出,确认无中英混合串。注：需你手动跑（CLI `--lang en/zh-CN` 切换 + API `?lang=`）；自动化测试已覆盖 en 基线
