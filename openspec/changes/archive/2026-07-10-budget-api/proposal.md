## Why

预算系统的 Service 层和数据库层已完整实现（CRUD + 执行查询），CLI 也已映射，但缺少 HTTP API handler，导致前端无法访问预算功能。前端预算页目前使用硬编码数据。需要将已有的 BudgetService 能力暴露为 REST API，为前端接入铺路。

## What Changes

- 新增 `handlers/budget.rs`，实现 6 个 HTTP 端点：
  - `GET /api/budgets` — 列出所有预算表
  - `POST /api/budgets` — 创建预算表
  - `GET /api/budgets/:id` — 获取预算表详情（含限额列表）
  - `PUT /api/budgets/:id` — 更新预算表
  - `DELETE /api/budgets/:id` — 删除预算表
  - `GET /api/budgets/:id/status` — 查询预算执行情况
- 在 `dto.rs` 中新增预算相关 DTO（请求/响应）
- 在 `router.rs` 中注册 budget handler

## Capabilities

### New Capabilities
- `budget-api`: 预算系统的 HTTP REST API，包括预算表 CRUD 和执行情况查询端点

### Modified Capabilities
<!-- 无需修改现有 spec 的需求层面 -->

## Impact

- **后端代码**: `accounting-api` crate 新增 `handlers/budget.rs`，修改 `dto.rs` 和 `router.rs`
- **API 表面**: 新增 6 个 REST 端点，无破坏性变更
- **依赖**: 复用现有 `BudgetService`、`BalanceSheetService`，无新依赖
- **前端**: 本次不涉及前端改动，仅为后续前端接入提供 API 基础
