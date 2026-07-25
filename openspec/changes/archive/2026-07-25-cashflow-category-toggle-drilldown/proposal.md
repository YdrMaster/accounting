# cashflow-category-toggle-drilldown

## Why

现金流量 tab 目前并排展示收入/支出两张旭日图，下方却是一张按**资产账户**统计流入/流出/净额的资金流量表——图表看收支分类、表格看资产账户，两者口径不一致，用户无法从分类图表追到对应的金额明细。同时后端同时存在 `cash-flow`（资产口径）与 `category-breakdown`（收支口径）两个报表，数据重复、维护成本高；且 cash-flow 的 inflow/outflow 是按净额符号伪拆分的，同一账户双向流动时口径失真。

## What Changes

- **BREAKING** 重定义资金流量表：统计对象从 Assets 根下账户改为 Income/Expenses 两根下账户，数据结构从每账户 inflow/outflow/net 改为各层级账户的周期金额汇总（单金额、取绝对值），排除"不计预算"标签分录的口径不变。
- **BREAKING** 退役收支分类明细报表：删除 `category_breakdown.rs` 服务与 `GET /api/reports/category-breakdown` 端点，其职责由重定义后的资金流量表承担。
- **BREAKING** API 契约调整：`GET /api/reports/cash-flow` 响应改为 `{ period_start, period_end, income[], expense[] }`，明细项为 `{ account_id, parent_id, name, amount }`——旭日图、钻入联动、列表筛选一律使用 `account_id` 关联，名字仅作展示。
- 前端现金流量 tab 改版：收入/支出两张并排旭日图改为单页旭日图 + 「支出 | 收入」toggle 切换；下方资产流量表替换为收支变动详情列表（树状缩进、每级按金额降序、含占比与比例条）。
- 钻入联动：点击旭日图扇区下钻时，下方列表同步筛选为被钻账户及其各级子账户；点中心返回上级时列表随之回退；toggle 或周期切换时钻入状态重置。
- CLI `cash-flow` 命令同步改为收支账户口径。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `cash-flow-report`: 统计对象从资产账户改为收支账户；数据结构从 inflow/outflow/net 改为各层级单金额汇总。
- `report-module`: 模块结构中移除 `category_breakdown.rs`，收支分类明细职责并入资金流量表。
- `assets-visual-reports`: 「收支太阳图」改为 toggle 单图并新增钻入联动要求；「资金流量表」展示要求替换为树状收支变动详情列表。

## Impact

- **后端**：`accounting-service/src/report/cash_flow.rs` 重写、`category_breakdown.rs` 删除、`report/mod.rs` 模块导出调整；`accounting-api/src/handlers/report.rs` 端点合并与 DTO 变更。
- **API**：`/api/reports/cash-flow` 响应结构变更（breaking）；`/api/reports/category-breakdown` 移除（breaking）。
- **CLI**：`accounting-cli/src/cmd/report.rs` cash-flow 输出改口径；相关本地化文案与测试更新。
- **前端**：`accounting-web` 的 `CashFlowPanel.vue`、`CategorySunburst.vue`（点击事件外抛）、新增 toggle 与树状详情列表组件、`CashFlowTable.vue` 删除、`stores/report.ts`、`types/api.ts`、`api/client.ts`、本地化文案。
- **规格**：`openspec/specs/cash-flow-report`、`report-module`、`assets-visual-reports` 三份规格同步更新。
