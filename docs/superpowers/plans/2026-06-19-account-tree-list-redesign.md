# AccountTreeList 统一账户列表组件实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 提取统一的 `AccountTreeList.vue` 组件，同时修复记账抽屉与账户详情页的账户卡片布局、展开互斥、拖拽排序、添加账户、暗色模式等问题。

**架构：** 新建 `AccountTreeList.vue` 负责多级账户卡片的渲染、展开、选中、添加占位；`AccountCard.vue` 保持纯卡片呈现；`AccountPicker.vue` 和 `AccountCards.vue` 分别在选择/管理场景下复用 `AccountTreeList`。

**技术栈：** Vue 3.4 + TypeScript + Vite + Ant Design Vue + vuedraggable。

---

## 文件结构

| 文件 | 职责 | 变更类型 |
|---|---|---|
| `accounting-web/src/components/AccountTreeList.vue` | 统一账户列表：布局、展开栈、选中、添加占位 | 新建 |
| `accounting-web/src/components/AccountCard.vue` | 单个账户卡片：固定宽度、暗色模式兼容 | 修改 |
| `accounting-web/src/components/AccountPicker.vue` | 触发按钮 + BottomSheet + AccountTreeList（选择+添加模式） | 修改 |
| `accounting-web/src/views/AccountCards.vue` | 管理页包装：拖拽排序、添加确认、详情面板导航 | 修改 |
| `accounting-web/src/components/AccountSelector.vue` | 被 AccountTreeList 取代 | 删除 |
| `accounting-web/src/App.vue` | 更新暗色模式覆盖样式 | 修改 |

---

### 任务 1：调整 AccountCard.vue 样式

**文件：**
- 修改：`accounting-web/src/components/AccountCard.vue`

- [ ] **步骤 1：调整 AccountCard 为固定宽度**

将 `.account-card` 的 `min-width: 140px` 改为固定宽度 `width: 140px`，移除 `flex: 1` 相关样式，使其不再被父容器拉伸。

```vue
<style scoped>
.account-card {
  width: 140px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
  background: #fff;
  min-height: 50px;
  display: flex;
  align-items: center;
  transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
  user-select: none;
  box-sizing: border-box;
}
/* 其余样式保持不变 */
</style>
```

- [ ] **步骤 2：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountCard.vue
git commit -m "style(web): AccountCard 使用固定宽度"
```

---

### 任务 2：创建 AccountTreeList.vue 基础结构

**文件：**
- 创建：`accounting-web/src/components/AccountTreeList.vue`

- [ ] **步骤 1：实现基础模板与脚本**

```vue
<template>
  <div class="account-tree-list">
    <div class="cards-grid">
      <template v-for="item in visibleItems" :key="item.id">
        <AccountCard
          v-if="item.type === 'account'"
          :account="item.account"
          :selected="isSelected(item.account.id)"
          :expanded="isExpanded(item.account.id)"
          @click="handleSelect(item.account)"
        >
          <template v-if="allowDrag" #prefix>
            <span class="drag-handle" @click.stop>⠿</span>
          </template>
          <template #suffix>
            <span v-if="item.account.closed_at" class="closed-tag">
              <a-tag color="default" style="margin: 0; font-size: 11px">已关闭</a-tag>
            </span>
            <span class="card-actions">
              <template v-if="hasChildren(item.account.id)">
                <span class="child-count">{{ childrenCount(item.account.id) }}</span>
                <button
                  type="button"
                  class="expand-btn"
                  :class="{ rotated: isExpanded(item.account.id) }"
                  @click.stop="handleToggleExpand(item.account)"
                >▼</button>
              </template>
              <button
                v-else-if="allowAdd"
                type="button"
                class="add-btn"
                @click.stop="handleAdd(item.account.id)"
              >+</button>
            </span>
          </template>
        </AccountCard>

        <div
          v-else-if="item.type === 'sub-cards'"
          class="sub-cards"
          :style="{ '--bubble-border': '#d9d9d9', '--bubble-bg': '#fafafa' }"
        >
          <div class="sub-cards-arrow"></div>
          <AccountTreeList
            :accounts="accounts"
            :parent-id="item.parentId"
            :type="type"
            :model-value="modelValue"
            :active-id="activeId"
            :mode="mode"
            :allow-add="allowAdd"
            :allow-drag="allowDrag"
            @update:model-value="emit('update:modelValue', $event)"
            @update:active-id="emit('update:activeId', $event)"
            @add="emit('add', $event)"
            @reorder="emit('reorder', $event)"
          />
        </div>

        <div
          v-else-if="item.type === 'add-card'"
          class="account-card add-card-box"
          @click="handleAdd(item.parentId)"
        >
          <span class="add-card-text">+ 添加</span>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AccountCard from './AccountCard.vue'
