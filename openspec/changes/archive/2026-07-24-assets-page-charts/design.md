## Context

资产页面（AssetsView.vue）当前仅调用 `GET /api/reports/balance-sheet` 展示静态快照。后端报表模块（`accounting-service/src/report/`）已有 balance_sheet、cash_flow、budget、daily_summary 四个服务，其中 cash_flow 无前端消费。SQL 层已有 `sum_by_account_with_descendants`（闭包表层级聚合）和 `posting_sum_by_period` 等共享查询。前端无图表库，有 @vueuse/core 和 decimal.js。

## Goals / Non-Goals

**Goals:**
- 资产页双 tab（资产负债表 / 现金流量），各 tab 顶部周期控件
- 趋势折线图：总资产线与总负债线之间半透明填充净资产区域，周/月/年粒度，显示点数随页宽自适应
- 收支太阳图：环层 = 账户层级，收入与支出各一个，与流量表共享周期导航
- 资金流量表 UI 接入已有 cash-flow API
- 新增两个只读 GET 端点，复用现有 handler 模式（金额字符串序列化、按语言解析账户路径）

**Non-Goals:**
- 多币种折算（本期仅统计单一币种，commodity 参数默认 1，与 cash-flow handler 一致）
- 日粒度趋势
- 太阳图点击跳转到其他页面（图内缩放下钻属于 Goals，见 Decision 10）
- 现金流量表编辑能力

## Decisions

### 1. 趋势数据计算：SQL 按账户×日聚合 + Rust 侧分桶前缀和

SQL 查询返回 `GROUP BY account_id, date(t.date_time)` 的每日增量，Rust 服务层用 `FinancePeriod::period_range(day).0` 将每日归入桶，按 (桶, 账户) 累加后排序，逐账户前缀和，再按符号拆分为每期总资产（正余额之和）与总负债（负余额绝对值之和）。

**备选方案**：纯 SQL 窗口函数（`SUM OVER (PARTITION BY account ORDER BY bucket)`）。放弃原因：SQLite 的周桶 strftime 无法控制周起始日（需与 FinancePeriod::WeeklyFromMonday 一致），且前缀和 + 符号拆分逻辑在 Rust 中可直接单元测试。个人记账数据量（日级行数通常 < 10k）下性能无差异。

### 2. 趋势 API 形状

```
GET /api/reports/net-worth-trend?period=weekly|monthly|yearly&commodity=1

Response:
{
  "period": "monthly",
  "points": [
    { "date": "2026-06-01", "assets": "80000.00", "liabilities": "18000.00" },
    ...
  ]
}
```

- `date` 为桶起始日；返回全量历史，前端按页宽截取最近 N 点
- `period` 参数映射：weekly → WeeklyFromMonday，monthly → Monthly，yearly → Yearly；缺省 monthly；非法值 400
- 净资产由前端用 decimal.js 计算（assets − liabilities）

### 3. 分类明细 API：复用 sum_by_account_with_descendants，一次返回收支两棵树

```
GET /api/reports/category-breakdown?date=&period=&commodity=

Response:
{
  "period_start": "2026-07-01",
  "period_end": "2026-07-31",
  "income": [{ "account": "Income:工资", "amount": "15000.00" }, ...],
  "expense": [{ "account": "Expenses:餐饮:外卖", "amount": "500.00" }, ...]
}
```

- 分别以 Income 根、Expenses 根调用 `sum_by_account_with_descendants`，闭包表自动给出每一层祖先的汇总
- 排除 exclude-from-budget 标签（与 cash_flow 一致）
- 金额取绝对值后返回（Income 侧分录为负值，Expenses 侧为正值，统一为正数便于前端绘图）
- 账户路径经 `load_account_paths` 按请求语言解析

### 4. 太阳图树构建：前端从扁平路径组装，伪子节点承载中间层直接分录 + 1% 过滤

后端返回每个祖先的含后代汇总值，前端计算：节点自身值 = 本节点汇总 − Σ 子节点汇总。若自身值 > 0（存在直接记入中间层的分录），插入与父同名的伪子节点承载差额。

**1% 过滤**：每一级仅保留超过该级总额 1% 的子节点（含伪子节点），被过滤节点的整个子树一并丢弃；级总额为 0 时不过滤。中间节点**不带 value**（ECharts 按可见子节点求和分配角度），因此被过滤项不参与比例计算，可见子节点归一化填满整环；中间节点的真实总额存于自定义 `total` 字段，tooltip 优先取该字段展示，避免父级悬停金额因过滤而虚低。

### 5. 趋势图渲染：stack 面积技巧实现三线效果

两个 series：
- `liabilities`：折线，无面积填充
- `netWorth`（assets − liabilities）：stack 在 liabilities 之上，带半透明 areaStyle

