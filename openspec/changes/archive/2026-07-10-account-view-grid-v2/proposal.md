## Why

当前账户页面的展开子账户采用高亮行形式，视觉上层级区分不明显，且切换选中时折叠条件过于宽松，容易误折叠用户仍在浏览的子树。本次改造旨在提升账户视图的层级可读性与交互精确性。

## What Changes

- 将账户视图中展开的账户树结构从“高亮行”改为“单一卡片网格”，每行固定列数、左对齐、右侧留空，所有深度卡片等宽对齐。
- 采用 HSV 深度渐变的包裹式纸片叠放高亮：卡片颜色保持统一，展开区背景和高亮框随深度在 HSV 空间变化，多层半透明背景相互叠加形成纵深。
- 调整折叠条件：仅当新选中卡片不在当前 `expandedPath` 上时才折叠；在路径上时只切换选中态，保持子树展开。
- 更新相关组件与测试：`AccountGrid`、`AccountRowGroup`、`AccountsView` 及其单元测试、端到端验证。

## Capabilities

### New Capabilities

- 无新增能力。

### Modified Capabilities

- `account-page`: 修改账户卡片网格的视觉呈现方式（从高亮行改为等宽卡片网格 + 包裹式纸片叠放高亮），并收紧选中切换时的折叠条件。

## Impact

- 前端：`accounting-web/src/components/layout/AccountGrid.vue`、`AccountRowGroup.vue`、`AccountCard.vue`。
- 视图：`accounting-web/src/views/AccountsView.vue` 的 `expandedPath` 与折叠逻辑。
- 测试：`AccountGrid.spec.ts`、`AccountRowGroup.spec.ts`、`AccountsView.spec.ts` 需要补充视觉与交互断言。
- 无 API 或后端变更。
