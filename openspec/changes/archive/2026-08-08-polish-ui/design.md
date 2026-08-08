# Design: polish-ui

## Context

`accounting-web` 无 UI 组件库，控件全部手写，全局样式仅 `src/style.css`(57 行，8 个 CSS 变量）。已确认的问题根因：

- **文本不居中**:`style.css` 只给 `button` 继承字体；输入控件用 UA 默认字体 + 继承 `:root` 的 `line-height: 1.5`，而按钮普遍 `line-height: 1`，同行时高度与基线错位。
- **抽屉滚动条闪现**:`ConfigPanel`/`.drawer` 从 `translateY(100%)` 滑入，包含块是视口（`ResponsiveShell` 未设 `position`),`#app` 的 `overflow: hidden` 裁剪不到它 → 探出视口产生纵向滚动条 → 视口变窄而 `#app { width: 100vw }` → 横向滚动条跟着出现。`AccountDrawer` 同模式。`TransactionFilterDrawer` 向下落入不受影响。
- **`--text-body` 未定义**:`TransactionFilterDrawer`、`DateRangePicker` 共 7 处引用，`style.css` 只有 `--text`。
- **硬编码颜色**:`AssetsView` 激活 tab `#4ade80`（绿） vs 全局 `--accent`（紫）;`TransactionCard`/`AccountDrawer`/`CalendarGrid`/`CashFlowDetailList`/`BalanceSheetPanel`/`TransactionView` hero 渐变等十余处字面量。
- **原生弹窗**:`TransactionView:96`、`CalendarView:69`、`AccountDrawer:211`、`TransactionFormOverlay:174` 共 4 处 `confirm()`/`alert()`。
- **重复输入框样式**:`.inline-input`/`.field-input`/`.input`/`.keyword-input`/`member-select` 等在 5+ 组件各写一遍。

## Goals / Non-Goals

**Goals:**

- 一次全局 CSS 治理，建立"样式地基"：变量完整、输入控件排版统一、抽屉动画无闪烁。
- 观感细节收敛：主色一致、原生弹窗替换、字体梯度有限、空状态齐全。

**Non-Goals:**

- 不引入 UI 组件库（保持手写体系）。
- 不做暗色主题、响应式重构等更大改造。
- 不改交互逻辑与数据流（由 fix-data-refresh 覆盖）。

## Decisions

### D1: 输入控件用全局 reset + 共享 class

在 `style.css` 加 `input, textarea, select { font: inherit; line-height: normal }`（或固定行高），并将重复的输入框样式抽成一个全局 class（如 `.field-input`）供 5+ 个组件复用；同行按钮对齐同一高度基准。删除 `AccountDrawer.vue` input 上无效的 `display:flex`。选全局 class 而非新组件：改动面最小，不动模板结构。

### D2: 滚动条闪现用最小修复

给 `.drawer-container`(ConfigPanel 与 AccountDrawer）加 `overflow: hidden`，使滑入中的抽屉被自身容器裁剪；同时将 `#app` 的 `width: 100vw` 改为 `width: 100%` 消除横向条根因。不引入 `position: relative` 到 shell 的方案——会改变既有定位上下文，风险更大。

### D3: 变量补齐与颜色收敛

在 `style.css` 定义 `--text-body`（与 `--text` 的关系明确）并补语义变量（如 `--color-income`/`--color-expense`/`--color-warning`)，把硬编码字面量逐一替换；`AssetsView` 激活 tab 改用 `--accent`。收支红绿保留为语义变量，不并入主色。

### D4: 原生弹窗替换为轻量应用内组件

实现一个极简的确认/提示组件（或复用已有 toast 体系扩展确认能力），替换 4 处 `confirm()`/`alert()`。不引入第三方依赖。

### D5: vue-router 空挂载

`router/index.ts` routes 为空且页面切换实际由 `ResponsiveShell` 轮播实现——移除 vue-router 依赖与挂载（死代码清理）；若后续规划路由再恢复。

## Risks / Trade-offs

- [全局 reset 改变某些输入框既有观感] → 逐视图目视回归（交易/资产/账户/日历/计划 5 个 pane + 各抽屉）。
- [颜色收敛时误改语义色] → 收敛前先列出字面量清单并逐一归类（主色/语义色/中性色），语义色只换变量不换色值。
- [确认组件与既有 toast 风格不统一] → 确认组件复用 toast 的样式变量与动画约定。
