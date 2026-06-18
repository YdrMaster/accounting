# AccountPicker 底部抽屉改造设计

## 背景

当前在交易表单页（`TransactionForm.vue`）中，点击分录行的账户输入框会弹出一个居中的模态框（`AccountPicker.vue`）。该模态框带有深色半透明遮罩，覆盖在表单上方并阻止用户与背景表单交互。

涉及以下页面/路由：

- `/transaction` — 记一笔
- `/transaction?id={id}` — 编辑交易
- `/transaction/refund` — 录入退款
- `/transaction/reimbursement` — 录入报销

（注：退款/报销当前代码中仅有录入入口，没有独立编辑路由；它们与记一笔/编辑交易共用同一个 `TransactionForm.vue` 组件，因此本次改造会同时覆盖以上所有入口。）

本设计将其改造为从页面底部向上展开的抽屉，并与表单在同一宽度范围内对齐，以提升移动端体验和视觉一致性。

## 目标

- 将 `AccountPicker` 的账户选择交互从居中模态框改为底部抽屉。
- 抽屉内部复用 `/accounts` 页面的账户卡片选择/展开 UI。
- 支持任意账户节点（包括父节点）被选中。
- 提供展开/收起动画，并在移动端支持拖动伸缩与意图 snap。
- 保持暗色模式兼容，不破坏现有账户管理页功能。

## 非目标

- 不改动账户管理页（`AccountTree.vue`、`AccountCards.vue`）的业务逻辑，仅将其选择/展开 UI 提取复用。
- 不改变后端 API 或账户数据模型。
- 不在抽屉中显示账户详情面板、添加/删除/重命名等管理功能。

## 现状

### 相关文件

- `accounting-web/src/components/AccountPicker.vue`：当前实现为居中模态框，内部使用网格按钮 + 面包屑导航。
- `accounting-web/src/views/TransactionForm.vue`：唯一使用 `AccountPicker` 的页面。
- `accounting-web/src/views/AccountTree.vue`：账户管理页，包含卡片网格和详情面板。
- `accounting-web/src/components/AccountCards.vue`：账户卡片组件，负责卡片展示、层级展开、选中高亮、拖拽排序、添加子账户等。

### 当前 AccountPicker 行为

1. 点击触发按钮 → 显示固定定位的全屏遮罩 + 居中面板（`z-index: 1000`）。
2. 面板内按当前层级显示账户网格按钮。
3. 只有叶子节点可被选中；非叶子节点点击进入下一级。
4. 选中后立即关闭面板并更新 `v-model`。

## 设计方案

### 第一节：整体变更概述

- 将 `AccountPicker.vue` 从“居中遮罩模态框”改为“底部向上滑出的抽屉”。
- 抽屉内部使用与 `/accounts` 页面一致的卡片网格 + 层级展开交互。
- 抽屉带极淡遮罩，点击遮罩或关闭按钮可关闭；抽屉打开时背景表单仍可见。
- 抽屉宽度与表单容器一致（最大 600px，居中），不充满整屏。

### 关键交互规则

- 点击卡片的非箭头区域：选中该账户（无论叶子节点还是父节点）。
- 点击卡片右侧的下箭头：展开/折叠该父级账户的子账户。
- 选中账户后抽屉**不自动关闭**，用户可继续浏览或切换选择。
- 关闭抽屉的方式：
  - 点击背景遮罩；
  - 点击关闭按钮；
  - 向下拖动抽屉（移动端）。
- 展开/收起带动画；移动端拖动时抽屉可在最大范围内实时跟随手指，松手后根据位置/速度判断意图，动画到完全展开或完全闭合。

### 不变的部分

- 账户数据来源、账户树结构、账户类型过滤逻辑保持不变。
- 选中账户后仍通过 `v-model` 更新表单中的 `accountId`。

## 第二节：组件架构

### 新增组件

#### `BottomSheet.vue`

可复用的底部抽屉容器：

- 支持 `v-model:open` 控制显示/隐藏。
- 宽度与父容器/表单对齐：
  - `width: 100%`;
  - `max-width: 600px`（与 `.transaction-form` 的 `max-width` 一致）；
  - 水平居中。
- 移动端（屏幕宽度 < 600px）时抽屉占满屏幕宽度减去两侧内边距。
- 展开/收起使用 CSS transition 动画。
- 移动端支持 touch 拖动：
  - 手指按住抽屉顶部 drag handle 或标题栏时，实时改变高度；
  - 松手后根据当前高度（< 40% 最大高度则关闭）或速度判断 snap；
  - 动画到完全展开或完全闭合。
