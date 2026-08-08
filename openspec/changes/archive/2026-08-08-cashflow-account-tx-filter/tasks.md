# Tasks: cashflow-account-tx-filter

## 1. 账户子树展开纯函数

- [x] 1.1 在 `accounting-web/src/utils/` 新增 `expandSubtree(accounts, rootId): number[]`：返回 rootId 自身 + 全部后代账户 ID；账户树数据来自 account store 已有结构；防御性处理：树中找不到 rootId 时至少返回 [rootId]
- [x] 1.2 单测：多级树展开、叶子账户（仅自身）、未知 ID 兜底、不含兄弟/父账户

## 2. 明细行可点击

- [x] 2.1 `CashFlowDetailList.vue`：行增加 `@click` 并 emit `select(accountId)`；行样式加 `cursor: pointer` 与 hover 底色（沿用现有 CSS 变量风格）
- [x] 2.2 组件测试：点击行 emit 正确 accountId

## 3. 点击处理与跳转（CashFlowPanel）

- [x] 3.1 `CashFlowPanel.vue` 监听 `select`：组装 `{ from: cashFlow.period_start, to: cashFlow.period_end, accounts: expandSubtree(...), members: [], tags: [], channels: [] }`，调用 `txStore.setFilter()`（整体替换），再 `useWheelScroll().spinTo(0)` 转动至交易面板
- [x] 3.2 测试：点击后 setFilter 参数正确（周期 + 子树、其余维度清空）、spinTo 被调用

## 4. 验证与收尾

- [x] 4.1 `npm run test` / `lint` / `build` 全绿
- [x] 4.2 手工验证：现金流量表点击「餐饮」→ 交易面板转回、仅显示该周期该子树交易、「已筛选」徽章出现；打开筛选抽屉确认条件符合预期
