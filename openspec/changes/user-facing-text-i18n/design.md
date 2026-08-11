# Design: user-facing-text-i18n

## Context

见 `proposal.md` 的 Why。当前 i18n 水准参差:`accounting/src/error.rs`(AccountingError)是标杆——`Display` 全用 `t!`、且 `test_error_display_en/zh` 双语测试锁定 locale 行为;`accounting-service` 的 ImportError 在 CLI 面已合规(`accounting-cli/src/cmd/import.rs:format_import_error` 按变体映射 `t!`),但 API 面漏;BudgetError/SavingPlanError 全程零 i18n 且经 `AccountingError::InvalidTransaction(String)` 穿透壳层泄漏。finance_period.rs L83 的 Display 被迫双职:既当 CLI 表格标签、又当配置文件往返键(`config/service.rs:213` 写、`:396` 读)。

探查确认:5 个未 i18n 的 Display 中 4 个直达用户(CLI/API/Web 三面),且 API handler 用 `msg.contains("预算表不存在")` 等中文子串匹配判定 HTTP 状态码——i18n 与控制流耦合,是其比"漏翻"更深的债 signals。

## Goals / Non-Goals

**Goals:**
- 三面(CLI/API/Web)用户可见文本与 `error.rs` 同等:可本地化、有双语测试、locale 行为可证。
- 错误的本地化与状态码/客户端分支逻辑解耦:不再以本地化字面量作 machine contract。
- FinancePeriod 的"用户标签"与"配置文件机器键"职责分离,不破坏现有配置文件契约。

**Non-Goals:**
- 不动 `auth-admin` 运维二进制(无 i18n 基础设施,运维工具非应用面)。
- 不动 `#[derive(Tabled)]` 字段名表头(机器契约标识符,单用户工具可接受 snake_case)。
- 不动 `account_type` 等 machine-contract 标识符字段(前端按枚举值分组,误包 `t!` 会断契约)。
- 不动日志/tracing/Debug/SQL 字面量(非用户可见运行面)。
- 不引入新 locale 体系外的库(继续 rust-i18n + vue-i18n)。

## Decisions

### D1: 错误边界本地化,而非每个枚举自建 i18n Display

**决策**:BudgetError/SavingPlanError/ImportError/AdaptError 的 `Display` 停止产出预格式化文案;在 service→API 边界由 handler 按**变体**映射到 `t!(..., locale=lang)`(参照已有 `format_import_error` 范式)。`RowErrorDetail::Other` 由携带 `String`(预字符串化的 AccountingError)改为携带结构化 `AccountingError` 枚举值,handler 渲染时 `t!(..., locale=lang)`。

**理由**:Display 是单 locale 的(读进程全局 locale),无法随请求 locale 渲染;置 i18n 于边界(handler 持有请求 `Lang`)才正确。`error.rs` 的 AccountingError 之所以 Display 内置 `t!` 能工作,是因 API 进程启动时 `set_locale(default_lang)` 单 locale——它牺牲了 per-request locale 换取 CLI/单 locale 场景的简洁。新错误类型不再沿袭此 hack,统一边界本地化。

**被否决备选**:
- (a) 给每个错误 Display 内置 `t!`(沿用 AccountingError 模式):否决——继承单 locale hack,API per-request locale 无法生效,且 `Other{message:String}` 穿透仍漏。
- (b) 给错误加 `code()` 方法返回稳定码,Display 改返回 `code`:否决——Display 语义本就该是人类可读文案,返回码破坏 trait 契约;码与文案应分处不同 API(`code()` 方法 + 边界 `t!`)。

### D2: 状态码/分支判定按错误变体,不按本地化子串

**决策**:handler `map_error` 由 `msg.contains("预算表不存在")` 改为 `match` 错误变体(`BudgetError::BudgetNotFound` → 404 / 其余 → 400);`account_rename` 的 `contains("system root account")` 改为按 repo 层类型化错误变体;认证 `MSG_BAD_TOTP` 改为响应带结构化 `code:"bad_totp"` 字段,前端 `LoginView` 按 `code` 分支。

**理由**:子串匹配把本地化文案既当展示又当 machine契约——一旦文案本地化,匹配即失效(英文 session 下 `contains("预算表不存在")` 恒假→404 误判 400)。这是 `contains` 耦合存在即证明泄漏在跑通的反向证据。

**被否决备选**:保留 `contains` 但匹配英文 key 而非中文:否决——仍是"文案即契约",换语言后又断;且 key 同时要人类可读又要稳定,双职又来。

