# Design: cashflow-account-tx-filter

## Context

现金流量表（`CashFlowPanel`，位于 assets pane 的 Reports 页签）下方的 `CashFlowDetailList` 以树状缩进展示收支账户明细，金额为**聚合口径**（闭包表，父含子）。交易页面已有完整筛选机制：`txStore.setFilter(TxFilters)` 支持 from/to/accounts 等维度，设置后列表自动重载并显示「已筛选」徽章。环形布局下面板切换由 `useWheelScroll.spinTo(index)` 驱动（transaction pane 固定为 index 0）。

关键约束（探索阶段已确认）：后端交易筛选 `account_ids` 是**精确匹配**（`postings.account_id IN (...)`，不含后代），与现金流量表的聚合口径不一致。若直接传父账户 ID，点「餐饮 ¥500」可能筛出 0 笔交易。

## Goals / Non-Goals

**Goals:**

- 明细行可点击，一键跳转交易页面并筛出「当前周期 × 该账户子树」的交易。
- 跳转前后金额口径一致（聚合 = 子树并集），不产生「筛选坏了」的困惑。

**Non-Goals:**

- 不改后端筛选语义（不支持闭包表展开，留待真有第二处需要时再做）。
- 不改旭日图交互、不改交易筛选抽屉、不新增筛选维度（如收支方向）。
- 不做「从交易页面返回现金流量表」的回跳。

## Decisions

### D1: 前端展开账户子树（方案 A），不改后端

点击行时，用 account store 中已加载的账户树，把被点账户展开为「自身 + 全部后代 ID」列表，作为 `TxFilters.accounts`。现有 `IN (...)` 多值 OR 语义恰好等于「涉及该子树任意账户」，与聚合口径对齐。

- 备选 B（后端闭包表展开）：更通用但动共享筛选语义，影响面大，YAGNI 否决。
- 数据来源：优先复用前端已有的账户列表（account store）；展开函数为纯函数 `expandSubtree(accounts, rootId): number[]`，独立单测。

### D2: 整体替换筛选，不叠加

`setFilter({ from: period_start, to: period_end, accounts: [子树] })` 替换现有 `activeFilter`。钻取语义是「看这笔账」，与抽屉里残留的成员/标签/渠道筛选叠加会产生费解结果。members/tags/channels/keyword/reimbursable 均置空。

### D3: 跳转用 spinTo(0)，不引入路由

环形布局下宽屏（交易面板可能已可见）与窄屏/移动端（需要转动）统一由 `spinTo` 处理动画；`useWheelScroll` 是模块级共享状态，`CashFlowPanel` 直接调用即可。跳转后 TransactionView 因 `activeFilter` 变化自动重载，并显示「已筛选」徽章，用户可经筛选抽屉查看/清除条件。

### D4: 点击事件沿组件链上抛，状态归 CashFlowPanel

`CashFlowDetailList` 增加 `@click` 行事件（emit `select(accountId)`），保持自身无状态；`CashFlowPanel` 持有周期/日期状态，负责组装 filter 与跳转。行加 `cursor: pointer` 与 hover 底色提示可点击。

## Risks / Trade-offs

- [账户列表未加载时展开结果只有自身] → account store 在应用启动时已加载（既有行为）；防御性处理：展开结果至少包含被点账户自身，退化为精确匹配也不会报错。
- [账户树很深时 accounts 参数很长] → 家庭账簿层级有限（几十 ID 上限），URL 长度无压力。
- [period_start/period_end 依赖 cashFlow DTO 已加载] → 明细列表仅在 `reportStore.cashFlow` 非空时渲染，点击时周期字段必然存在。

## Migration Plan

纯前端增量改动，随下个版本直接发布；无数据迁移，回滚即 revert。

## Open Questions

（无）
