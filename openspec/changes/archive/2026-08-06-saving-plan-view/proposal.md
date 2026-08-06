# Proposal: saving-plan-view

## Why

攒钱计划已全链路落地（含全局资金分配与满足率），但完全没有前端界面，只能走 CLI/API。同时预算页的执行情况也从未实现 UI（`loadStatus` 停在 store 层）——预算表卡片上看不到剩余/超支。本次把两个页面的状态可视化一次补齐，并让预算表单跟上后端已支持的一次性/deadline 能力。

## What Changes

- 新增攒钱计划页（SavingPlanView）：卡片列表（满足率环形 + 达标/缺口/已失效徽标）、卡片展开状态详情（余额/缺口/met + 每账户分配明细）、抽屉表单（创建/编辑/删除）。
- 后端新增批量状态端点：`GET /api/saving-plans/statuses?date=`（按检查点序返回，含满足率）与 `GET /api/budgets/statuses?date=`，避免前端 N+1。
- 预算页（BudgetView）补执行情况 UI：卡片环形（周期内剩余/超支，红色标记超支）、展开各账户 limit/actual/remaining/percentage 明细。
- 预算表单扩展：period 增加「一次性」选项、deadline 日期输入；攒钱计划/预算表单的账户选择分别限制 Assets/Expenses 子树（AccountPicker 加类型过滤）。
- 攒钱计划表单字段：名称、周期（5 种 + 一次性）、deadline、目标金额、账户多选；币种与预算一致硬编码 CNY。
- **BREAKING**（前端内部）：无——API 均为新增端点，既有端点不变。

## Capabilities

### New Capabilities

- `saving-plan-view`: 攒钱计划页面——列表（满足率环形）、状态详情（账户分配明细）、抽屉表单、页面注册与 i18n。

### Modified Capabilities

- `saving-plan-api`: 新增 `GET /api/saving-plans/statuses` 批量状态端点。
- `budget-api`: 新增 `GET /api/budgets/statuses` 批量状态端点。
- `budget-view`: 预算列表增加执行情况环形与展开明细；创建/编辑表单增加一次性周期选项、deadline 字段与支出账户限制。

## Impact

- **后端**: `accounting-service`（list_budget_statuses 薄封装）、`accounting-api`（2 个新端点 + 测试）。
- **前端**: `accounting-web/src/{types/api.ts, api/client.ts, stores/savingPlan.ts, stores/budget.ts, views/SavingPlanView.vue(新), views/BudgetView.vue, components/layout/AccountPicker*.vue, composables/useResponsiveLayout.ts, components/layout/ResponsiveShell.vue, locales/{zh-CN,en}.ts}`，新增公共环形进度组件。
- **测试**: 后端端点集成测试；前端 api client spec + 视图组件测试（vitest）。
- **数据库**: 无变更。
