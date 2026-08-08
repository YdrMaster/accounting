# Proposal: polish-ui

## Why

`accounting-web` 没有任何 UI 组件库，所有控件手写，但缺少全局样式规范：输入框未继承字体与行高导致文字上下不居中；底部抽屉滑入动画期间因包含块与 `100vw` 问题短暂出现双轴滚动条；CSS 变量 `--text-body` 未定义却被 7 处引用；主题色不统一（激活 tab 绿色 vs 全局紫色 `--accent`)、十几处硬编码颜色；4 处原生 `confirm()`/`alert()`；输入框样式在 5+ 个组件中重复定义；font-size 梯度碎片化。整体观感"不够精致"。

## What Changes

- **全局样式地基**:`input/textarea/select` 统一继承字体与合适行高；修复设置抽屉与账户抽屉滑入时的滚动条闪现；补齐或统一 `--text-body` 变量；收敛硬编码颜色到主题变量（含 AssetsView 激活 tab 与全局主色统一）。
- **共享控件样式**:抽取 5+ 处重复的输入框样式为共享 class，顺带解决文本垂直居中；删除 `AccountDrawer` 中 input 上无效的 flex 样式。
- **观感细节**:以应用内确认/提示组件替换原生 `confirm()`/`alert()`；统一 font-size 梯度；补齐 ConfigPanel 列表的空状态提示。
- **死代码清理**:移除空 routes 的 vue-router 挂载（或确认保留意图）。
- 纯前端样式与交互打磨，不改任何数据逻辑与 API。

## Capabilities

### New Capabilities

- `ui-polish`: 全局视觉一致性能力——输入控件排版规范、抽屉动画无滚动条闪现、主题变量完整且唯一来源、原生弹窗替换、字体梯度与空状态规范。

### Modified Capabilities

- `config-panel`: 设置抽屉打开时不得出现滚动条闪现；成员/渠道/标签列表需有空状态提示。

## Impact

- **代码**:`accounting-web/src/style.css`、`ConfigPanel.vue`、`AccountDrawer.vue`、`TransactionFormOverlay.vue`、`TransactionFilterDrawer.vue`、`DateRangePicker.vue`、`TransactionCard.vue`、`AssetsView.vue`、`CalendarGrid.vue`、`CashFlowDetailList.vue`、`BalanceSheetPanel.vue`、`TransactionView.vue`、`CalendarView.vue`、`router/index.ts`、`main.ts` 等。
- **API**：无变化。
- **风险**：全局 CSS 改动影响面广，需逐视图目视回归；颜色收敛需确认各处语义色（收/支红绿）与主题色的边界。
