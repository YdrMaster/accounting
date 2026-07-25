# Design: PostingDto 增加 account_id

## Context

编辑交易时，前端 `TransactionFormOverlay.vue` 的 `loadTransaction` 将每条分录的 `accountId` 硬编码为 `null`，因为后端 `PostingDto` 只有 `account: String`（账户路径名），不含账户 id。账户选择器 `AccountPicker` 的显示完全由 `accountId`（number）驱动，因此编辑时账户永远显示占位符"选择账户"，且 `canSubmit` 校验 `accountId` 非空导致无法直接保存。

提交方向（前端 → 后端）走账户名：`handleSubmit` 只发送 `accountName`，后端按名称解析账户。本设计只补齐查询方向（后端 → 前端）的 id 传递，不改变提交协议。

## Goals / Non-Goals

**Goals:**

- `GET /api/transactions/:id`（及其他返回 PostingDto 的端点）响应中每条分录携带 `account_id`
- 编辑表单打开时账户选择器正确回显已有账户，无需重新点选即可保存

**Non-Goals:**

- 不改变创建/更新交易的请求格式（仍按账户名提交）
- 不做前端按名称反查 id 的兜底逻辑
- 不处理账户被删除/改名后名称失效的历史数据问题（与本 bug 无关）

## Decisions

### 决策 1：DTO 新增字段而非前端按名称反查

后端 `posting_to_dto` 内部本来就持有 `p.account_id`，直接放入 DTO 成本极低且精确。前端按名称反查依赖名称唯一性和账户未关闭，脆弱且难排查。

### 决策 2：字段类型 `i64` / `number`，与内部 `AccountId` 一致

现有 `posting_to_dto` 中 `p.account_id.0` 即 `i64`，与 `AccountDto.id`（number）一致，前端可直接与账户列表的 `id` 匹配。

### 决策 3：新增字段而非替换 `account` 字段

`account`（路径名）仍被列表展示等场景使用，保留；`account_id` 为纯新增，向后兼容，旧客户端不受影响。

## Risks / Trade-offs

- [其他返回 PostingDto 的端点遗漏填充 account_id] → `posting_to_dto` 是统一转换入口，在该函数内填充即可覆盖所有调用点；实现时用 Grep 确认无其他手工构造 PostingDto 的位置。
- [前后端类型不同步导致前端读到 undefined] → 同一 change 内同步修改 `accounting-web/src/types/api.ts`，并用 `loadTransaction` 的填充作为验证点。