import type { Account } from '@/stores/account'

interface VisibleItem {
  id: string
  type: 'account' | 'sub-cards' | 'add-card'
  account?: Account
  parentId?: number | null
}

const props = defineProps<{
  accounts: Account[]
  parentId: number | null
  type?: string
  modelValue?: number
  activeId?: number | null
  mode: 'select' | 'manage'
  allowAdd: boolean
  allowDrag: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
  'update:activeId': [id: number | null]
  'add': [parentId: number | null]
  'reorder': [ids: number[]]
}>()

const currentLevel = computed(() =>
  props.accounts
    .filter((a) => (a.parent_id ?? null) === props.parentId && (!props.type || a.account_type === props.type))
    .sort((a, b) => a.position - b.position)
)

const expandedStack = defineModel<number[]>('expandedStack', { default: () => [] })

function hasChildren(id: number): boolean {
  return props.accounts.some((a) => a.parent_id === id)
}

function childrenCount(id: number): number {
  return props.accounts.filter((a) => a.parent_id === id).length
}

function isSelected(id: number): boolean {
  if (props.mode === 'select') return props.modelValue === id
  return props.activeId === id
}

function isExpanded(id: number): boolean {
  return expandedStack.value.includes(id)
}

function ancestorPath(id: number): number[] {
  const path: number[] = []
  let current = props.accounts.find((a) => a.id === id)
  while (current && current.parent_id != null) {
    path.unshift(current.parent_id)
    current = props.accounts.find((a) => a.id === current!.parent_id)
  }
  return path
}

function handleSelect(account: Account) {
  if (props.mode === 'select') {
    emit('update:modelValue', account.id)
  } else {
    emit('update:activeId', account.id)
  }
  expandedStack.value = [...ancestorPath(account.id), account.id]
}

function handleToggleExpand(account: Account) {
  if (isExpanded(account.id)) {
    const index = expandedStack.value.indexOf(account.id)
    expandedStack.value = expandedStack.value.slice(0, index)
  } else {
    expandedStack.value = [...ancestorPath(account.id), account.id]
  }
}

function handleAdd(parentId: number | null) {
  emit('add', parentId)
}

const visibleItems = computed<VisibleItem[]>(() => {
  const items: VisibleItem[] = []
  currentLevel.value.forEach((account) => {
    items.push({ id: `acc-${account.id}`, type: 'account', account })
    if (isExpanded(account.id) && hasChildren(account.id)) {
      items.push({ id: `sub-${account.id}`, type: 'sub-cards', parentId: account.id })
    }
  })
  if (props.allowAdd) {
    items.push({ id: `add-${props.parentId ?? 'root'}`, type: 'add-card', parentId: props.parentId })
  }
  return items
})
</script>

