# Proposal: transaction-zero-amount

## Why

非转账交易若资产分录净额为零，前端金额计算会触发「资产净额为零时回退取正值资产分录之和」的遗留逻辑，把这个凭空得来的正数渲染成绿色收入样式；净额为零又无正分录时，则渲染成红色 `0.00`。两者都与「普通交易金额 = 资产分录金额之和」这一实际口径不符，会误导用户对资产变动的判断。该回退原本是在 `is_transfer` 字段出现前用来兼容转账（转账的资产分录净额恒为零），如今转账已由 `is_transfer` 单独分支渲染，回退只剩误伤非转账交易的副作用。

## What Changes

- 非转账交易金额严格取资产分录金额之和，去掉「资产净额为零时回退取正值之和」的遗留回退。
- 非转账交易金额为 `0` 时，金额以低对比度（`--text-muted`）渲染为 `0.00`，不带正负号；不再套用红色支出样式。
- 该口径同时作用于交易卡片金额与按日分组的收/支汇总，保证两处一致（金额为 0 的交易不再计入当日收入或支出）。
- 转账交易（`is_transfer`）金额渲染保持不变（蓝色转出额 + 可选资产变动额）。

## Capabilities

### New Capabilities

### Modified Capabilities

- `transaction-list-ui`: 「金额计算规则」——明确普通交易金额严格为资产分录金额之和（取消零净额回退取正值）；新增零金额渲染规则（非转账、金额为 `0` 时低对比度渲染 `0.00`），并明确按日分组收/支汇总采用同一口径。

## Impact

- 前端组件：`accounting-web/src/components/TransactionCard.vue`（`computeAmount`、`displayAmount`、金额样式类），`accounting-web/src/components/TransactionList.vue`（复制的 `computeAmount`，用于按日汇总）。
- 影响共用这两个组件的 `TransactionView` 与 `CalendarView`。
- 无后端、无 API、无 schema、无数据迁移。
- 测试：`TransactionCard.spec.ts` 增补零金额场景；如存在按日汇总的组件测试则同步增补。
