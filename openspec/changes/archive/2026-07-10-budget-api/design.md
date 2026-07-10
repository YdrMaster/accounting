## Context

预算系统已有完整的分层实现：
- **Domain** (`accounting/src/budget.rs`): Budget/BudgetLimit 结构体、validate_budget 验证函数、BudgetError 错误类型
- **DB** (`accounting-sql/src/repo/budget.rs`): budget_create/get/list/update/delete/get_limits 等 CRUD 函数
- **Service** (`accounting-service/src/report/budget.rs`): BudgetService，封装验证+持久化+执行查询
- **CLI** (`accounting-cli/src/cmd/budget.rs`): 5 个子命令已映射

现需在 `accounting-api` crate 中新增 HTTP handler 层，复用已有 BudgetService。现有 handler 模式（account.rs、report.rs 等）已建立清晰的模式可参照。

## Goals / Non-Goals

**Goals:**
- 将 BudgetService 的 6 个核心方法暴露为 REST API
- 遵循现有 handler 的代码模式（AppState、Result<Json<..>, String>、DTO 转换）
- 错误映射：BudgetError → HTTP 400/404

**Non-Goals:**
- 不涉及前端改动
- 不新增 BudgetService 方法
- 不做权限控制或认证
- 不做分页

## Decisions

### 1. 新建独立 handler 文件 vs 合并到 report.rs

**选择**: 新建 `handlers/budget.rs`

**理由**: report.rs 专注于报表查询（balance-sheet、cash-flow），职责是"只读聚合"。预算 API 包含 CRUD 写操作，语义不同。独立文件保持单一职责，也便于路由注册。

### 2. FinancePeriod 序列化方式

**选择**: 在 DTO 层用字符串（"daily"/"weekly-sun"/"weekly-mon"/"monthly"/"yearly"），handler 内做 string ↔ FinancePeriod 转换。

**理由**: 与 CLI 的 `parse_period` 保持一致的命名约定。report.rs 已有 `parse_period` 函数可复用。避免在 domain 层引入 serde 依赖。

### 3. Decimal 序列化方式

**选择**: 序列化为字符串（`amount.to_string()`），与现有 TransactionDto/PostingDto 中 amount 字段的处理方式一致。

**理由**: 前端用 JavaScript，无法精确表示 Decimal。字符串避免精度丢失。现有 API 已统一用此模式。

### 4. 错误处理策略

**选择**: handler 中 `map_err(|e| e.to_string())` 返回 `Err(String)`，由 axum 自动映射为 HTTP 500。对特定 BudgetError 变体做匹配：BudgetNotFound → 404，验证类错误 → 400。

**理由**: 现有 handler 统一用 `Result<Json<T>, String>` 模式。需增强为按错误类型返回不同状态码。考虑引入简单的 `IntoResponse` 实现。

**替代方案**: 用 `(StatusCode, Json<ErrorResponse>)` 作为错误类型——但与现有模式不一致，改动大。选择在 budget handler 内部做 match 返回不同 StatusCode。

### 5. 路由注册

**选择**: budget.rs 内定义 `pub fn router() -> Router<Arc<AppState>>`，在 `router.rs` 中 `.merge(handlers::budget::router())`。

**理由**: 与 account.rs、channel.rs 等现有 handler 完全一致的模式。

## Risks / Trade-offs

- **[错误类型映射不够精细]** → BudgetError 有 8 个变体，目前只区分 404（NotFound）和 400（验证错误），其余归 500。对当前规模足够，未来如需更精细可逐个映射。
- **[budget status 查询性能]** → get_budget_status 会查 posting 聚合，数据量大时可能慢。当前数据规模无需优化，但需留意。
