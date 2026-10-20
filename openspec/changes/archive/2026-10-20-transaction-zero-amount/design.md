# Design: transaction-zero-amount

## Context

动机见 proposal.md — Why。相关现状与约束：

- 折叠态卡片金额由 `accounting-web/src/components/TransactionCard.vue` 的 `computeAmount()` + `displayAmount` + 样式类决定；按日分组头的收/支汇总由 `accounting-web/src/components/TransactionList.vue` 里**另一份复制的** `computeAmount()` 提供。
- 两份 `computeAmount()` 都含同一段遗留回退：资产分录净额为 `0` 时，改取「正值资产分录之和」。这是 `is_transfer` 字段出现前兼容转账的手段——归档 `transaction-transfer-and-delete` 的 design D5 明确记载「转账（资产分录之和为零、回退取正值之和）因此与收入同形」；同一变更还把「日期分组头仍会把转账计入「收」」列为已知限制、留作后续变更。
- 转账现已由后端实时判定的 `is_transfer` 标识（见 `transaction-list-ui`「转账交易识别」），前端走独立渲染分支（蓝色转出额 + 可选资产变动额），卡片侧回退只剩误伤非转账交易。
- 金额样式现状：`.amount` 默认红（`--color-expense`），`amount-positive` 绿，`amount-transfer` 蓝，退款金额已用 `--text-muted`（低对比度）。活规格 `transaction-list-ui`「金额计算规则」正文只写「普通交易：资产账户分录金额之和」，未包含该回退——即代码本就偏离规格。
- 项目约定：组件颜色取自主题变量（见 `ui-polish`「主题变量完整且颜色来源唯一」），低对比度语义色为 `--text-muted`。

## Goals / Non-Goals

**Goals:**
- 非转账交易金额严格等于资产分录金额之和；净额为 `0` 时以低对比度渲染 `0.00`。
- 交易卡片金额与按日分组收/支汇总采用同一口径，不再分叉。

**Non-Goals:**
- 不改转账卡片渲染与转账判定（`is_transfer`）。
- 不改月收支汇总（`TransactionView` 直接对 expense/income 分录求和）与日历格子统计（后端 stats）。
- 不改展开态分录行的金额样式（默认为 `--color-info` 的那套）。

## Decisions

### D1 删除零净额回退，非转账金额直接取资产分录金额之和
- 理由：回退的唯一历史用途是兼容 `is_transfer` 之前的转账；该场景已由后端标识接管，保留它只会让「资产净额为 0 的非转账交易」显示出一个凭空的正数。
- **备选**：保留回退，仅在「净额为 0 且无正值分录」时改色——否决：覆盖不到「有正值分录」时误显绿色的主场景。
- **备选**：后端在 DTO 里直接返回展示金额——否决：金额是纯展示推导，前端已有 postings 与 `is_transfer`，无谓扩契约。

### D2 零金额渲染为低对比度 `0.00`
- 用 `--text-muted` 渲染 `0.00`，不带正负号。理由：零没有方向，不应占用收入绿/支出红语义色；与退款金额的 muted 先例保持一致。
- **备选**：保留红色 `0.00`——否决：红色是支出语义，误导。
- **备选**：隐藏金额——否决：违背 `transaction-list-ui`「折叠态金额对每笔交易必现」。

### D3 卡片与按日汇总同步改口径
- `TransactionList.vue` 的那份 `computeAmount()` 同步去掉回退。理由：两处口径若分叉，卡片显示 `0.00` 而日头仍把它计为收入，自相矛盾。
- 连带影响（有意为之）：纯转账的资产分录净额也为 `0`，故按日收/支汇总将**不再**把纯转账计入当日收入——这正好消解归档 `transaction-transfer-and-delete` 留下的已知限制，规格中以独立 Scenario 显式锁定。
- **备选**：只改卡片、日汇总不动——否决：留下不一致，且当日汇总会把零净额交易与纯转账虚增为收入。
- **备选**：仅为非转账去回退、纯转账沿用旧回退——否决：等于在两处复制同一特例，反而加深漂移。

### D4 抽取共享纯函数，消除两份复制
- 把「非转账金额 = 资产分录金额之和」抽为 `accounting-web/src/utils/amount.ts` 中的共享纯函数，供 `TransactionCard` 与 `TransactionList` 共用。
- 理由：正是这份复制让旧口径同时潜伏在两处；本轮两处都要改，顺手收敛可避免下一次漂移。
- **备选**：保持两份复制、各改各的——否决：已被证明易漂移，且校验成本翻倍。

## Risks / Trade-offs

- [回归：是否有测试或代码依赖旧回退] → 已检索前端测试，转账用例均显式设 `is_transfer: true`，无断言依赖零净额回退；实现时补零金额用例锁定新口径。
- [按日汇总口径变化超出「零净额非转账」] → 纯转账不再计入当日收/支，属预期修复；在规格中独立成 Scenario，评审可见。
- [深色主题下 muted 对比度偏低] → 零金额本身是弱信息，且沿用退款金额同一变量，保持全局一致。

## Migration Plan

无数据迁移、无 schema/API 契约变更。纯前端渲染口径调整，随前端发布即时生效；回滚即还原组件与共享函数。

## Open Questions

- 展开态分录行中 `0.00` 的样式（当前为 `--color-info` 蓝）是否也要 muted——本轮未纳入，若需要另开变更。
