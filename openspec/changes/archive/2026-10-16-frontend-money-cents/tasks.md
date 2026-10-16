# Tasks: frontend-money-cents

## 1. 核心换算 `money.ts`

- [x] 1.1 新建 `accounting-web/src/utils/money.ts`:`toCents(value, precision = 2)`(小数点拆分、右补零、超长四舍五入、非法输入抛 `Error`,不经浮点)与 `fromCents(cents, precision = 2)`(整除/取余拼接定长串);接口注释标注整数分语义。验证:`npx vue-tsc --noEmit` 无新增错误。
- [x] 1.2 新增 `accounting-web/src/utils/__tests__/money.spec.ts`:precision=0/2/8、负数金额、超长小数四舍五入(如 `"0.005"` precision 2)、空串/多小数点/非数字抛错、`fromCents(toCents(v)) === v` 往返。验证:`npx vitest run src/utils/__tests__/money.spec.ts` 全绿。

## 2. utils 计算层迁移

- [x] 2.1 重构 `utils/sunburst.ts`:累加、父子差额 `own`、降序排序、1% 过滤(`childCents * 100 > parentCents`)、判零(`!== 0`)全部 cents 化,删除 `EPSILON`;`SunburstNode.value/total` 注释标 cents。同步更新 `sunburst.spec.ts`:夹具金额改 cents 域断言,「ignores floating point residue」用例重写为整数精确语义(子项和恰等总额 → 无伪子节点)。验证:`npx vitest run src/utils/__tests__/sunburst.spec.ts` 全绿。
- [x] 2.2 重构 `utils/cashFlowList.ts`:兄弟排序比较 cents 化,`DetailRow.amount` 语义改整数分并注释(ratio 仍为浮点,由 cents 输入相除);`cashFlowList.spec.ts` 断言如涉中间值则同步。验证:`npx vitest run src/utils/__tests__/cashFlowList.spec.ts` 全绿。
- [x] 2.3 重构 `utils/amount.ts`:`formatCalendarAmount` 阈值与缩写在 cents 域判定(签名字符串不变,未达阈值透传后端原文——后端 Decimal 除法归约 scale 可变,fromCents 还原会补零改口径),非法输入随 toCents 抛错;更新 `amount.spec.ts`(补 scale 透传与抛错用例)。验证:`npx vitest run src/utils/__tests__/amount.spec.ts` 全绿。

## 3. 视图与组件迁移

- [x] 3.1 重构 `views/BudgetView.vue`:`totals()` 累加、`isOverspent`、`ringAmount` 改 cents 域,环内差额经 `fromCents` + 既有去尾零口径展示(`"150"` 不变);明细直显的后端字符串不动。验证:`npx vitest run src/views/__tests__/BudgetView.spec.ts` 全绿且断言未放宽。
- [x] 3.2 重构 `views/SavingPlanView.vue`:分配口径缺口 `target − allocated`、`occupiedWidth` 比较改 cents 输入(缺口展示经 `fromCents` → `formatDecimal` 保持去尾零);satisfaction/percentage 比率路径不动。验证:`npx vitest run src/views/__tests__/SavingPlanView.spec.ts` 全绿且断言未放宽。
- [x] 3.3 重构 `components/NetWorthTrendChart.vue`:净资产 `assets − liabilities` cents 化;喂给 ECharts 的序列数据在 `render()` 边界经 `fromCents` 转回元单位(`yLabel` 万/k 阈值与轴刻度语义不变)。tooltip 维持后端原文字符串上的 Decimal.js 精确运算(原任务字面"改由 fromCents 结果构造"系无谓往返,与 design D1 矛盾,按 D1 修订)。验证:`npx vue-tsc --noEmit` 通过;手动核对并入 4.3。
- [x] 3.4 重构 `components/CalendarGrid.vue`:日支出/收入零值判断改 `toCents(...) !== 0`;缩写调用按修订后的 D5 仍传后端原文字符串(formatCalendarAmount 内部 cents 判定)。验证:`npx vitest run src/views/__tests__/CalendarViewRefresh.spec.ts` 与相关组件测试全绿。
- [x] 3.5 调整渲染边界 `components/CategorySunburst.vue` 与 `components/CashFlowDetailList.vue`:tooltip/行金额以 `fromCents(cents)` 结果构造既有 `formatAmount`/Decimal 展示,1% 过滤与排序逻辑随 2.1 语义对齐。验证:`npx vitest run src/components/__tests__/CashFlowPanel.spec.ts src/components/__tests__/CashFlowDetailList.spec.ts` 全绿。

## 4. 集成验收

- [x] 4.1 复查 `accounting-web/src` 剩余 `Number(` 调用:均为非金额转换(满意度百分比、ID)或受信整数字段,无误伤。验证:grep 输出逐条核对并记录结论。
- [x] 4.2 全量测试。验证:`npm test`(vitest)全绿,含 `money.spec.ts`。
- [ ] 4.3 手动核对四处合计与明细一致:预算汇总/环形差额、旭日图层级与 tooltip、现金流详情列表、净值曲线。验证:`npm run dev` 逐页核对,无展示口径变化(千分位、去尾零、万/k 缩写、`"150"` 口径)。(用户决定先归档,目验后续进行)

## Workflow follow-up

- `openspec validate frontend-money-cents` 通过后,按项目约定 `openspec archive frontend-money-cents --skip-specs` 归档。
- 归档后复核归档变更的 design.md 已入库为决策来源。
