# Tasks: saving-plan-view

## 1. 后端：批量状态端点

- [x] 1.1 `accounting-service`：新增 `list_budget_statuses(date)`（循环 get_budget_status）；`list_saving_plan_statuses` 确认排序语义（参与分配的计划按 (检查点, plan_id) 升序在前，过期/永久在后），如不满足则调整
- [x] 1.2 `accounting-api`：`GET /api/saving-plans/statuses?date=` 与 `GET /api/budgets/statuses?date=` 两个端点 + DTO 复用 + router 注册
- [x] 1.3 后端集成测试：批量返回、排序（saving-plan 检查点序）、批量与单条口径一致、空数组、日期格式无效 400

## 2. 前端：数据层

- [x] 2.1 `types/api.ts`：SavingPlan 全套类型（SavingPlanDto/DetailDto/StatusDto/AccountAllocationDto/Create/UpdateRequest）+ Budget statuses 相关类型对齐后端
- [x] 2.2 `api/client.ts`：saving-plan 6 个函数 + `fetchBudgetStatuses`；`api/__tests__/client.spec.ts` 补用例
- [x] 2.3 `stores/savingPlan.ts`（镜像 budget.ts：plans/statuses/currentStatus + CRUD）；`stores/budget.ts` 增加 statuses/loadStatuses

## 3. 前端：公共组件

- [x] 3.1 `components/ProgressRing.vue`：SVG 环形进度（灰底环+彩色弧+中心文本），参数 percentage/color/中心内容
- [x] 3.2 `AccountPicker`/`AccountPickerOverlay`：新增 `accountType` prop（'asset'|'expense'|undefined），按类型过滤分组；既有调用点行为不变

## 4. 前端：SavingPlanView

- [x] 4.1 卡片列表：名称/周期或截止/目标 + 满足率环形（100 绿/<100 黄/失效灰+徽标）+ 空状态
- [x] 4.2 卡片内联展开：余额/缺口/met + allocated/satisfaction + 每账户分配明细（余额条叠加被占用）
- [x] 4.3 抽屉表单：名称/周期(含一次性)/deadline(date 输入)/目标金额/账户多行选择(限 Assets)；创建/编辑共用、删除带确认；commodity_id 硬编码 1

## 5. 前端：BudgetView 状态 UI 与表单扩展

- [x] 5.1 卡片加执行环形（未超支绿弧+剩余/超支红环+超支额）+ 已失效徽标；数据改走 `fetchBudgetStatuses`
- [x] 5.2 卡片内联展开：各账户 limit/actual/remaining/percentage 明细（超支红色）；一次性预算不显示周期区间
- [x] 5.3 表单：period 加一次性选项、deadline date 输入、限额账户选择限 Expenses；创建/编辑/删除流程回归

## 6. 注册、i18n 与验证

- [x] 6.1 页面注册三处：`useResponsiveLayout.ts` paneNames/paneLabels、`ResponsiveShell.vue` componentMap、locales `nav.savingPlan`
- [x] 6.2 locales zh-CN/en：`savingPlan.*` 命名空间 + budget 新增词条（已失效/超支/一次性等），i18n.spec 对齐校验通过
- [x] 6.3 前端测试：SavingPlanView 组件测试（列表/展开/表单）+ ProgressRing 测试 + AccountPicker 过滤测试
- [x] 6.4 全量验证：`npm test` 与 `npm run build`（vue-tsc）通过；`cargo test --workspace` 全绿
- [ ] 6.5 手工验收：dev server 起页面，计划面板选项卡切换、攒钱计划列表环形/展开明细/创建编辑删除全流程，预算页环形/超支红环/一次性表单

## 7. 合并面板（范围调整）

- [x] 7.1 新建 `PlansView.vue`：顶部选项卡（镜像 ConfigPanel `.tab-bar/.tab-btn`）+ `v-if` 切换 BudgetView/SavingPlanView，默认预算视图
- [x] 7.2 注册改为单面板：paneNames 用 `plan` 替换 `budget`/`savingPlan`，componentMap 指向 PlansView，locales 加 `nav.plan`（计划/Plans），选项卡标签复用 `nav.budget`/`nav.savingPlan`
- [x] 7.3 `PlansView.spec.ts`：默认显示预算、点击选项卡切换到攒钱计划（仅活动视图挂载、标题栏动作跟随切换）
- [x] 7.4 全量验证：`npm test` 全绿 + `npm run build` 通过
