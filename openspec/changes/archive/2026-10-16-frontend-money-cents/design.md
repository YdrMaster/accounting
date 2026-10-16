# Design: frontend-money-cents

## Context

后端一律以 Decimal 字符串返回金额;前端 `accounting-web` 在预算汇总、攒钱计划缺口、旭日图累加、现金流排序、净值差值、日历零值判断等处先 `Number()` 转浮点再运算。`utils/sunburst.ts` 已有 `EPSILON = 1e-6` 容差补丁,说明浮点残差是现实问题而非理论担忧。展示层已有两套格式化:`utils/decimal.ts`(去尾零)与 `utils/amount.ts`(千分位 + 日历缩写,内部 Decimal.js)。commodity precision 在 `accounting-sql/src/schema.rs` 默认 2,当前全部数据 precision=2,且 DTO 不携带该字段。

## Goals / Non-Goals

**Goals:**

- 金额的累加、相减、比较、排序全部迁入最小单位整数域,消灭浮点残差与 `EPSILON` 容差补丁。
- 展示行为(千分位、去尾零、万/k 缩写、`"150"` 与 `"150.00"` 口径)逐字符不变,现有视图测试原样通过。
- 核心换算收敛为两个纯函数,可离线穷举单测。

**Non-Goals:**

- 不改后端、API、DTO(不补 precision 字段下发,见 D6)。
- 不动非金额的 `Number()` 转换(满意度百分比直显、ID 转换)。
- 不引入 vue-router、虚拟滚动等 OPTIMIZATIONS.md 其他分组项。
- 不重写既有 `formatAmount`(Decimal.js)千分位格式化,仅收敛其输入来源。

## Decisions

### D1: 新建 `utils/money.ts` 纯函数,不引入新依赖

`toCents(value: string, precision = 2): number` 与 `fromCents(cents: number, precision = 2): string` 为纯字符串/整数运算。
否决备选:用已在仓库中的 decimal.js 承担全部计算——否决原因是每个计算点都要构造 Decimal 对象,改动面与运行时开销更大,且换算核心本就只需十余行,纯函数零依赖、单测可穷举。decimal.js 保留在既有展示格式化(`formatAmount`)处不动。

### D2: 换算算法不经浮点

`toCents`:按小数点拆分,整数部分直接 `Number`(受信后端整数字段,长度远小于安全整数上限),小数部分按 precision 右补零、超长则看第 precision+1 位四舍五入进位到整数部分;符号随整数部分。`fromCents`:`Math.trunc(cents / 10**precision)` 与余数补零拼接,不出现 `cents / 100` 浮点除法。
否决备选:`Number(v) * 100` 再取整(0.1+0.2 类误差)与 `Math.round(cents) / 10**precision` 模板拼接(fromCents 经浮点除法,大数值再现精度问题)。

### D3: cents 域语义简化,删除 EPSILON

- 差额判零(日历零值等比较场景):`Number(x) !== 0` → `toCents(x) !== 0`(整数精确)。
- 伪子节点生成:`own > EPSILON` → `own > 0`——仅正差额生成伪子节点,忠实原行为(负差额属数据异常,原本静默忽略;若按 `!== 0` 生成负值伪子节点反而会破坏父环 = 子环之和)。
- 1% 可见性过滤:`value > parentTotal * 0.01` → `childCents * 100 > parentCents`(严格大于,保持「超过 1%」语义不变)。
- 排序:cents 直接相减比较。

否决备选:保留 EPSILON 作为防御——整数域不存在残差,容差只会错误吞掉真实的 0.01 分录差额。溢出边界:JS 安全整数 2^53 分 ≈ 907 万亿元,个人记账场景余量充足。

### D4: 视图模型携带 cents,渲染边界集中转回

