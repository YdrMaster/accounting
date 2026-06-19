# AccountTreeList 统一账户列表组件设计

## 背景

在 `2026-06-18-account-picker-bottom-sheet-design.md` 的实现中，出现了以下问题：

1. `AccountPicker` 的触发按钮比其他表单控件大一圈，且暗色模式下背景色不统一。
2. 记账抽屉内的账户卡片布局与账户详情页不一致：
   - 卡片数量不足一行时被拉长充满整行；
   - 只有 2 个子账户时变成竖排；
   - 展开逻辑与账户详情页不一致，多个非直系账户可同时展开。
3. 账户详情页 `AccountCards.vue` 的改动引入了回归：
   - 卡片上预留了 drag handle 但拖动排序未实装；
   - 当卡片恰好填满整行时，“+ 添加”按钮会出现在下一行，导致展开的子账户气泡被添加按钮隔开。

本设计通过提取一个统一的 `AccountTreeList.vue` 集合控件，同时修复抽屉和账户详情页的布局与交互。

## 目标

- 提取 `AccountTreeList.vue`，统一账户详情页与记账抽屉的账户列表布局与交互。
- 修复触发按钮尺寸与暗色模式样式。
- 修复卡片固定宽度、子账户横排、子账户气泡紧跟父卡片、添加按钮位于层级末尾等布局问题。
- 统一展开/选中互斥规则：基于栈的子树展开，同一时间只存在一条展开链。
- 在记账抽屉中支持直接添加账户。
- 实装账户详情页的拖拽排序，并优化添加按钮在排序列表中的处理。

## 非目标

- 不改变后端 API 或账户数据模型。
- 不修改账户详情面板的内容与样式。
- 不在抽屉中显示账户重命名、关闭、删除等管理功能。

## 涉及页面

- `/transaction` — 记一笔
- `/transaction?id={id}` — 编辑交易
- `/transaction/refund` — 录入退款
- `/transaction/reimbursement` — 录入报销
- `/accounts` — 账户详情页

## 第一节：组件架构

新增 `AccountTreeList.vue` 作为纯布局/交互组件，负责多级账户卡片的渲染、展开、选中、添加。账户详情页和记账抽屉都复用它。

```
AccountTree.vue
└── AccountCards.vue（管理页包装）
    └── AccountTreeList.vue
        ├── AccountCard.vue
        └── AddAccountCard.vue（可选）

TransactionForm.vue
└── AccountPicker.vue
    └── BottomSheet.vue
        └── AccountTreeList.vue
```

- `AccountTreeList.vue`：核心集合控件，处理布局、展开、选中、添加占位。
- `AccountCard.vue`：单个账户卡片，负责视觉呈现和点击事件。
- `AddAccountCard.vue`：添加账户占位卡片，用于层级末尾和展开气泡末尾。
- `AccountCards.vue`：账户详情页的包装，注入拖拽、添加确认、详情面板导航。
- `AccountPicker.vue`：记账抽屉的包装，注入选择事件。

## 第二节：布局规则

### 规则清单

1. 所有同级账户卡片固定宽度，按行排列；数量不足一行时不拉伸。
2. 父账户展开后，其子账户作为一个整体气泡**紧跟父卡片之后插入**，同级后续卡片被下推。
3. 当前层级的“+ 添加”按钮始终位于**该层级所有卡片及展开气泡之后**。
4. 子账户气泡与父卡片之间通过三角箭头视觉连接。

### 正确布局示例 HTML

以下 HTML 归档了账户详情页（暗色模式）下“支出”类型中“正餐”被展开时的正确 DOM/视觉结构，供后续实现和查询：