- 提供默认关闭按钮和点击遮罩关闭行为。
- 极淡遮罩：仅提供视觉分隔，点击遮罩会关闭抽屉，随后可继续操作背景表单。

#### `AccountSelector.vue`

从 `AccountCards.vue` 提取的纯展示与选择组件：

- 输入 props：
  - `accounts: Account[]` — 账户列表；
  - `parentId: number | null` — 当前渲染的父节点；
  - `type?: string` — 按账户类型过滤（如 `'Asset'`）；
  - `modelValue?: number` — 当前选中账户 ID。
- 输出事件：
  - `update:modelValue` — 选中账户时触发。
- 内部状态：
  - `expandedStack: number[]` — 哪些父节点已展开。
- 不包含：拖拽排序、添加/删除/重命名账户、详情面板、账户管理相关逻辑。

### 改造现有组件

#### `AccountPicker.vue`

职责变轻：

- 保留触发按钮（显示当前选中账户或占位文字）。
- 点击触发按钮打开 `BottomSheet`。
- 在 `BottomSheet` 内嵌入 `AccountSelector`。
- 接收 `accountType` 等 props 并透传给 `AccountSelector`。
- 保存 `showSheet` 状态；抽屉关闭时重置 `AccountSelector` 的 `expandedStack`，下次打开时回到根层级。

#### `AccountCards.vue`

把卡片展示/展开/选中的逻辑下沉到 `AccountSelector`，`AccountCards` 保留：

- 拖拽排序（vuedraggable）；
- 添加子账户/根账户；
- 展开后显示详情面板的导航逻辑。

### 组件关系

```
TransactionForm.vue
└── AccountPicker.vue（改造）
    ├── AccountSelector.vue（新增/提取）
    │   └── 递归使用自身处理子级
    └── BottomSheet.vue（新增）
        ├── 动画/拖动逻辑
        └── 极淡遮罩

AccountTree.vue
└── AccountCards.vue（改造：使用 AccountSelector）
```

## 第三节：UI/UX 细节

### 抽屉尺寸与定位

- 定位：`position: fixed; bottom: 0;` 水平居中对齐。
- 宽度：`width: 100%; max-width: 600px;`。
- 移动端（< 600px）：抽屉宽度等于屏幕宽度减去两侧内边距（如 16px）。
- 最大高度：默认展开到内容高度，但不超过视口高度的 70%（`max-height: 70vh`），超出可内部滚动。
- 最小展开高度：即使内容很少，也至少显示标题栏 + 少量卡片（约 160px）。
- 遮罩：全屏覆盖，亮度模式下使用 `rgba(0, 0, 0, 0.15)`，暗色模式下使用 `rgba(0, 0, 0, 0.25)`。

### 动画与拖动行为

- 打开：从屏幕底部外滑入到目标高度，动画 250ms，ease-out。
- 关闭：向下滑出屏幕底部，动画 200ms，ease-in。
- 拖动时：手指按住抽屉顶部 drag handle 或标题栏，实时跟随垂直位移；高度在 `[0, maxHeight]` 之间线性映射。
- 松手 snap：
  - 如果当前高度 < 最大高度的 40%，或松手时向下速度 > 阈值，则动画关闭；
  - 否则动画展开到最大高度。
- 点击遮罩关闭：与拖动关闭共用同一套退出动画。

### AccountSelector 交互

- 卡片默认边框：`1px solid #d9d9d9`，圆角 6px。
- 选中态：边框 `#1890ff`，背景 `#e6f7ff`。
- 父节点卡片：右侧显示展开箭头 `▼`，展开后旋转为 `▲`；点击箭头才展开/折叠，点击卡片其他区域选中。
- 叶子节点卡片：无箭头，点击即选中。
- 子账户以气泡形式嵌套在父卡片下方，与 `AccountCards` 视觉一致。

### 暗色模式

- 抽屉背景：`html.dark` 下使用 `#1f1f1f`，边框 `#434343`。
- 卡片默认：背景 `#1f1f1f`，边框 `#434343`，文字 `#fff`。
- 卡片选中：背景 `#111d2c`，边框 `#177ddc`。
- 遮罩：暗色模式下使用 `rgba(0,0,0,0.25)`，保持“极淡”。

## 第四节：数据流

### 打开抽屉

1. 用户点击 `AccountPicker` 的触发按钮。
2. `AccountPicker` 将内部 `showSheet` 设为 `true`。
3. `BottomSheet` 接收 `open` prop，执行进入动画并渲染内容。
4. `AccountSelector` 从 `accountStore.accounts` 读取账户树，根据 `accountType` prop 过滤根账户。

### 选择账户

