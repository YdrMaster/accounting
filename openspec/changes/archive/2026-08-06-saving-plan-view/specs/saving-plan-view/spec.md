# saving-plan-view

## Purpose

攒钱计划页面——前端攒钱计划管理能力，以卡片列表展示各计划的目标金额与满足率（基于全局资金分配），支持展开查看状态详情（账面余额/缺口/达标与每账户分配明细），并通过抽屉表单支持计划的创建、编辑与删除，帮助用户规划与跟踪攒钱目标。

## ADDED Requirements

### Requirement: 页面注册与导航

预算页与攒钱计划页 SHALL 合并为同一个面板「计划」（paneNames、componentMap、nav i18n 三处注册，面板名为 `plan`），面板内通过选项卡在「预算」与「攒钱计划」两个视图间切换，默认显示预算视图。任一时刻只挂载活动视图（标题栏动作按钮由当前活动视图提供）。

#### Scenario: 切换到计划面板

- **WHEN** 用户在页面切换器中选择「计划」
- **THEN** 显示计划面板，默认展示预算视图，顶部有「预算」「攒钱计划」两个选项卡

#### Scenario: 选项卡切换到攒钱计划

- **WHEN** 用户点击「攒钱计划」选项卡
- **THEN** 面板内容切换为攒钱计划视图，标题栏动作按钮变为攒钱计划的新建按钮

#### Scenario: 选项卡切回预算

- **WHEN** 用户点击「预算」选项卡
- **THEN** 面板内容切换回预算视图，标题栏动作按钮变为预算的新建按钮

### Requirement: 计划列表展示

攒钱计划页 SHALL 从 GET /api/saving-plans/statuses 获取全部计划状态（按检查点升序），以卡片展示：名称、周期或截止日期、目标金额、满足率环形（ProgressRing）、状态徽标。环形颜色规则：satisfaction 为 100 显示绿色，小于 100 显示黄色，已失效计划显示灰色并带「已失效」徽标。`satisfaction` 显示为归一化百分比（如 75 而非 75.00）。

#### Scenario: 加载计划列表

- **WHEN** 用户切换到攒钱计划页
- **THEN** 系统调用批量状态端点，按检查点顺序展示计划卡片与满足率环形

#### Scenario: 共享账户计划满足率不同

- **WHEN** 计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,E} 目标 2000），A 3000、B 1000、E 500
- **THEN** 计划 1 环形显示 100（绿色），计划 2 环形显示 75（黄色）

#### Scenario: 已失效计划置灰

- **WHEN** 列表中存在 deadline 早于当天的计划
- **THEN** 该卡片环形为灰色并显示「已失效」徽标

#### Scenario: 无攒钱计划

- **WHEN** 没有任何攒钱计划
- **THEN** 显示空状态提示

### Requirement: 状态详情展示

点击计划卡片 SHALL 内联展开状态详情：target_amount、current_balance、gap、met（账面口径）、allocated、satisfaction（分配口径），以及每账户分配明细（账户显示名、balance、occupied_by_earlier、allocated，每账户一条余额条叠加被占用部分）。再次点击收起。

#### Scenario: 展开状态详情

- **WHEN** 用户点击计划 1（{A,B} 目标 3000），A 3000、B 1000
- **THEN** 展开区显示余额 4000、缺口 -1000、达标，账户明细为 A（余额 3000/被占用 0/分配 2000）与 B（余额 1000/被占用 0/分配 1000）

#### Scenario: 账面与分配口径不一致时同时展示

- **WHEN** 某计划 met 为 true 但 satisfaction 小于 100
- **THEN** 详情区同时显示账面口径（余额/缺口/达标）与分配口径（已分配/满足率）

### Requirement: 创建攒钱计划

攒钱计划页 SHALL 支持通过覆盖式抽屉创建计划，字段：名称、周期类型（daily/weekly-sun/weekly-mon/monthly/yearly/一次性）、deadline（日期，可选）、目标金额、账户集合（多行账户选择，账户选择器仅显示 Assets 子树账户）。币种与预算一致固定为 CNY。一次性计划的 deadline 允许留空（永久计划）。

#### Scenario: 打开创建抽屉

- **WHEN** 用户点击标题栏新建按钮
- **THEN** 抽屉覆盖列表，包含名称、周期、deadline、目标金额、账户多行选择字段

#### Scenario: 提交创建

- **WHEN** 用户填写完整并确认
- **THEN** 系统调用 POST /api/saving-plans 创建计划，成功后刷新列表

#### Scenario: 账户选择器仅显示资产账户

- **WHEN** 用户在表单中打开账户选择器
- **THEN** 仅 Assets 根账户子树的账户可选

#### Scenario: 校验失败提示

- **WHEN** 名称为空、目标金额非正或账户集合为空时提交
- **THEN** 显示错误提示，不发起创建

### Requirement: 编辑攒钱计划

攒钱计划页 SHALL 支持通过抽屉编辑已有计划，预填当前数据（period 为 null 时显示为一次性）。

#### Scenario: 打开编辑抽屉

- **WHEN** 用户点击计划卡片的编辑按钮
- **THEN** 抽屉预填名称、周期/一次性、deadline、目标金额、账户集合

#### Scenario: 提交更新

- **WHEN** 用户修改后确认
- **THEN** 系统调用 PUT /api/saving-plans/:id 更新计划，成功后刷新列表

### Requirement: 删除攒钱计划

攒钱计划页 SHALL 支持删除计划，删除前弹出确认。

#### Scenario: 确认删除

- **WHEN** 用户点击删除按钮并确认
- **THEN** 调用 DELETE /api/saving-plans/:id，计划从列表移除

### Requirement: 界面文本本地化

攒钱计划页的所有界面文本 SHALL 走 vue-i18n，zh-CN 与 en 词条结构对齐（受 i18n.spec 校验），命名空间为 `savingPlan.*` 与 `nav.savingPlan`。

#### Scenario: 切换语言

- **WHEN** 用户切换界面语言
- **THEN** 攒钱计划页全部文本（含环形徽标、表单项、错误提示）切换为对应语言