```html
<div class="cards-grid">
  <!-- 第一行：同级账户 -->
  <div class="card-wrapper">
    <div class="account-card">
      <span class="drag-handle">⠿</span>
      <span class="card-name">手续费</span>
      <span class="card-actions"><span class="add-btn">+</span></span>
    </div>
  </div>
  <div class="card-wrapper">
    <div class="account-card">
      <span class="drag-handle">⠿</span>
      <span class="card-name">折扣</span>
      <span class="card-actions"><span class="add-btn">+</span></span>
    </div>
  </div>
  <div class="card-wrapper">
    <div class="account-card">
      <span class="drag-handle">⠿</span>
      <span class="card-name">分期手续费</span>
      <span class="card-actions"><span class="add-btn">+</span></span>
    </div>
  </div>

  <!-- 第二行：正在展开的父卡片 + 后续同级卡片 -->
  <div class="card-wrapper">
    <div class="account-card selected expanded">
      <span class="drag-handle">⠿</span>
      <span class="card-name">正餐</span>
      <span class="card-actions">
        <span class="child-count">2</span>
        <span class="expand-arrow rotated">▲</span>
      </span>
    </div>
  </div>
  <div class="card-wrapper">
    <div class="account-card">
      <span class="drag-handle">⠿</span>
      <span class="card-name">日用品</span>
      <span class="card-actions"><span class="add-btn">+</span></span>
    </div>
  </div>
  <div class="card-wrapper">
    <div class="account-card">
      <span class="drag-handle">⠿</span>
      <span class="card-name">零食</span>
      <span class="card-actions"><span class="add-btn">+</span></span>
    </div>
  </div>

  <!-- 第三行：展开气泡，紧跟父卡片之后，同级后续卡片被下推 -->
  <div class="sub-cards" style="position: relative;">
    <!-- 气泡三角箭头，指向父卡片 -->
    <div class="sub-cards-arrow"></div>
    <div class="card-wrapper">
      <div class="account-card">
        <span class="drag-handle">⠿</span>
        <span class="card-name">餐厅</span>
        <span class="card-actions"><span class="add-btn">+</span></span>
      </div>
    </div>
    <div class="card-wrapper">
      <div class="account-card">
        <span class="drag-handle">⠿</span>
        <span class="card-name">外卖</span>
        <span class="card-actions"><span class="add-btn">+</span></span>
      </div>
    </div>
    <div class="card-wrapper">
      <div class="account-card add-card-box">
        <span class="add-card-text">+ 添加</span>
      </div>
    </div>
  </div>

  <!-- 第四行：当前层级的末尾添加按钮 -->
  <div class="card-wrapper">
    <div class="account-card add-card-box">
      <span class="add-card-text">+ 添加</span>
    </div>
  </div>
</div>
```

对应的 CSS 箭头实现参考：

```css
.sub-cards {
  position: relative;
  border: 1px solid var(--bubble-border, #d9d9d9);
  border-radius: 8px;
  background: var(--bubble-bg, #fafafa);
  padding: 12px;
}
.sub-cards-arrow {
  position: absolute;
  top: -8px;
  left: 16%; /* 与父卡片水平中心对齐 */
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid var(--bubble-border, #d9d9d9);
}
.sub-cards-arrow::after {
  content: '';
  position: absolute;
  top: 1px;
  left: -8px;
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid var(--bubble-bg, #fafafa);
}
```

## 第三节：基于栈的展开/选中互斥规则

`AccountTreeList` 内部维护一个 `expandedStack: number[]`，表示**从当前层级到最深展开账户的唯一路径**。这个栈保证同一时间只存在一条展开链。

### 栈的语义

- `expandedStack` 中的每个元素都是下一个元素的父账户。
- 只有栈内账户及其直系子账户处于可见/可交互状态。
- 栈顶账户是“当前最深层展开的账户”，它的子账户会被渲染出来。

### 状态计算

- 账户 A 被选中：当且仅当 `expandedStack[expandedStack.length - 1] === A.id`（栈顶），或 A 是无子账户的叶子且处于单独选中状态。
- 账户 A 被展开：当且仅当 `expandedStack.includes(A.id)` 且 A 不是栈底（即它有子账户被展示）。

### 操作规则

