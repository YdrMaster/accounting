## Context

当前 accounting-web 使用 Vue 3 + TypeScript，采用 ResponsiveShell 多面板横向滑动布局。交易列表视图（TransactionView）目前使用硬编码假数据，没有 API 接入层。

后端 accounting-api 使用 Rust + Axum，已实现交易 CRUD 端点（`/api/transactions`），返回 `TransactionDto` 和 `PostingDto`。但 DTO 缺少前端展示所需的字段：交易标签名、成员名、账户类型。

## Goals / Non-Goals

**Goals:**
- 交易列表展示真实数据，按日分组，显示月收支汇总
- 交易卡片正确展示收支账户、成员、备注、金额、资产账户
- 支持分页加载最新交易
- 样式匹配设计稿（字号、颜色、布局）

**Non-Goals:**
- 预算功能（保持假数据，后续实现）
- 交易创建/编辑（后续实现）
- 交易筛选/搜索（后续实现）
- 日历视图、资产视图（后续实现）

## Decisions

### 1. 月收支汇总计算位置

**决策**: 在 service 层新增 `ReportService::summary` 接口，通过数据库查询直接计算。

**理由**: 
- 月汇总是高频使用的数据，每次打开交易列表都需要
- 数据库层面聚合效率更高（SUM + WHERE 条件）
- 前端无需遍历所有交易计算，减少客户端负载
- 日汇总也可复用同一接口（按日分组查询）

**接口设计**:
```rust
// ReportService
pub async fn summary(&self, from: NaiveDate, to: NaiveDate) -> Result<Summary, Error>

// Summary 结构
pub struct Summary {
    pub income: Decimal,   // 资产类分录正金额之和
    pub expense: Decimal,  // 资产类分录负金额之和（绝对值）
}

// SQL 逻辑
// income = SUM(amount) WHERE account_type IN ('asset','equity') AND amount > 0 AND date BETWEEN from AND to
// expense = ABS(SUM(amount)) WHERE account_type IN ('asset','equity') AND amount < 0 AND date BETWEEN from AND to
```

**API 端点**: `GET /api/reports/summary?from=2026-02-01&to=2026-02-28`

### 2. 交易卡片金额计算逻辑

**决策**: 前端计算，基于 `account_type` 区分收支账户和资产账户。

**逻辑**:
- 收支账户 = `income` + `expense` 类型的 posting
- 资产账户 = `asset` + `equity` 类型的 posting
- 金额 = 资产账户 postings 的 amount 之和
- 若和为 0（转账），显示正值之和

### 3. 前端状态管理

**决策**: 使用 Pinia store 管理交易列表状态。

**理由**:
- 交易列表需要跨组件共享（月汇总、分组展示）
- Pinia 是 Vue 3 官方推荐的状态管理方案
- 支持缓存和增量加载

### 4. 分页策略

**决策**: 基于日期的分页，每次加载一个月的数据。

**理由**:
- 月收支汇总是按月计算的，自然边界
- 用户通常查看当月交易
- 简化分页逻辑，避免复杂的 offset/limit

## Risks / Trade-offs

- **[数据一致性]** 前端分组依赖客户端时间 → 使用服务器返回的 `date_time` 字段
- **[向后兼容]** DTO 新增字段不影响现有客户端 → 无风险
- **[数据库性能]** 汇总查询需要 JOIN account 表获取 account_type → 可添加索引优化