1. 用户点击 `AccountSelector` 中的卡片主体。
2. `AccountSelector` 触发 `update:modelValue`，把账户 ID 传回 `AccountPicker`。
3. `AccountPicker` 通过 `v-model` 把新 ID 同步给 `TransactionForm` 中的 `posting.accountId`。
4. 抽屉保持打开，用户可继续浏览。

### 展开/折叠层级

1. 用户点击父节点卡片的展开箭头。
2. `AccountSelector` 内部维护 `expandedStack: number[]`。
3. 如果该节点已在 stack 顶部，则弹出（折叠）；否则推入 stack（展开）。
4. 子级通过递归渲染自身，传入更新后的 stack。

### 关闭抽屉

1. 用户点击遮罩、关闭按钮，或向下拖动并触发关闭 snap。
2. `BottomSheet` 执行退出动画，动画结束后通知父组件 `update:open`。
3. `AccountPicker` 将 `showSheet` 设为 `false`。

### 状态边界

- `BottomSheet` 不保存业务状态，只管理自身的开合/拖动动画。
- `AccountSelector` 不保存选中状态，只通过 `modelValue` 与父组件同步；`expandedStack` 为内部 UI 状态。
- `AccountPicker` 保存 `showSheet` 状态，并在关闭时重置 `expandedStack`。

## 第五节：边界情况与错误处理

### 账户列表为空

- 如果 `accountStore.accounts` 为空，抽屉内显示“暂无账户”。
- 触发按钮禁用或显示“请先添加账户”。

### 按类型过滤后无结果

- `AccountPicker` 可能传入 `accountType="Asset"`（如退款/报销模式）。
- 过滤后如果没有匹配的根账户，显示“没有该类型的账户”。

### 选中后账户被删除/改名

- 表单中已经保存了 `accountId`，如果对应账户后续被删除，触发按钮显示账户 ID 或“账户不存在”。
- 不强制清空已选值，避免破坏用户输入；提交时由后端校验。

### 移动端拖动冲突

- 抽屉内部卡片区域滚动与抽屉整体拖动需要区分：只有按住抽屉顶部 drag handle 或抽屉标题栏时才触发整体拖动。
- 抽屉内容区域垂直滚动不应误触发关闭。
- 抽屉出现时锁定 body 滚动（可选），防止背后页面滚动干扰拖动体验。

### 暗色模式切换

- 所有新增样式使用 `html.dark` 选择器，与现有暗色模式兼容。
- 抽屉打开期间切换主题，样式应立即生效。

### 可访问性

- 抽屉打开时，焦点应移动到抽屉内第一个可交互元素（或关闭按钮）。
- 按 `Esc` 键关闭抽屉。
- 触发按钮应使用 `button` 元素，支持键盘操作。

## 第六节：验证策略

### 手动验证清单

- 在 `/transaction` 页面点击账户输入框，抽屉从底部滑出，宽度与表单对齐。
- 点击账户卡片主体，表单中的账户显示更新，抽屉保持打开。
- 点击父节点箭头，子账户气泡展开/折叠，选中态不丢失。
- 点击遮罩、关闭按钮、向下拖动抽屉，抽屉关闭动画正常。
- 移动端真机/模拟器：手指拖动抽屉可实时伸缩，松手后正确 snap 到打开/关闭。
- 退款/报销模式（`/transaction/refund`、`/transaction/reimbursement`）只显示资产类账户。
- 暗色模式下抽屉、卡片、选中态、遮罩颜色正确。
- `/accounts` 页面账户卡片交互（选择、展开、拖拽、添加）未受影响。

### 自动验证

- 运行 `npm run build`（TypeScript 类型检查 + Vite 构建）确保无编译错误。
- 如果项目后续引入组件测试，补充 `AccountSelector` 的单元测试：选中事件、展开状态、过滤。

### 回归范围

- `AccountPicker` 在普通记账、退款、报销三种模式下的使用。
- 账户管理页 `AccountCards` / `AccountTree` 的现有功能。
- 暗色模式样式。

## 方案对比与选择

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A：提取共享 `AccountSelector` + 自定义 `BottomSheet` | 真正复用现有 UI 逻辑；抽屉行为完全可控；后续任何页面选账户都能复用 | 需改动 `AccountCards`，有回归风险；自定义抽屉需处理滚动/焦点/暗色模式 | **选用** |
| B：新建独立 `AccountSelector` + 自定义 `BottomSheet` | 不触碰账户管理页，风险最低 | 两套相似代码，维护成本高 | 未选用 |
| C：最小改动，只改弹窗为底部抽屉 | 实现最快 | 没有真正层级展开卡片，与账户页交互不一致 | 未选用 |

## 后续步骤

1. 调用 `writing-plans` 技能生成实现计划。
2. 按实现计划编码、验证、提交。
