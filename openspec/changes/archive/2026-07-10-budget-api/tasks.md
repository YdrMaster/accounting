## 1. DTO 定义

- [x] 1.1 在 `dto.rs` 中添加 `BudgetDto`（id, name, period, commodity_id）、`BudgetLimitDto`（account_id, amount）、`BudgetDetailDto`（budget, limits）、`BudgetStatusDto`（budget, period_start, period_end, items）、`BudgetItemStatusDto`（account_id, limit_amount, actual_amount, remaining, percentage）
- [x] 1.2 在 `dto.rs` 中添加 `CreateBudgetRequest`（name, period, commodity_id, limits）和 `UpdateBudgetRequest`（name, period, commodity_id, limits），limits 为 `Vec<BudgetLimitRequest>`（account_id, amount）
- [x] 1.3 添加 `FinancePeriod` 与字符串的互转函数（复用 report.rs 中 parse_period 的模式，增加反向 to_period_string）

## 2. Handler 实现

- [x] 2.1 创建 `handlers/budget.rs`，实现 `list_budgets` handler：调用 BudgetService.list_budgets()，转换为 Vec<BudgetDto> 返回
- [x] 2.2 实现 `create_budget` handler：解析 CreateBudgetRequest，调用 BudgetService.create_budget()，返回 201 + BudgetDto；对 BudgetError 做 match 区分 400/404/500
- [x] 2.3 实现 `get_budget_detail` handler：调用 BudgetService.get_budget_detail()，返回 BudgetDetailDto；BudgetNotFound 时返回 404
- [x] 2.4 实现 `update_budget` handler：解析 UpdateBudgetRequest，调用 BudgetService.update_budget()，返回 200；错误处理同 create
- [x] 2.5 实现 `delete_budget` handler：调用 BudgetService.delete_budget()，返回 200；BudgetNotFound 时返回 404
- [x] 2.6 实现 `get_budget_status` handler：解析 date 查询参数（默认当天），调用 BudgetService.get_budget_status()，返回 BudgetStatusDto；日期格式错误返回 400

## 3. 路由注册

- [x] 3.1 在 `handlers/budget.rs` 中定义 `pub fn router() -> Router<Arc<AppState>>`，注册 6 个路由（GET/POST /api/budgets, GET/PUT/DELETE /api/budgets/:id, GET /api/budgets/:id/status）
- [x] 3.2 在 `handlers/mod.rs` 中添加 `pub mod budget;`
- [x] 3.3 在 `router.rs` 的 `create_app` 中 `.merge(handlers::budget::router())`

## 4. 验证

- [x] 4.1 编译通过，无 warning
- [x] 4.2 手动测试：用 curl 测试 6 个端点的正常路径和错误路径
