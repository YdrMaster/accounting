## Why

当前 accounting-web 的交易列表视图使用硬编码假数据，没有接入后端 API。需要实现真实数据接入，使交易列表能够展示实际的交易记录、按日分组、显示月收支汇总，并支持分页加载最新交易。

## What Changes

- 后端新增 `ReportService::summary` 接口，提供日/月收支汇总（资产类分录正负金额之和）
- 后端新增 `/api/reports/summary?from=&to=` 端点
- 后端 `TransactionDto` 新增 `tags: Vec<String>` 和 `member_name: Option<String>` 字段
- 后端 `PostingDto` 新增 `account_type: String` 字段（"asset"|"equity"|"income"|"expense"）
- 前端新增 API client 封装和 TypeScript 类型定义
- 前端新增 Pinia store 管理交易列表状态
- 前端重构 TransactionView 组件：
  - 移除硬编码假数据和月度预算卡片
  - 实现按日分组展示（日期、星期、当日收支汇总）
  - 实现交易卡片渲染（收支账户、成员、备注、金额、资产账户）
  - 实现月收支汇总展示（从后端 API 获取）
  - 支持分页加载最新交易
  - 调整字号和样式匹配设计稿

## Capabilities

### New Capabilities
- `transaction-api`: 后端 DTO 字段扩展，支持前端所需的交易展示数据
- `transaction-summary-api`: 后端新增收支汇总 service 接口和 API 端点
- `transaction-list-ui`: 前端交易列表视图，包括数据接入、分组展示、样式调整

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- 后端：`accounting-api/src/dto.rs`、`accounting-api/src/handlers/transaction.rs`
- 前端：`accounting-web/src/` 新增 `api/`、`types/`、`stores/` 目录，重构 `views/TransactionView.vue`
- 依赖：前端新增 `pinia`、`decimal.js` 依赖