| 操作 | 对 expandedStack 的影响 | 结果 |
|---|---|---|
| 选中 A（A 无子账户） | 清空栈，将 A 压入栈 | A 高亮；其他选中/展开全部清除 |
| 选中 A（A 有子账户） | 清空栈，将 A 的直系祖先路径 + A 压入栈 | A 高亮；A 的子账户不自动展开 |
| 展开 A | 若 A 是当前栈顶账户的父级或同级，清空栈并重建 A 的祖先路径 + A；若 A 是当前栈顶账户的子级，将 A 压入栈 | A 展开，显示其子账户；清除其他无关选中/展开 |
| 展开已选中的 A | A 已在栈顶，将 A 的子级压入栈（如果点击的是子级展开）或保持 A 在栈顶 | A 保持选中，A 的子账户展开 |
| 选中已展开的 A | A 已在栈中，保持栈不变 | A 保持展开并高亮 |
| 选中 B（A 已展开/选中） | 清空栈，重建 B 的祖先路径 + B | A 收起，B 选中 |
| 展开 B（A 已选中/展开） | 清空栈，重建 B 的祖先路径 + B | A 收起，B 展开 |
| 折叠当前展开的 A | 从栈中弹出 A 及其以上所有后代 | A 的子账户收起；A 自身保持选中（如果它在栈底被选中） |

### 核心不变量

1. **单链原则：** `expandedStack` 始终是一条从当前层级某个账户到其深层后代的单一路径。
2. **互斥原则：** 任何不在 `expandedStack` 路径上的账户，不会同时处于选中或展开状态。
3. **可见性原则：** 子账户只在父账户位于栈中时被渲染。

## 第四节：AccountTreeList 接口设计

### Props

```ts
interface AccountTreeListProps {
  accounts: Account[]           // 全部账户
  parentId: number | null       // 当前渲染层级的父账户 ID
  type?: string                 // 按账户类型过滤（如 'Asset'）
  modelValue?: number           // 当前选中账户 ID（选择模式）
  activeId?: number | null      // 当前活动账户 ID（管理页与抽屉共用）
  mode: 'select' | 'manage'     // 选择模式（抽屉）/ 管理模式（账户页）
  allowAdd: boolean             // 是否允许直接添加账户
  allowDrag: boolean            // 是否允许拖拽排序
}
```

### Emits

```ts
interface AccountTreeListEmits {
  'update:modelValue': [id: number]          // 选择模式：选中账户
  'update:activeId': [id: number | null]     // 活动账户变化
  'add': [parentId: number | null]            // 请求添加子账户/根账户
  'reorder': [ids: number[]]                  // 拖拽排序完成
}
```

### 模式差异

| 功能 | `mode='select'`（抽屉） | `mode='manage'`（账户详情页） |
|---|---|---|
| 点击卡片主体 | 选中并 emit `update:modelValue` | 选中并 emit `update:activeId`，用于显示详情面板 |
| 展开箭头 | 用于浏览子账户 | 用于浏览子账户 |
| drag handle | 不显示 | 显示，支持拖拽排序 |
| 添加按钮 | 显示，触发 `add` 事件 | 显示，触发 `add` 事件 |
| 内联添加输入 | 由父组件提供并控制 | 由父组件提供并控制 |

## 第五节：添加账户行为

1. **无子账户的卡片**：显示 `+` 按钮，点击后该卡片展开并进入内联添加状态。
2. **有子账户的卡片**：展开后子账户区域末尾出现 `+ 添加` 卡片，点击进入添加状态。
3. **层级末尾**：始终有一个 `+ 添加` 按钮/卡片，用于添加同级账户。
4. **添加状态控制**：`AccountTreeList` 只负责渲染内联输入行占位并触发 `add` 事件；实际输入框、确认/取消按钮由父组件通过 slot 或状态控制。
5. **抽屉内添加**：确认添加后刷新账户列表，抽屉保持打开，用户可继续选择。

## 第六节：拖拽排序实现优化

### 列表构成

可排序列表 = `[同级账户卡片 ..., "+ 添加"卡片]`。

