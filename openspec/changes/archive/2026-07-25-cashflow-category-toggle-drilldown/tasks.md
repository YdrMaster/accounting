# cashflow-category-toggle-drilldown 任务

## 1. 后端服务层

- [x] 1.1 重写 `accounting-service/src/report/cash_flow.rs`：`CashFlowItem` 改为 `{ account, amount }`，`CashFlowReport` 改为 `{ period_start, period_end, income, expense }`，聚合逻辑改用 `sum_by_account_with_descendants`（参照待删的 `category_breakdown.rs`），保留排除"不计预算"标签口径
- [x] 1.2 改写 `cash_flow.rs` 单测：多层逐层汇总、收入负金额归一化、退款冲抵净额、排除标签四个场景
- [x] 1.3 删除 `accounting-service/src/report/category_breakdown.rs`，调整 `report/mod.rs` 模块导出

## 2. API 层

- [x] 2.1 修改 `accounting-api/src/handlers/report.rs` 的 cash_flow handler：响应改为 `{ period_start, period_end, income[], expense[] }`，明细项为 `{ account_id, parent_id, name, amount }`，`name` 按请求语言回退链解析
- [x] 2.2 移除 `/api/reports/category-breakdown` 路由、`CategoryBreakdownQuery/Response` 等相关类型

## 3. CLI

- [x] 3.1 修改 `accounting-cli/src/cmd/report.rs` cash-flow 命令：分 Income/Expenses 两节、树状缩进、每级按金额降序输出账户与金额
- [x] 3.2 更新 CLI locales（`zh-CN.yaml`、`en.yaml`）与 `accounting-cli/docs/commands/report.md` 文档，修复受影响的 CLI 测试

## 4. 前端数据层

- [x] 4.1 更新 `accounting-web/src/types/api.ts`：`CashFlowDto` 改为新契约（`income`/`expense` 明细含 `account_id`/`parent_id`/`name`/`amount`），删除 `CategoryBreakdownDto` 相关类型
- [x] 4.2 更新 `accounting-web/src/api/client.ts` 与 `stores/report.ts`：删除 `fetchCategoryBreakdown`，现金流量 tab 仅请求 cash-flow
- [x] 4.3 重写 `accounting-web/src/utils/sunburst.ts`：改为按 `account_id`/`parent_id` 建树（`Map<id, node>` 链接），保留 1% 过滤与伪子节点行为，节点携带 `account_id`；补充/更新其单测

## 5. 前端组件

- [x] 5.1 修改 `CategorySunburst.vue`：节点 data 携带 `account_id`，监听 echarts 点击事件并 `emit('drill', account_id | null)`（点中心返回时抛上级 id 或 null）
- [x] 5.2 新增「支出 | 收入」分段 toggle 组件（或在 CashFlowPanel 内实现），选中态样式参照截图
- [x] 5.3 新增收支变动详情列表组件：树状缩进、每级金额降序、行含名称/占比/金额/比例条，以 `drillId ?? 根id` 为起点筛选后代
- [x] 5.4 改造 `CashFlowPanel.vue`：单页旭日图 + toggle + 详情列表布局，持有 `side` 与 `drillId` 状态，toggle/周期/日期变化时重置 `drillId`
- [x] 5.5 删除 `CashFlowTable.vue`，更新 `locales/zh-CN.ts`、`en.ts` 文案

## 6. 验证

- [x] 6.1 `cargo test -p accounting-service -p accounting-api -p accounting-cli` 通过
- [x] 6.2 前端 `npm run test`（含 sunburst utils 单测）与 `npm run build`（vue-tsc）通过
- [x] 6.3 手动验证：启动服务，现金流量 tab 完成 toggle 切换、点击下钻/返回、列表联动、周期切换重置下钻
