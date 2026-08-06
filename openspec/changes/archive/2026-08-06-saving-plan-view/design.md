# Design: saving-plan-view

## Context

攒钱计划后端已完整（含全局分配满足率），预算执行情况有 store 无 UI。前端关键现状（已勘探）：

- **无组件库**：Vue 3 + 手写组件，暗色主题 + CSS 变量（`--bg/--card-bg/--accent` 等）。
- **页面注册三处**：`useResponsiveLayout.ts` 的 `paneNames` + `ResponsiveShell.vue` 的 `componentMap` + locales 的 `nav.*`（vue-router 路由表为空，不使用）。
- **BudgetView 模式**：卡片列表 + 覆盖式抽屉表单（创建/编辑共用）+ `panelActionKey` 标题栏新建按钮 + 抽屉内 `.picker-portal`。
- **`AccountPicker`/`AccountPickerOverlay`**：单选，按账户类型分组展示，无类型过滤 prop。
- **预算表单硬编码 `commodity_id=1`**，无币种选择器先例；`fetchCommodities` 存在。
- **测试**：vitest + happy-dom，就近 `__tests__/`，预算页无测试，有 `api/__tests__/client.spec.ts` 与视图测试先例。

## Goals / Non-Goals

**Goals:**

- SavingPlanView 完整页面（列表环形 + 展开分配明细 + 抽屉表单）。
- BudgetView 执行情况 UI（环形 + 展开明细）与表单扩展（一次性/deadline/支出账户限制）。
- 两个批量状态端点，前端零 N+1。
- AccountPicker 类型过滤 prop。

**Non-Goals:**

- 币种选择器（攒钱计划与预算表单都硬编码 CNY，与现状一致）。
- 预算页的统计图表（趋势/占比等）。
- 后端任何既有端点的行为变更。
- e2e 测试框架引入。

## Decisions

### D1: 批量状态端点 ×2

- `GET /api/saving-plans/statuses?date=` → `[SavingPlanStatusDto]`，service 直接调既有 `list_saving_plan_statuses(date)`，**按检查点升序返回**（分配顺序即展示顺序，最急的计划在最上）。
- `GET /api/budgets/statuses?date=` → `[BudgetStatusDto]`，service 新增 `list_budget_statuses(date)`（循环 `get_budget_status`），按预算 id 序返回。
- 备选：前端 N+1（拒绝，列表随请求逐个跳动）;扩展现有 list 端点加 `?with_status`（拒绝，状态是重计算，独立端点语义更清晰）。

### D2: 页面结构镜像 BudgetView

SavingPlanView 复用 BudgetView 的全部结构惯例：`.budget-card` 风格卡片、覆盖式抽屉、`panelActionKey` 新建按钮、`.picker-portal`。状态详情用**卡片内联展开**（点击卡片切换），不开第二抽屉——详情是只读信息，层级比表单浅。

### D2a: 单面板 + 选项卡（范围调整）

预算与攒钱计划合并为一个「计划」面板（`plan`）：新建轻量 `PlansView.vue` 包装组件，顶部选项卡（镜像 ConfigPanel 的 `.tab-bar/.tab-btn` 样式）+ `v-if` 切换挂载 BudgetView/SavingPlanView。用 `v-if` 而非 `v-show`：两个子视图都通过 `panelActionKey` 设置标题栏按钮，同时挂载会互相覆盖；`v-if` 保证任一时刻只有活动视图设置按钮（切换时新挂载视图覆盖旧按钮，既有惯例下无需 unmount 清理）。代价是切换选项卡时子视图状态（展开态/滚动）重置，可接受。选项卡标签复用 `nav.budget`/`nav.savingPlan` 词条，面板名用新词条 `nav.plan`（计划/Plans）。

### D3: 环形进度组件（新公共组件 `ProgressRing.vue`）

SVG 圆环：灰底环（`--border`）+ 彩色弧（stroke-dasharray），中心显示百分比或金额。参数：percentage、color、中心文本。颜色规则：

- 攒钱计划：satisfaction = 100 → 绿（`#2ecc71`）；< 100 → 黄（`#f1c40f`）；已失效 → 灰。环内显示百分比。
- 预算：未超支 → 绿弧显示已用百分比，环内显示剩余；超支 → 红（`#e74c3c`，与现有超支色一致），环内显示「超支 + 金额」（镜像 resources/预算.jpg 参考图）。

### D4: 攒钱计划双口径展示

`met`（账面余额口径）与 `satisfaction`（全局分配口径）可能不一致（账面够但轮到时不够）。卡片环形显示 satisfaction；展开详情同时给出 current_balance/gap/met 与 allocated/satisfaction，账户明细逐行显示 `balance / occupied_by_earlier / allocated`（每账户一条余额条：底色余额、叠加被占用部分）。文案用 i18n 区分「账面」与「分配」。

### D5: 表单设计

- 两个表单共用模式：period `<select>` 增加「一次性」(once) 选项（值为空语义）；选一次性时 deadline 输入框**必填**（一次性无截止日期的永久计划允许留空——攒钱计划支持，预算同理允许留空）；选周期时 deadline 可选。
- deadline 用原生 `<input type="date">`。
- 攒钱计划账户多选：多行 AccountPicker（每行一个账户 + 删除按钮，底部「添加账户」按钮），与 budget limits 行模式一致但无金额输入；AccountPicker 新增 `accountType` prop（`'asset' | 'expense' | undefined`），Overlay 按 prop 过滤分组——攒钱计划表单传 asset，预算表单传 expense（配合后端校验，表单层就不让选错）。
- target_amount 用 number input；commodity_id 硬编码 1。
- 编辑时预填现有值（含 period=null → 一次性、deadline）。

### D6: store 与 API 层

- `stores/savingPlan.ts` 镜像 `stores/budget.ts`（plans/statuses/currentStatus/loading/error + CRUD 动作），列表数据直接用批量端点一次取回（store 里存 `statuses: SavingPlanStatusDto[]`，卡片渲染不再逐计划请求）。
- `stores/budget.ts` 增加 `statuses` 状态与 `loadStatuses` 动作。
- `api/client.ts` 按既有样板加 `fetchSavingPlans/fetchSavingPlanStatuses/createSavingPlan/updateSavingPlan/deleteSavingPlan/fetchSavingPlanStatus`（单个 status 保留给详情展开用）与 `fetchBudgetStatuses`。

## Risks / Trade-offs

- [手写 SVG 环形组件的视觉质量] → 参数化简单几何（圆周长 dasharray），暗色主题下风险低；参考图风格明确。
- [BudgetView 单文件已 454 行，加执行情况会更大] → 接受（项目惯例就是单文件视图）；若超 700 行可把执行明细抽为 `BudgetStatusDetail.vue`，任务中视情况决定并在报告注明。
- [批量预算状态是周期口径，月切换不存在] → 状态始终按「今天」所在周期计算（与 CLI 一致）；日期切换器不在本次范围。
- [一次性+永久（period/deadline 皆空）的预算/计划在表单中的引导] → 表单不主动引导，留空即可，spec 场景锁定。

## Migration Plan

纯增量：后端新端点 + 前端新页面/组件，无数据迁移，无既有端点变更。前端 `npm run build`（vue-tsc）通过即类型安全。

## Open Questions

- 无。