- “+ 添加”卡片没有 drag handle，用户无法用它发起拖拽。
- 其他账户卡片有 drag handle，可以拖拽。

### 拖拽结束后的自动校正

1. vuedraggable 的 `@end` 事件返回新的顺序。
2. 检查新顺序中“+ 添加”卡片是否位于最后一位。
3. 如果不是最后一位，更新列表把它移动到最后一位；vuedraggable 的 `:animation` 属性会让这次移动播放过渡动画，用户能看到添加按钮自动滑回末尾。
4. 动画结束后触发 `reorder` 事件，仅包含真实账户 ID，不包含添加按钮的占位 ID。

### 为什么这样设计

- 避免为“添加按钮不参与排序”写大量特殊分支。
- 用户视觉上仍然感知添加按钮固定在末尾。
- 排序持久化时只需提交真实账户 ID 列表。

## 第七节：触发按钮与暗色模式修复

### 触发按钮尺寸

- 高度与表单中其他输入框一致（约 38px）。
- 内边距、边框圆角与其他输入框一致。

### 暗色模式样式

```css
html.dark .picker-trigger {
  background: #1f1f1f !important;
  border-color: #434343 !important;
  color: #fff !important;
}
html.dark .picker-trigger:hover {
  border-color: #177ddc !important;
}
html.dark .picker-trigger .placeholder {
  color: #888 !important;
}
html.dark .picker-trigger .selected-name {
  color: #fff !important;
}
```

### AccountTreeList 暗色模式

随组件调整同步更新 `App.vue` 中的全局暗色覆盖样式，确保卡片、气泡、添加按钮、drag handle 在暗色模式下颜色正确。

## 第八节：验证策略

### 自动验证

- `cd accounting-web && npm run build` 通过，无 TypeScript 类型错误。

### 手动验证清单

#### 触发按钮

- [ ] `/transaction` 页面中触发按钮高度与币种、金额输入框一致。
- [ ] 暗色模式下触发按钮背景、边框、文字与其他输入框一致。

#### 抽屉内账户列表

- [ ] 点击账户输入框，抽屉从底部滑出。
- [ ] 卡片固定宽度，数量不足一行时不拉伸。
- [ ] 只有 2 个子账户时横向排列。
- [ ] 展开父账户后，子账户气泡紧跟父卡片，同级后续卡片被下推。
- [ ] 同一时间只有一个账户被展开/选中。
- [ ] 选中账户后抽屉保持打开，表单更新。
- [ ] 可在抽屉内直接添加子账户/同级账户。

#### 账户详情页

- [ ] 卡片上 drag handle 可拖动排序。
- [ ] 拖拽结束后添加按钮自动滑回末尾。
- [ ] 展开/选中互斥规则与抽屉一致。
- [ ] 子账户气泡位置正确，不被添加按钮隔开。
- [ ] 添加子账户功能正常。

#### 暗色模式

- [ ] 抽屉、卡片、气泡、添加按钮、drag handle 颜色正确。

## 第九节：文件变更清单

| 文件 | 变更类型 | 说明 |
|---|---|---|
| `accounting-web/src/components/AccountTreeList.vue` | 新建 | 统一账户列表集合控件 |
| `accounting-web/src/components/AccountCard.vue` | 修改 | 调整样式以支持固定宽度、统一暗色模式 |
| `accounting-web/src/views/AccountCards.vue` | 修改 | 改用 `AccountTreeList`，保留管理功能 |
| `accounting-web/src/components/AccountPicker.vue` | 修改 | 改用 `AccountTreeList`，支持添加账户 |
| `accounting-web/src/components/AccountSelector.vue` | 删除 | 被 `AccountTreeList` 取代 |
| `accounting-web/src/components/BottomSheet.vue` | 修改 | 如有必要，微调样式配合新列表 |
| `accounting-web/src/App.vue` | 修改 | 更新暗色模式样式 |

## 后续步骤

1. 调用 `writing-plans` 技能生成实现计划。
2. 按实现计划编码、验证、提交。
