# Proposal: frontend-money-cents

## Why

后端以 Decimal 字符串返回金额,前端却在大量位置用 `Number()` 转浮点后做累加、相减、比较与排序,存在经典浮点精度隐患(如 `0.1 + 0.2 !== 0.3`);`sunburst.ts` 已被迫用 `EPSILON = 1e-6` 容差补丁掩盖浮点残差。趁 `cargo check` 基线干净、相关视图测试完备时整型化,消除整类缺陷。

## What Changes

- 新增 `accounting-web/src/utils/money.ts`,提供纯函数 `toCents(value, precision = 2)`(字符串 → 最小单位整数,不经过浮点)与 `fromCents(cents, precision = 2)`(整数 → 定长小数字符串,纯整数除法/取余)。
- 金额的累加、相减、比较、排序计算点全部迁入整型最小单位域,共 4 个计算文件:
  - `utils/sunburst.ts`:金额累加、父子差额 `own`、降序排序、1% 可见性过滤、差额判零。
  - `utils/cashFlowList.ts`:金额降序排序比较。
  - `utils/amount.ts`:`formatCalendarAmount` 改为在 cents 域判零与缩写。
- 视图/组件计算点迁入整型域:
  - `views/BudgetView.vue`:`totals()` 限额/实际累加,超支判断、差额展示。
  - `views/SavingPlanView.vue`:分配口径缺口 `target − allocated`、余额占用比较。
  - `components/NetWorthTrendChart.vue`:净资产 `assets − liabilities`(喂给 ECharts 的序列数据在渲染边界转回元单位)。
  - `components/CalendarGrid.vue`:日金额零值判断。
  - 渲染边界 `components/CategorySunburst.vue`(tooltip 金额)与 `components/CashFlowDetailList.vue`(行金额)改用 cents → 展示串。
- 计算结果仅在渲染边界经 `fromCents` 恢复展示串;千分位、去尾零、万/k 缩写等展示行为保持不变。
- 展示域之外的 `Number()` 转换(满意度百分比、ID 转换等)不在改造范围。
- 测试:新增 `money.ts` 单测(precision 0/2/8、负数、超长小数四舍五入、非法输入);`sunburst.spec.ts` 数值随 cents 域更新、"浮点残差容差"用例语义改写为整数精确相等;视图测试断言的渲染字符串预期不变。

不改变任何用户可感知的展示行为;无后端、API、依赖变化。

## Capabilities

### New Capabilities

（无——本变更为纯内部重构,计算域整型化对合法输入不可观测,现有视图级规格的行为事实不变,故 `.openspec.yaml` 声明 `skip_specs: true`。）

### Modified Capabilities

（无——`budget-view`、`saving-plan-view`、`calendar-view`、`assets-visual-reports` 等现有规格描述的展示行为全部保持。）

## Impact

- 代码:`accounting-web/src` 下新增 1 个工具文件,修改 3 个 utils、4 个视图/组件、2 个渲染边界组件,共约 10 个文件。
- 测试:新增 `utils/__tests__/money.spec.ts`;更新 `sunburst.spec.ts`;`BudgetView.spec.ts`、`SavingPlanView.spec.ts`、`cashFlowList.spec.ts`、`amount.spec.ts`、`decimal.spec.ts` 预期保持绿色(如断言失稳则同步微调)。
- 已知限制:DTO 不携带 commodity precision,调用点按默认 precision=2(与现有数据一致,`accounting-sql/src/schema.rs` 中 `commodities.precision DEFAULT 2`);未来出现非 2 精度币种时需在调用点显式传参(函数已参数化,单测覆盖)。
- 验收:`npm test`(vitest)全绿;手动核对预算汇总、旭日图、现金流、净值曲线四处合计与明细一致。