### D3: FinancePeriod 双职拆分——稳定 machine key + 边界本地化标签

**决策**:新增 `FinancePeriod::config_key()` 返回稳定 snake-case(`"daily"`/`"weekly_sun"`/`"weekly_mon"`/`"monthly"`/`"yearly"`),作为配置文件往返键;`config/service.rs:213` 写 `config_key()` 而非 `Display`;`parse_budget_period(:396)` 接受新 key 并向后兼容旧 PascalCase 读取(过渡期)。CLI 标签经 `t!("finance_period.{}", ...)` 边界渲染,`Display` 保留为 PascalCase 作 debug/回退用途但不再当用户标签的唯一来源。

**理由**:Display 单职化是根本解;复用已有 `dto::to_period_string` 的 snake-case 维度避免再造一套命名。向后兼容读避免破坏存量配置文件。

**被否决备选**:
- (a) 保留 Display 双职,只把 CLI 表格标签改 `t!`:否决——配置文件往返键仍绑定在 Display 上,日后改 CLI 标签会断配置。
- (b) 配置文件改存整数(`as_i64`):否决——更具侵入,且 YAML 可读性下降;snake-case 兼容期更自然。

### D4: raw sqlite 错误统一 generic 本地化,详情仅日志

**决策**:API 面 `.map_err(|e| e.to_string())?` 模式统一改为输出 `t!("database_error", locale=lang)` 通用文案;原始 `e` 详情经 `tracing::error!` 记服务端日志,不进响应体。

**理由**:sqlite 错误文本随系统 locale 变化、不可控、对用户无意义;通用文案 + 服务端日志是 REST API 错误处理的常规形态。

**被否决备选**:逐个 sqlite 错误子类型 i18n:否决——面太大、收益低,且 sqlite 文案非项目可维护资源。

## Risks / Trade-offs

- **BREAKING `/api/auth/*` 增加 `code` 字段**:早期客户端按 `error` 字面量分支的逻辑需迁移。→ Mitigation: `error` 字段保留(本地化 message),`code` 为新增;客户端先迁 `code` 分支再服务端可删字面量匹配。过渡期双信号并存。
- **BREAKING 配置文件 `period` PascalCase→snake-case**:存量配置文件可能含 `"Monthly"`。→ Mitigation: `parse_budget_period` 读侧双格式兼容;写侧只发新格式。无强制迁移。
- **ImportError API 面 BREAKING**:`channel.rs import` 错误体由单一 `import_failed` 改按变体 `t!`。→ Mitigation: 错误仍为 HTTP 400 + `{error, code?}`,仅文案与 code 维度变;前端 `apiErrorMessage` 仅取 `error` 不受影响。
- **边界本地化使 Display 失去人读性**:BudgetError 等 Display 若不再产出文案,Debug/日志里只剩码。→ Mitigation: Display 保留变体名/关键参数的可读形式(非本地化),不依赖它做用户展示;D1 已申明 Display 不携带 `t!`。
- **测试覆盖面扩大**:本地化行为要双语测试。→ Mitigation: 沿用 `error.rs` 的 `LOCALE_LOCK` 串行化范式,集中补测试。

## Migration Plan

1. 先落 locale key(en/zh 双语),后端类型化错误与边界映射(不改响应契约字段,仅文案本地化+`code` 新增但暂保留 `contains` 兼容)。
2. 前端 `LoginView` 迁 `code` 分支(与 step 1 的 `code` 并存期)。
3. 去除 handler `contains` 与前端 `SERVER_BAD_CODE`(`code` 稳定后)。
4. FinancePeriod config_key 切换:写侧发 snake-case,读侧双格式兼容。
5. raw sqlite generic 化批量替换(机械改动,逐 handler 验证不丢语义)。
6. 双语测试补齐,`cargo test` + 手测 en/zh session 验证。

**Rollback**: 各步独立可回退;D2 去耦合与 D1 边界本地化是同一 PR 内的原子改动(去 `contains` 必须与变体映射同现),不拆。

## Open Questions

- `accounting-api` 是否已有 per-request `Lang` extractor 覆盖全部错误返回路径?探查显示 transaction.rs 的 `Result<_,String>` 错误路径未一致注入 `lang`。→ 实现阶段确认 `Lang` 透传范围;若缺,task 中显式补。不改变 specs(已 SHALL per-request locale)。
