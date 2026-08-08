# Proposal: cashflow-account-tx-filter

## Why

现金流量表下方的收支明细列表目前只是静态展示。用户看到「餐饮 ¥500」时，想看这 500 块具体是哪些交易，只能手动切到交易页面、手动设置日期范围和账户筛选，操作链路长且容易设错（尤其父账户需要手动勾选全部子账户）。

## What Changes

- `CashFlowDetailList` 的明细行变为可点击（hover/cursor 提示）。
- 点击某账户行后：
  1. 以「自身 + 全部后代账户 ID」组装账户筛选（对齐现金流量表的聚合口径，方案 A，纯前端展开，不改后端筛选语义）；
  2. 以现金流量表当前周期的 `period_start`/`period_end` 设置日期范围；
  3. **整体替换**交易页面当前筛选（不与已有筛选叠加）；
  4. 通过 `useWheelScroll.spinTo` 将交易面板转回可视中心（环形布局下宽屏/窄屏/移动端统一处理）。
- 旭日图点击语义（下钻）不变；仅明细列表行新增点击行为。
- 无后端改动、无 API 改动。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `cash-flow-report`: 新增需求——现金流量表明细行可点击，跳转交易页面并设置周期 + 账户子树筛选。

## Impact

- **accounting-web**：`CashFlowDetailList.vue`（行点击事件）、`CashFlowPanel.vue`（点击处理：组装 filter、setFilter、spinTo）、新增「账户子树展开」纯函数及单测。
- **不改动**：后端所有 crate、`TxFilters` 结构、交易筛选 API、旭日图组件。
