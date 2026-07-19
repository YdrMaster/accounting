# calendar-daily-stats 任务

## 1. 后端：按天收支汇总

- [x] 1.1 在 `accounting-service/src/report/` 新增 `daily_summary.rs`，实现按天聚合查询（asset+equity 分录，正额为 income、负额绝对值为 expense，参数 from/to，仅返回有交易的日期），并在 `report/mod.rs` 导出
- [x] 1.2 为 daily_summary 编写单元/集成测试：普通收支日、转账日（收支各计）、无交易日期不返回、跨月范围
- [x] 1.3 在 `accounting-api/src/handlers/report.rs` 新增 `GET /api/reports/daily-summary` 路由与 handler，校验 `from`/`to` 必填及 `YYYY-MM-DD` 格式（错误返回 400），响应 `[{ date, income, expense }]`（金额为字符串）
- [x] 1.4 运行 `cargo test` 确认后端全部通过

## 2. 前端：API 与工具函数

- [x] 2.1 `accounting-web/src/types/api.ts` 新增 `DailySummaryDto`；`api/client.ts` 新增 `fetchDailySummary(from, to)`
- [x] 2.2 新增金额缩写工具 `formatCalendarAmount(value, locale)`（zh >99999 用"万"、en >9999 用"k"，保留 1 位小数）并编写单元测试（阈值边界、两种 locale、未达阈值原样显示）

## 3. 前端：日历格子展示

- [x] 3.1 `CalendarGrid.vue`：新增 `dailyStats` prop（Map/Record）与 `visible-range-change` 事件（挂载及翻月时携带网格首末日期，含补位天）
- [x] 3.2 `CalendarGrid.vue`：格子布局改为日期顶部居中 + 支出（红，上）/收入（绿，下）双行，无 +/- 符号；移除 `aspect-ratio: 1` 改为最小高度；删除 `.transaction-dot`；无交易日期不渲染金额行；选中/today 状态下金额保持红绿可区分
- [x] 3.3 `CalendarView.vue`：监听 `visible-range-change` 加载统计并传入 `CalendarGrid`；交易新建/编辑/删除后刷新当前可见范围统计
- [x] 3.4 locales（zh-CN / en）补充所需文案（如有），并确认缩写单位无需文案条目
- [x] 3.5 运行前端测试与 `npm run build`（或项目既有验证命令）确认通过；手动核对 zh/en 两种 locale 下的缩写显示与跨月补位格子的统计展示
