# calendar-daily-stats 设计

## Context

日历页（`accounting-web/src/views/CalendarView.vue` + `components/CalendarGrid.vue`）当前只通过 `transactionDates: Set<string>` 知道某天"有无交易"，格子内显示一个绿点。后端报表模块（`accounting-service/src/report/`）已有 balance_sheet、cash_flow 两个服务，`accounting-api/src/handlers/report.rs` 暴露对应路由。`transaction-summary-api` spec 已定义收支口径（资产类 asset+equity 分录正额为收入、负额绝对值为支出），但尚无代码实现。实际库中仅 CNY 一个 commodity。

## Goals / Non-Goals

**Goals:**

- 新增按天粒度的收支汇总端点，供日历页（及未来其他视图）使用。
- 日历格子直接展示每日支出/收入，范围覆盖可见的跨月补位日期。
- 金额按 locale 缩写，保证窄格子不溢出。

**Non-Goals:**

- 不实现显示模式切换（收&支/结余/收入/支出），本期固定双行收支。
- 不做热力图、tooltip 等其他可视化。
- 不处理多币种换算（当前仅 CNY；端点聚合所有 commodity，与现有报表行为一致）。
- 不改动既有 `GET /api/reports/summary`（该 spec 端点尚未实现，不在本期范围）。

## Decisions

### 1. 后端：独立 daily-summary 查询，而非前端聚合

在 `accounting-service/src/report/` 新增 `daily_summary.rs`：单条 SQL 按天分组聚合——

```sql
SELECT date(t.date_time) AS day,
       SUM(CASE WHEN p.amount > 0 THEN p.amount ELSE 0 END) AS income,
       SUM(CASE WHEN p.amount < 0 THEN -p.amount ELSE 0 END) AS expense
FROM postings p
JOIN transactions t ON t.id = p.transaction_id
JOIN accounts a ON a.id = p.account_id
WHERE a.type IN ('asset', 'equity') AND date(t.date_time) BETWEEN ? AND ?
GROUP BY day
```

（具体表名/字段以 `accounting-sql` 现有 schema 为准，实现时对齐 cash_flow 的写法。）

理由：一次 HTTP + 一次聚合查询即可渲染整月日历，避免把整月交易明细拉到前端；结果天然只含有交易的日期，契合 spec「无交易日期不返回」。

`accounting-api/src/handlers/report.rs` 新增 `GET /api/reports/daily-summary`，参数校验（`from`/`to` 必填、`YYYY-MM-DD` 格式）沿用 cash-flow 的模式，响应为 `[{ date, income, expense }]`，金额为字符串（对齐现有 API 的 Decimal 序列化惯例）。

### 2. 前端：CalendarGrid 暴露可见范围，CalendarView 负责取数

月份状态在 `CalendarGrid` 内部，而数据加载惯例在 View 层。因此：

- `CalendarGrid` 新增 `visible-range-change` 事件（挂载及翻月时触发），携带网格首/末日期（含补位天）。
- `CalendarView` 监听该事件，调用 `fetchDailySummary(from, to)`，将结果存为 `Map<date, {income, expense}>`，以 prop 传给 `CalendarGrid`。
- 交易增删改后使统计失效：复用现有 store 的刷新时机，简单起见在 `onFormSaved`/`onDeleteTx` 后重新请求当前可见范围。

理由：保持 CalendarGrid 纯展示、数据流单向，与现有 `transaction-dates` prop 模式一致。

### 3. 格子布局：日期置顶 + 双行金额，无符号

```
┌──────────┐
│   23     │   日期（顶部居中）
│  340.92  │   支出（红色，上）
│   72.66  │   收入（绿色，下）
└──────────┘
```

- 移除 `aspect-ratio: 1`，格子改为固定最小高度（约 3.5rem），容纳三行。
- 支出恒在上、收入恒在下；某一项为 0/不存在时不渲染该行。
- 选中态下金额保持红/绿原色（强制转白会导致收支无法区分）；today 边框样式保留。
- 删除 `.transaction-dot`（被金额取代）。

### 4. 金额缩写：展示层工具函数

新增 `utils/amount.ts`（或并入现有 utils）的 `formatCalendarAmount(value: string, locale: string)`：

- zh：|v| > 99999 → `(v/10000)` 保留 1 位小数 + `万`；en：|v| > 9999 → `(v/1000)` 保留 1 位小数 + `k`。
- 阈值判断用原始值，四舍五入后恰好越界不特殊处理（如 99999.4 → 显示 `99999.4`）。
- locale 取自 vue-i18n 当前 locale，无需新增文案条目。

## Risks / Trade-offs

- [缩写损失精度，用户可能想看确切金额] → 点击下方交易列表的日汇总行已显示精确收支（TransactionList day-header），日历格子定位为速览。
- [格子变高导致日历整体占用更多纵向空间] → 单格约 3.5rem、6 行约 21rem，桌面/移动均可接受；若后续拥挤再做紧凑模式。
- [每日汇总与交易列表口径不一致的风险] → 两者都基于资产类分录求和；后端口径以 `transaction-summary-api` spec 为准，实现时对照。
