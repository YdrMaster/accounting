## 1. 后端 SQL 层

- [x] 1.1 在 `accounting-sql/src/repo/posting.rs` 新增 `posting_daily_delta_by_account` 查询：Assets 根下所有账户按 (account_id, date) 聚合分录净额，返回 Vec<(AccountId, NaiveDate, Decimal)>；编写单元测试覆盖多账户多日、Equity 根排除
- [x] 1.2 运行 `cargo test -p accounting-sql` 确认通过

## 2. 后端服务层：资产趋势

- [x] 2.1 新建 `accounting-service/src/report/net_worth_trend.rs`：定义 NetWorthTrendPoint { date, assets, liabilities } 与 NetWorthTrendService
- [x] 2.2 实现分桶前缀和算法：日增量按 period_range 归桶 → 按 (桶, 账户) 累加 → 逐账户前缀和 → 按符号拆分 assets/liabilities；编写单元测试覆盖跨桶累计、空桶延续、正负拆分
- [x] 2.3 在 report mod.rs 注册 net_worth_trend 模块；运行 `cargo test -p accounting-service` 确认通过

## 3. 后端服务层：收支分类明细

- [x] 3.1 新建 `accounting-service/src/report/category_breakdown.rs`：定义 CategoryBreakdownItem { account: Account, amount: Decimal } 与 CategoryBreakdownService
- [x] 3.2 实现分类明细：分别以 Income/Expenses 根调用 sum_by_account_with_descendants，金额取绝对值，排除 exclude-from-budget 标签；编写单元测试覆盖多层级汇总、收入负值归一、标签排除
- [x] 3.3 运行 `cargo test -p accounting-service` 确认通过

## 4. 后端 API 层

- [x] 4.1 在 `accounting-api/src/handlers/report.rs` 新增 net_worth_trend handler：解析 period（weekly→WeeklyFromMonday / monthly / yearly，缺省 monthly，非法值 400）与 commodity（缺省 1），返回 { period, points }
- [x] 4.2 新增 category_breakdown handler：解析 date/period/commodity，调用 load_account_paths 解析路径，返回 { period_start, period_end, income, expense }
- [x] 4.3 注册路由 `/api/reports/net-worth-trend` 与 `/api/reports/category-breakdown`
- [x] 4.4 运行 `cargo test -p accounting-api && cargo clippy` 确认通过

## 5. 前端基础设施

- [x] 5.1 安装 echarts 依赖（`npm install echarts`），创建 `src/utils/echarts.ts` 按需注册 LineChart、SunburstChart、CanvasRenderer、TooltipComponent、GridComponent
- [x] 5.2 在 `src/types/api.ts` 新增 NetWorthTrendDto、CategoryBreakdownDto 类型；在 `src/api/client.ts` 新增 fetchNetWorthTrend(period) 与 fetchCategoryBreakdown(date, period) 函数
- [x] 5.3 在 `src/stores/report.ts` 扩展：趋势数据状态与 loadNetWorthTrend action、分类明细状态与 loadCategoryBreakdown action、现金流量状态与 loadCashFlow action

## 6. 前端：Tab 布局与周期控件

- [x] 6.1 重构 AssetsView.vue 为双 tab 结构（资产负债表 / 现金流量），activeTab ref 控制 v-if 渲染，默认资产负债表；现有余额卡片内容提取为 BalanceSheetPanel 组件
- [x] 6.2 实现周期粒度下拉组件（周/月/年）与周期导航组件（◀ 日期 ▶），导航步长随粒度变化（weekly ±7天 / monthly ±1月 / yearly ±1年）
- [x] 6.3 添加相关 i18n 文案（zh-CN / en）

## 7. 前端：资产趋势折线图

- [x] 7.1 新建 NetWorthTrendChart.vue：消费 store 趋势数据，用 decimal.js 计算净资产序列，渲染负债折线 + 净资产堆叠面积（半透明填充），tooltip 显示三个值
- [x] 7.2 实现点数自适应：useResizeObserver 监听容器，N = max(2, floor(width/48)) 截取最近 N 点，resize 时仅重新切片
- [x] 7.3 粒度切换时重新请求 API 并更新图表；空数据显示空状态提示

## 8. 前端：收支太阳图

- [x] 8.1 新建工具函数 `buildSunburstTree(items)`：从扁平路径列表组装树，计算节点自身值（本节点 − Σ 子节点），正值差额插入同名伪子节点
- [x] 8.2 新建 CategorySunburst.vue：渲染单个太阳图（中心显示总额），编写 buildSunburstTree 的 Vitest 单元测试
- [x] 8.3 在现金流量 tab 中并排放置收入/支出两个太阳图，响应周期导航刷新

## 9. 前端：资金流量表

- [x] 9.1 新建 CashFlowTable.vue：表格渲染 items（账户/流入/流出/净额）+ 合计行 + 周期范围显示，金额用 decimal.js 格式化
- [x] 9.2 现金流量 tab 组装：周期导航同时驱动太阳图与流量表数据加载

## 10. 集成验证

- [x] 10.1 运行全量后端测试 `cargo test` 与前端测试 `npm run test`
- [x] 10.2 启动应用，验证：tab 切换、趋势图三种粒度与 resize 自适应、太阳图层级与伪子节点、流量表周期导航、空数据状态
- [x] 10.3 运行 `npm run build` 确认类型检查与构建通过

## 11. 太阳图改为可下钻动态旭日图

- [x] 11.1 修改 CategorySunburst.vue：开启 animation，series 增加 `nodeClick: 'zoomToNode'`，标签改为扇区内嵌（`position: 'inside'`、`rotate: 'radial'`），移除外部 labelLine
- [x] 11.2 启动应用验证：点击扇区动画下钻、点击中心返回上级、周期导航刷新后层级状态合理、空数据状态不受影响；运行 `npm run test` 与 `npm run build` 确认通过