`SunburstNode.value/total`、`DetailRow.amount` 的语义变为整数分,仅在最外层展示点(tooltip、行金额、环形差额文本)转换;`CategorySunburst.vue`、`CashFlowDetailList.vue` 的 `new Decimal(...)` 包装改为以 `fromCents` 结果构造。净值趋势图特殊:喂给 ECharts 的序列数据在 `render()` 边界经 `fromCents` 转回**元单位数值**,保住 `yLabel` 的万/k 阈值与轴自动缩放语义。
否决备选:整个图表域用 cents——需重定义 yLabel 阈值与轴刻度,tooltip 又要换算回去,两处口径不一致。

### D5: 展示格式化分工不变

`formatDecimal` 保留为通用小数格式化(百分比、satisfaction 等非金额路径原样);金额**计算结果**统一走 `fromCents` 得定长串,再按现状经 `formatDecimal` 去尾零(如分配口径缺口展示 `"150"`),复现既有口径。`formatCalendarAmount` **签名字符串不变**:阈值与缩写换算内部走 `toCents`(整数分域),未达阈值时原样透传后端字符串——实施期核实后端 `from_db_amount` 经 rust_decimal 除法归约,DTO 字符串 scale 可变(`"75"`、`"100.5"`),若经 `fromCents` 还原会补零成 `"75.00"` 改变展示口径;唯一例外是透传路径无需 cents 输出。
否决备选:让 `fromCents` 直接输出去尾零串——一个函数一种职责,定长串便于断言与千分位拼接。

### D6: precision 默认 2、参数化但不接线

DTO 不携带 commodity precision,现有数据恒为 2;调用点一律用默认值,不为传参新增查询链路。函数签名参数化,单测覆盖 precision=0/2/8。
否决备选:本次一并下发 precision 并在调用点接线——跨后端/DTO/service 改动,超出「纯前端重构」边界且当前收益为零;该链路属 OPTIMIZATIONS.md 分组 4.1 #8 范畴,另行处理。

### D7: 非法输入快速失败

`toCents` 对空串、缺整数/小数部分以外的畸形输入(多小数点、非数字字符)抛 `Error`。后端 Decimal 字符串受信,非法即程序 bug,应尽早暴露。
否决备选:静默回退 0 或原样透传——账目场景掩盖 bug 不可接受。

### D8: 测试策略

新增 `utils/__tests__/money.spec.ts`:precision 0/2/8、负数、超长小数四舍五入(如 `"0.005"` → precision 2 进位)、空串/非法输入抛错、`toCents`↔`fromCents` 往返。`sunburst.spec.ts` 数值改 cents 域,原「ignores floating point residue」用例按整数语义重写(子项和恰等于总额 → 无伪子节点,精确相等而非容差)。视图 spec(`BudgetView`/`SavingPlanView`)断言渲染字符串,预期原样通过;若失败视为展示行为回归,修实现而非改断言。

## Risks / Trade-offs

- [未来出现 precision ≠ 2 的 commodity] → `toCents`/`fromCents` 已参数化;调用点届时需显式传参,已在 proposal Impact 记录为已知限制。
- [`SunburstNode.value/total`、`DetailRow.amount` 语义从元变为分,后来者误解] → 接口注释标注 cents 单位;边界转换集中一处(D4)。
- [展示串口径回归(如 `"150"` 变 `"150.00"`)] → 视图测试锁定渲染字符串;D5 明确去尾零链路。
- [`grep Number(` 复查遗漏隐式浮点路径] → 验收时复查所有剩余 `Number(` 调用均为非金额转换或受信整数字段。
- [接受的小代价] cents 化后 `sunburst` 内 `EPSILON` 常量与部分浮点测试语义消亡,属于一次性认知成本,换来删除防御性容差。

## Migration Plan

纯前端重构,无数据迁移、无持久状态。实施顺序:`money.ts` + 单测 → utils 层(sunburst/cashFlowList/amount)→ 视图与组件层 → 渲染边界 → 测试更新 → `npm test` 全绿。回滚为单次 revert。

## Open Questions

无。
