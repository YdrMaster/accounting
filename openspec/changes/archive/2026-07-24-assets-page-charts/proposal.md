## Why

资产页面目前仅展示静态的资产负债表快照，用户无法看到资产随时间的变化趋势，也无法直观了解收入来源与支出去向的构成。后端已实现资金流量表 API 但前端未接入，历史趋势和分类明细则完全缺失后端支持。增加可视化报表能让记账数据从"记录"升级为"洞察"。

## What Changes

- 资产页面重构为双 tab 布局：「资产负债表」与「现金流量」，各 tab 顶部提供周期控件
- 资产负债表 tab 上方新增资产趋势折线图：总资产线与总负债线之间以半透明区域填充净资产，支持周/月/年粒度切换，显示点数由页宽决定
- 现金流量 tab 上方新增收入/支出两个太阳图（sunburst），环层对应账户层级深度，展示周期内收支分类构成
- 现金流量 tab 下方新增资金流量表，接入已有的 `GET /api/reports/cash-flow` API，带周期前后导航
- 新增后端 API `GET /api/reports/net-worth-trend`：按时间桶返回历史总资产与总负债
- 新增后端 API `GET /api/reports/category-breakdown`：按账户层级返回周期内收入/支出分类明细
- 前端引入 ECharts 依赖
- 多币种暂只统计基准币种，后续迭代再考虑折算

## Capabilities

### New Capabilities

- `net-worth-trend-report`: 资产趋势报表——按周/月/年时间桶聚合资产账户累计余额，拆分为总资产与总负债序列，提供 HTTP API
- `category-breakdown-report`: 收支分类报表——按账户层级汇总周期内 Income/Expenses 根下各账户金额，返回带路径的明细列表，提供 HTTP API
- `assets-visual-reports`: 资产页面可视化——tab 布局、ECharts 趋势折线图（净资产面积填充）、收支太阳图、资金流量表 UI 与周期导航

### Modified Capabilities

- `assets-view`: 页面从单一资产负债表视图重构为 tab 结构，资产负债表内容保留在第一个 tab 中
- `report-module`: 报表模块从 3 个子模块扩展为 5 个（新增 net_worth_trend.rs 与 category_breakdown.rs）

## Impact

- **后端**: `accounting-sql` 新增按账户×时间桶聚合查询；`accounting-service/src/report/` 新增两个服务模块；`accounting-api` 新增两个 handler 与路由
- **前端**: `accounting-web` 新增 ECharts 依赖；AssetsView.vue 重构为 tab 布局；新增趋势图、太阳图、流量表组件；report store 与 api client 扩展
- **API**: 新增 2 个只读 GET 端点，无破坏性变更