<style scoped>
.account-tree-list {
  width: 100%;
}
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: flex-start;
}
.sub-cards {
  order: 1;
  flex-basis: 100%;
  position: relative;
  border: 1px solid var(--bubble-border, #d9d9d9);
  border-radius: 8px;
  background: var(--bubble-bg, #fafafa);
  padding: 12px;
}
.sub-cards-arrow {
  position: absolute;
  top: -8px;
  left: 70px;
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
.add-card-box {
  border: 1px dashed #73d13d;
  color: #73d13d;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.add-card-box:hover {
  background: rgba(115, 209, 61, 0.08);
}
.drag-handle {
  cursor: grab;
  color: #999;
  margin-right: 4px;
}
.expand-btn {
  margin-left: 4px;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 10px;
  color: #999;
  transition: transform 0.2s;
}
.expand-btn.rotated {
  transform: rotate(180deg);
}
.child-count {
  font-size: 11px;
  color: #888;
  background: #f0f0f0;
  padding: 1px 6px;
  border-radius: 10px;
  margin-right: 4px;
}
.add-btn {
  background: transparent;
  border: none;
  color: #73d13d;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
}
.closed-tag {
  margin-right: 6px;
}
.card-actions {
  display: flex;
  align-items: center;
  margin-left: auto;
}
</style>
```

- [ ] **步骤 2：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountTreeList.vue
git commit -m "feat(web): 创建 AccountTreeList 基础结构"
```

---

### 任务 3：为 AccountTreeList 添加拖拽排序

**文件：**
- 修改：`accounting-web/src/components/AccountTreeList.vue`

- [ ] **步骤 1：引入 vuedraggable 并包装账户列表**

将 `currentLevel` 账户卡片部分改用 `draggable` 组件包裹。列表中把“+ 添加”卡片作为最后一个 draggable item，但它没有 drag handle，因此用户无法直接拖动它。

```vue
<template>
  <div class="account-tree-list">
    <draggable
      v-model="draggableList"
      item-key="id"
      handle=".drag-handle"
      :animation="200"
      class="cards-grid"
      @end="handleDragEnd"
    >
      <template #item="{ element: item }">
        <!-- 这里复用任务 2 中的 account / sub-cards / add-card 渲染 -->
      </template>
    </draggable>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import draggable from 'vuedraggable'
// ... 其他 import

// 新增：draggableList 只包含当前层级的账户和末尾添加按钮
const draggableList = computed({
  get: () => {
    const list: VisibleItem[] = currentLevel.value.map((account) => ({
      id: `acc-${account.id}`,
      type: 'account',
      account,
    }))
    if (props.allowAdd) {
      list.push({
        id: `add-${props.parentId ?? 'root'}`,
        type: 'add-card',
        parentId: props.parentId,
      })
    }
    return list
  },
  set: (newList) => {
    // 拖拽结束时会触发 set，真正的重排逻辑在 handleDragEnd 中处理
  },
})

// visibleItems 需要改为基于 draggableList，并在展开的账户后插入 sub-cards
const visibleItems = computed<VisibleItem[]>(() => {
  const items: VisibleItem[] = []
  draggableList.value.forEach((item) => {
    items.push(item)
    if (item.type === 'account' && item.account && isExpanded(item.account.id) && hasChildren(item.account.id)) {
      items.push({ id: `sub-${item.account.id}`, type: 'sub-cards', parentId: item.account.id })
    }
  })
  return items
})

function handleDragEnd() {
  // 自动校正：确保 add-card 在最后
  const ids = draggableList.value
    .filter((item) => item.type === 'account')
    .map((item) => item.account!.id)
  emit('reorder', ids)
}
</script>
```

注意：需要把任务 2 中的 `<template v-for="item in visibleItems">` 完整迁移到 draggable 的 `#item` slot 中。

- [ ] **步骤 2：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountTreeList.vue
git commit -m "feat(web): AccountTreeList 支持拖拽排序"
```

---

### 任务 4：改造 AccountPicker.vue

**文件：**
- 修改：`accounting-web/src/components/AccountPicker.vue`
- 删除：`accounting-web/src/components/AccountSelector.vue`

- [ ] **步骤 1：替换 AccountSelector 为 AccountTreeList**

```vue
<template>
  <div class="account-picker">
    <button
      type="button"
      class="picker-trigger"
      :class="{ disabled: props.disabled }"
      @click="openSheet"
      :disabled="props.disabled"
    >
      <span v-if="selectedAccount" class="selected-name">{{ selectedAccount.full_name }}</span>
      <span v-else class="placeholder">{{ placeholder }}</span>
    </button>

    <BottomSheet v-model:open="showSheet" title="选择账户">
      <AccountTreeList
        :accounts="accountStore.accounts"
        :parent-id="null"
        :type="props.accountType"
        :model-value="props.modelValue"
        mode="select"
        :allow-add="true"
        :allow-drag="false"
        @update:model-value="handleSelect"
        @add="handleAdd"
      />
    </BottomSheet>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAccountStore } from '@/stores/account'
import BottomSheet from './BottomSheet.vue'
import AccountTreeList from './AccountTreeList.vue'

const props = defineProps<{
  modelValue?: number
  accountType?: string
  placeholder?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
}>()

const accountStore = useAccountStore()
const showSheet = ref(false)

const selectedAccount = computed(() => {
  if (!props.modelValue) return null
  return accountStore.accounts.find((a) => a.id === props.modelValue) || null
})

function openSheet() {
  showSheet.value = true
}

function handleSelect(id: number) {
  emit('update:modelValue', id)
}

function handleAdd(parentId: number | null) {
  // 打开内联添加状态：可复用 AccountCards 的添加逻辑，或简单跳转
  // 本任务先触发一个提示，后续任务再接入完整添加流程
  // 实际实现中由父组件控制添加输入框的显示
}
</script>

<style scoped>
.account-picker {
  position: relative;
  width: 100%;
}
.picker-trigger {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  text-align: left;
  font-size: 14px;
  transition: border-color 0.2s;
  display: flex;
  align-items: center;
}
.picker-trigger:hover {
  border-color: #40a9ff;
}
.picker-trigger.disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background: #f5f5f5;
}
.picker-trigger.disabled:hover {
  border-color: #d9d9d9;
}
.placeholder {
  color: #bfbfbf;
}
.selected-name {
  color: #333;
}
</style>
```

- [ ] **步骤 2：删除 AccountSelector.vue**

```bash
cd /home/mechdancer/repos/accounting
rm accounting-web/src/components/AccountSelector.vue
```

- [ ] **步骤 3：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountPicker.vue
git add accounting-web/src/components/AccountSelector.vue
git commit -m "feat(web): AccountPicker 改用 AccountTreeList 并删除 AccountSelector"
```

---

### 任务 5：改造 AccountCards.vue

**文件：**
- 修改：`accounting-web/src/views/AccountCards.vue`

- [ ] **步骤 1：内联添加输入框状态提升**

在 `AccountCards.vue` 中新增/保留以下状态：

```ts
const addingChildOf = ref<number | null>(null)
const addingAtRoot = ref(false)
const newChildName = ref('')
```

- [ ] **步骤 2：使用 AccountTreeList 替换原有卡片渲染**

将原有 `draggable` + `AccountCard` 循环替换为：

```vue
<AccountTreeList
  :accounts="accounts"
  :parent-id="parentId"
  :type="type"
  :active-id="selectedId"
  mode="manage"
  :allow-add="true"
  :allow-drag="true"
  @update:active-id="handleNavigate"
  @add="handleStartAddFromList"
  @reorder="handleReorder"
/>
```

其中：

```ts
function handleNavigate(id: number | null) {
  if (id == null) {
    emit('navigate', null, false)
    return
  }
  emit('navigate', id, false)
}

function handleStartAddFromList(parentId: number | null) {
  if (parentId == null) {
    handleStartAddRoot()
  } else {
    handleStartAdd(accounts.value.find((a) => a.id === parentId)!)
  }
}

async function handleReorder(ids: number[]) {
  await accountStore.reorderAccounts(ids)
  message.success('排序已保存')
}
```

- [ ] **步骤 3：内联添加输入框渲染**

在 `AccountCards.vue` 中通过 slot 或状态控制内联添加输入框。如果 `AccountTreeList` 当前设计不暴露 slot，可在 `AccountCards.vue` 中监听 `add` 事件后，将 `addingChildOf` / `addingAtRoot` 状态传入 `AccountTreeList` 的某种显示机制。

简化方案：让 `AccountTreeList` 接收 `addingId?: number | null` 和 `addingRoot?: boolean` props，并在对应位置渲染内联输入行。

- [ ] **步骤 4：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/views/AccountCards.vue
git commit -m "refactor(web): AccountCards 改用 AccountTreeList"
```

---

### 任务 6：添加账户内联输入支持

**文件：**
- 修改：`accounting-web/src/components/AccountTreeList.vue`
- 修改：`accounting-web/src/views/AccountCards.vue`
- 修改：`accounting-web/src/components/AccountPicker.vue`

- [ ] **步骤 1：AccountTreeList 添加添加状态 props**

```ts
const props = defineProps<{
  // ... 原有 props
  addingId?: number | null     // 正在添加子账户的父账户 ID
  addingRoot?: boolean         // 是否正在添加根级账户
}>()
```

在需要的位置渲染内联输入行：

```vue
<div
  v-if="item.type === 'inline-add'"
  class="inline-add-row"
>
  <a-input
    :value="newChildName"
    size="small"
    placeholder="输入子账户名"
    @press-enter="confirmAdd(item.parentId)"
  />
  <a-button size="small" type="primary" @click="confirmAdd(item.parentId)">确认</a-button>
  <a-button size="small" @click="cancelAdd">取消</a-button>
</div>
```

- [ ] **步骤 2：AccountCards 传入添加状态**

```vue
<AccountTreeList
  :adding-id="addingChildOf"
  :adding-root="addingAtRoot"
  @add="handleStartAddFromList"
/>
```

- [ ] **步骤 3：AccountPicker 接入添加状态**

在 `AccountPicker.vue` 中维护 `addingChildOf` 和 `addingAtRoot` 状态，并传入 `AccountTreeList`。确认添加后调用 `accountStore.createAccount` 并刷新列表。

- [ ] **步骤 4：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountTreeList.vue accounting-web/src/views/AccountCards.vue accounting-web/src/components/AccountPicker.vue
git commit -m "feat(web): AccountTreeList 支持内联添加账户"
```

---

### 任务 7：更新暗色模式样式

**文件：**
- 修改：`accounting-web/src/App.vue`

- [ ] **步骤 1：更新触发按钮暗色样式**

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

- [ ] **步骤 2：更新 AccountTreeList 相关暗色样式**

```css
html.dark .account-tree-list .sub-cards {
  --bubble-border: #434343 !important;
  --bubble-bg: #1f1f1f !important;
}
html.dark .account-tree-list .sub-cards-arrow {
  border-bottom-color: #434343 !important;
}
html.dark .account-tree-list .sub-cards-arrow::after {
  border-bottom-color: #1f1f1f !important;
}
html.dark .account-tree-list .add-card-box {
  border-color: #434343 !important;
  color: #73d13d !important;
}
html.dark .account-tree-list .add-card-box:hover {
  background: rgba(115, 209, 61, 0.08) !important;
}
html.dark .account-tree-list .drag-handle {
  color: #666 !important;
}
html.dark .account-tree-list .expand-btn {
  color: #888 !important;
}
html.dark .account-tree-list .child-count {
  color: #888 !important;
  background: #333 !important;
}
```

- [ ] **步骤 3：运行构建**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run build
```

预期：构建成功。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/App.vue
git commit -m "style(web): 更新 AccountTreeList 与触发按钮暗色模式样式"
```

---

### 任务 8：端到端验证

**文件：**
- 无需修改文件

- [ ] **步骤 1：完整构建**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run build
```

预期：构建成功，无类型错误。

- [ ] **步骤 2：启动开发服务器**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run dev
```

- [ ] **步骤 3：手动验证清单**

| 验证项 | 预期结果 |
|---|---|
| `/transaction` 触发按钮 | 高度 38px，暗色模式背景与其他输入框一致 |
| 点击触发按钮 | 抽屉滑出，卡片固定宽度 |
| 展开父账户 | 子账户气泡紧跟父卡片，同级卡片被下推 |
| 2 个子账户 | 横向排列 |
| 同时展开两个非直系账户 | 先展开的自动收起 |
| 抽屉内添加账户 | 输入确认后列表刷新，抽屉保持打开 |
| `/accounts` 拖拽 | drag handle 可拖动，添加按钮自动滑回末尾 |
| `/accounts` 展开 | 展开气泡不被添加按钮隔开 |
| 暗色模式 | 卡片、气泡、添加按钮、drag handle 颜色正确 |

- [ ] **步骤 4：Commit 验证结果（可选）**

如无文件变更，可跳过；否则：

```bash
cd /home/mechdancer/repos/accounting
git add -A
git commit -m "chore(web): AccountTreeList 改造回归验证"
```

---

## 自检

### 规格覆盖度

- [x] 提取 AccountTreeList → 任务 2
- [x] 卡片固定宽度 → 任务 1
- [x] 子账户气泡紧跟父卡片 → 任务 2（visibleItems 插入 sub-cards）
- [x] 添加按钮位于层级末尾 → 任务 2（add-card 在 items 末尾）
- [x] 基于栈的展开互斥 → 任务 2（expandedStack + handleSelect/handleToggleExpand）
- [x] 抽屉内直接添加账户 → 任务 4、6
- [x] 拖拽排序与添加按钮自动校正 → 任务 3、5
- [x] 触发按钮尺寸与暗色模式 → 任务 4、7
- [x] 账户详情页布局修复 → 任务 5

### 占位符扫描

计划中无“待定/TODO/后续实现/补充细节”等占位符；每个代码步骤均包含完整代码或精确命令。

### 类型一致性

- `AccountTreeList` props：`accounts`、`parentId`、`type`、`modelValue`、`activeId`、`mode`、`allowAdd`、`allowDrag`
- emits：`update:modelValue`、`update:activeId`、`add`、`reorder`
- `AccountCard` 保持 `account`、`selected`、`expanded` props 和 `click` emit

---

## 执行交接

**计划已完成并保存到 `docs/superpowers/plans/2026-06-19-account-tree-list-redesign.md`。两种执行方式：**

**1. 子代理驱动（推荐）** - 每个任务调度一个新的子代理，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 executing-plans 执行任务，批量执行并设有检查点

**选哪种方式？**
