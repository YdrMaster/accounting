# Proposal: saving-plan

## Why

预算是「支出流量的上限」，但缺少与之平行的「资产存量的下限」工具：用户无法表达「9 月底在余额宝+微信零钱中备齐 5000 元旅行基金」「每月 30 日前招行账户须留有 6000 元交房租」这类攒钱目标。同时预算自身也需要演进：一次性目标（旅行、季度差旅）需要截止日期使计划自动失效，无节奏的目标需要周期可空。

## What Changes

- 新增攒钱计划（SavingPlan）全链路功能：核心域模型、SQL 存储、service 状态计算、REST API、CLI 命令（不含前端 UI）。
- 攒钱计划模型：账户集合共享一个目标金额（pooled target），判定口径为账户集合的**实时余额合计**（含后代子账户），`balance ≥ target` 即达标，状态随时可查当前缺口（gap）。
- **BREAKING** 预算校验收紧：预算账户仅限 Expenses 子树；攒钱计划账户仅限 Assets 子树（仅写入时校验，不迁移历史数据）。
- 预算与攒钱计划共用新时间语义：`period` 可空（NULL = 一次性/无节奏）、新增 `deadline` 列（过期后计划失效）。
  - 一次性预算的计量窗口 = 从最早记录累计到 min(查询日, deadline)。
  - `FinancePeriod` 周期计算器接口不变，失效判断在 service 层进行。
- 状态查询在 deadline 过期后返回 200 + `expired: true` 标志（预算与攒钱计划一致）。

## Capabilities

### New Capabilities

- `saving-plan-model`: 攒钱计划核心域模型（SavingPlan、账户集合、共享目标金额、nullable period、deadline、校验规则）。
- `saving-plan-tracking`: 攒钱计划 service 层 CRUD、账户限制（Assets 子树）、失效判定。
- `saving-plan-report`: SavingPlanService 状态计算（余额聚合含后代、gap/达标判定、expired 标志）。
- `saving-plan-api`: 攒钱计划 REST 端点（CRUD + status）。
- `saving-plan-cli`: 攒钱计划命令行（create/list/show/update/delete）。

### Modified Capabilities

- `budget-model`: `budgets` 表新增 `deadline` 列；`period` 改为可空；账户校验收紧为仅 Expenses 子树。
- `budget-tracking`: 一次性预算（period 为空）的计量窗口语义；deadline 失效判定；写入校验的账户类型限制。
- `budget-report`: BudgetStatus 增加 `expired` 标志与失效后的返回结构。
- `budget-api`: 创建/更新 DTO 增加可选 `deadline`、`period` 改为可选；status 响应增加 `expired` 字段。
- `budget-cli`: `create`/`update` 增加 `--deadline` 选项，`--period` 改为可选。

## Impact

- **核心域**: `accounting/src/saving_plan.rs`（新）、`accounting/src/budget.rs`、`accounting/src/id.rs`（新增 SavingPlanId）。
- **SQL 层**: `accounting-sql/src/schema.rs`（新表 saving_plans / saving_plan_accounts / saving_plan_names；budgets 加列、period 改可空）、`accounting-sql/src/repo/`（新增 saving_plan.rs、余额查询）、`accounting-sql/src/names.rs`、`accounting-sql/src/database.rs`。
- **service 层**: `accounting-service/src/report/saving_plan.rs`（新）、`accounting-service/src/report/budget.rs`。
- **API 层**: `accounting-api/src/handlers/saving_plan.rs`（新）、`handlers/budget.rs`、`dto.rs`、`router.rs`。
- **CLI 层**: `accounting-cli/src/cmd/saving_plan.rs`（新）、`cmd/budget.rs`、`resolver.rs`、locales。
- **前端**: 本次不变（budget API 响应新增可选字段，前端可忽略）。
- **数据库**: `SCHEMA_STATEMENTS` 追加幂等 DDL；无存量数据，不做迁移，旧库文件删除重建。
