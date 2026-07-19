# calendar-daily-stats

## Why

日历页的日期格子目前只显示一个"有交易"圆点，大量空间被浪费，用户无法一眼看出每日收支规模。参考主流记账应用，日历格子应直接显示每日支出/收入金额，让日历本身成为收支统计视图。

## What Changes

- 后端新增 `GET /api/reports/daily-summary?from=YYYY-MM-DD&to=YYYY-MM-DD` 端点，按天返回收支汇总（口径与现有 `transaction-summary-api` 的汇总计算规则一致：资产类分录正额为收入、负额绝对值为支出），仅返回有交易的日期。
- 前端日历格子布局调整：日期数字上移至格子顶部，下方两行显示当日支出（上行，红色）和收入（下行，绿色），仅通过位置和颜色区分，不显示 +/- 符号；无交易的日期不显示金额行。
- 统计范围按日历**可见范围**（含前后月份补位天数）请求，跨月补位格子同样显示统计。
- 金额按 locale 缩写：中文模式超过 99999 切换为"万"为单位，英文模式超过 9999 切换为"k"为单位。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `transaction-summary-api`: 新增按天粒度的收支汇总端点（daily-summary），复用既有汇总计算规则。
- `calendar-view`: 日期格子的"有交易标记"升级为显示每日支出/收入金额，格子布局（日期上移 + 双行金额）与金额缩写规则随之变化。

## Impact

- `accounting-service`：report 模块新增按天汇总查询。
- `accounting-api`：`handlers/report.rs` 新增 daily-summary 路由。
- `accounting-web`：`CalendarGrid.vue`（格子布局与统计展示）、`CalendarView.vue`（按可见范围加载统计）、`api/client.ts`、`types/api.ts`、locales（zh-CN / en）。
- 无破坏性变更：既有端点与页面行为保持兼容。