视觉上顶边 = 资产线，底边 = 负债线，中间填充区域 = 净资产。tooltip 同时展示总资产、总负债、净资产三个值。

### 6. 点数自适应：后端全量 + 前端按容器宽度切片

`useResizeObserver`（@vueuse/core 已有）监听图表容器，`N = max(2, floor(width / 48))`，取 points 数组最后 N 项渲染。resize 时仅重新切片 + setOption，不重新请求。

### 7. ECharts 按需引入

从 `echarts/core` 模块化注册：LineChart、SunburstChart、CanvasRenderer、TooltipComponent、GridComponent。避免全量引入（~1MB → ~500KB 未压缩差异）。

### 8. Tab 与数据加载策略

AssetsView 持有 `activeTab: ref<'balance' | 'cashflow'>`，v-if 切换。各 tab 组件 mount 时自行加载数据（懒加载：首次切入才请求）。周期导航（◀ ▶ + 粒度下拉）为现金流量 tab 内共享状态，同时驱动太阳图与流量表；资产负债表 tab 仅有粒度下拉，驱动趋势图。

### 9. 周期导航的前端日期运算

◀ ▶ 按当前粒度偏移参考日期：weekly ±7 天，monthly 用 Date setMonth ±1，yearly ±1 年。不引入 dayjs，原生 Date 足够。

### 10. 太阳图改为可下钻的动态旭日图（Plotly 风格）

初版太阳图 `animation: false` + 外部引线标签，在层级浅（常见 2 层）时视觉上接近饼图，观感差。参考 Plotly 旭日图交互，改为 ECharts 原生下钻：点击扇区平滑缩放至圆心展开子类，点击中心返回上级。

最终样式采用 ECharts 官方 `sunburst-borderRadius` 示例：扇区圆角（`borderRadius: 7` + 2px 描边）。曾尝试扇区内嵌标签（`position: 'inside'` + `rotate: 'radial'`），但小扇区上文字必然重叠截断，hideOverlap 也只能隐藏不能根治；也曾改为完全无标签只靠 tooltip，但分类构成不够一目了然。最终采用**外侧水平标签**（`position: 'outside'` + `rotate: 0` + `minAngle: 5`，内容 = 分类名 + 百分比，颜色跟随扇区），配合引线呈现；过小扇区不显示标签，tooltip 始终可用。

**实现要点**：
- 下钻交互：`nodeClick: 'rootToNode'`。注意 ECharts 6.1.0 运行时代码只识别 `'rootToNode'`（SunburstView.js），类型声明中的 `'zoomToNode'` 无效——这是一个版本文档/类型与运行时不一致的坑
- 颜色：采用官方 `sunburst-drink` 示例的一级分类配色（9 色 PALETTE 循环指派给一级节点）；后代节点不显式着色，由 ECharts 自动提亮为同色系浅色（sunburstVisual 的 `lift`），保证同级区分度
- 标签：`position: 'outside'` 配合 `labelLine` 引线；`rotate: 0` 保持水平；`minAngle: 5` 隐藏过小扇区的标签；标签 formatter 与 tooltip 一致地取 `data.total`（中间节点真实总额）计算百分比
- 里圈标签：ECharts sunburst 的 `position: 'outside'` 仅将标签放到**节点自身环**的外侧（SunburstPiece.js 中 `r = layout.r + distance`），且完全不支持 labelLine——里圈标签只能落在扇区上。放弃"白线牵引出图外"（需多重 pie 重建，代价是失去原生下钻动画），按用户决定采用白色文字描边（`textBorderColor: rgba(255,255,255,0.9)`）保证可辨，仅应用于非最外环节点
- 尺寸：图表容器 360px，radius ['12%', '75%']（外圈留边距给外侧标签）

**取舍**：
- 不引入 Plotly.js（+3MB 依赖）或手写 D3，ECharts 原生下钻可达到同构体验，改动集中于 CategorySunburst.vue
- 伪子节点下钻（看到占满整环的同名块）等边界不做特殊处理，先观察实际效果

## Risks / Trade-offs

- **[当前桶不完整]** 趋势图包含进行中的当前周期（如月中），末点数值偏低 → 可接受，反映实时状态；tooltip 中日期标注桶起始日供用户理解
- **[中间层直接分录]** 用户可能直接记账到非叶子账户 → 伪子节点模式保证太阳图数值一致性，视觉上出现与父同名的内环段
- **[ECharts 包体积]** 新增约 500KB（未压缩）→ 按需引入已最小化；SPA 单次加载可接受
- **[全量历史传输]** 数据增长后 trend 响应变大 → 当前规模（个人记账）无压力；若需要可后续加 limit 参数，API 形状不变
