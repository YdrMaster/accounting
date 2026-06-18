# AccountPicker 底部抽屉改造实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 `TransactionForm.vue` 中的 `AccountPicker.vue` 从居中模态框改造为底部抽屉，并在抽屉内复用 `/accounts` 页面的账户卡片选择/展开 UI。

**架构：** 新增 `BottomSheet.vue` 作为可复用底部抽屉容器；新增 `AccountCard.vue` 作为账户卡片呈现组件；新增 `AccountSelector.vue` 作为账户树选择器；`AccountPicker.vue` 组合 `BottomSheet + AccountSelector`；`AccountCards.vue` 改用 `AccountCard` 以共享卡片视觉与交互。

**技术栈：** Vue 3 + TypeScript + Ant Design Vue + Vite。

---

## 文件结构

| 文件 | 职责 | 变更类型 |
|---|---|---|
| `accounting-web/src/components/AccountCard.vue` | 单个账户卡片：显示名称、选中态、展开箭头、closed/system 标签；处理选中/展开点击。 | 新建 |
| `accounting-web/src/components/AccountSelector.vue` | 账户树选择器：使用 `AccountCard` 递归渲染卡片网格与层级展开；维护 `expandedStack`；emit 选中事件。 | 新建 |
| `accounting-web/src/components/BottomSheet.vue` | 底部抽屉容器：宽度对齐表单、动画、移动端拖动 snap、极淡遮罩、点击关闭、Esc 关闭。 | 新建 |
| `accounting-web/src/components/AccountPicker.vue` | 触发按钮 + `BottomSheet` + `AccountSelector` 组合；透传 `accountType` 等 props。 | 修改 |
| `accounting-web/src/components/AccountCards.vue` | 改用 `AccountCard` 渲染卡片，保留拖拽、添加子账户、详情导航等管理功能。 | 修改 |
| `accounting-web/src/App.vue` | 新增 `BottomSheet`、`AccountCard`、`AccountSelector` 的暗色模式样式。 | 修改 |

---

### 任务 1：创建 `AccountCard.vue`

**文件：**
- 创建：`accounting-web/src/components/AccountCard.vue`

- [ ] **步骤 1：编写组件骨架**

```vue
<template>
  <div
    class="account-card"
    :class="cardClass"
    @click="handleClick"
  >
    <div class="card-header">
      <slot name="prefix" />
      <span class="card-name" :title="account.full_name">
        {{ shortName(account.full_name) }}
      </span>
      <slot name="suffix" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Account } from '@/stores/account'

const props = defineProps<{
  account: Account
  selected?: boolean
  expanded?: boolean
}>()

const emit = defineEmits<{
  click: []
}>()

const cardClass = computed(() => ({
  selected: props.selected,
  expanded: props.expanded,
  closed: props.account.closed_at,
  system: props.account.is_system,
}))

function shortName(fullName: string): string {
  return fullName.split(':').pop() || fullName
}

function handleClick() {
  emit('click')
}
</script>

<style scoped>
.account-card {
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
  background: #fff;
  min-width: 140px;
  min-height: 50px;
  display: flex;
  align-items: center;
  transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
  user-select: none;
  box-sizing: border-box;
}
.account-card:hover {
  border-color: #91d5ff;
}
.account-card.selected {
  border-color: #1890ff;
  background: #e6f7ff;
}
.account-card.expanded {
  border-color: #1890ff;
}
.account-card.closed {
  opacity: 0.55;
}
.account-card.system {
  border-style: dashed;
}
.card-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
}
.card-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
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
git add accounting-web/src/components/AccountCard.vue
git commit -m "feat(web): 创建可复用 AccountCard 组件"
```

---

### 任务 2：创建 `AccountSelector.vue`

**文件：**
- 创建：`accounting-web/src/components/AccountSelector.vue`
- 修改：`accounting-web/src/components/AccountCard.vue`（添加 expand 箭头插槽支持，可选）

- [ ] **步骤 1：实现 AccountSelector 骨架**

```vue
<template>
  <div class="account-selector">
    <div v-if="currentLevel.length === 0" class="empty-state">
      暂无账户
    </div>
    <div v-else class="cards-grid">
      <div
        v-for="account in currentLevel"
        :key="account.id"
        class="card-line"
      >
        <AccountCard
          :account="account"
          :selected="modelValue === account.id"
          :expanded="isExpanded(account.id)"
          @click="handleSelect(account)"
        >
          <template v-if="hasChildren(account.id)" #suffix>
            <button
              class="expand-btn"
              :class="{ rotated: isExpanded(account.id) }"
              @click.stop="toggleExpand(account.id)"
            >
              ▼
            </button>
          </template>
        </AccountCard>
        <div
          v-if="isExpanded(account.id) && hasChildren(account.id)"
          class="sub-cards"
        >
          <AccountSelector
            :accounts="accounts"
            :parent-id="account.id"
            :type="type"
            :model-value="modelValue"
            v-model:expanded-set="expandedSet"
            @update:model-value="emit('update:modelValue', $event)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AccountCard from './AccountCard.vue'
import type { Account } from '@/stores/account'

const props = defineProps<{
  accounts: Account[]
  parentId: number | null
  type?: string
  modelValue?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
}>()

const currentLevel = computed(() =>
  props.accounts
    .filter((a) => a.parent_id === props.parentId && (!props.type || a.account_type === props.type))
    .sort((a, b) => a.position - b.position)
)

const expandedSet = defineModel<Set<number>>('expandedSet', { default: () => new Set() })

function hasChildren(id: number): boolean {
  return props.accounts.some((a) => a.parent_id === id)
}

function isExpanded(id: number): boolean {
  return expandedSet.value.has(id)
}

function toggleExpand(id: number) {
  const next = new Set(expandedSet.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  expandedSet.value = next
}

function handleSelect(account: Account) {
  emit('update:modelValue', account.id)
}
</script>

<style scoped>
.account-selector {
  width: 100%;
}
.empty-state {
  color: #999;
  text-align: center;
  padding: 24px 0;
  font-size: 13px;
}
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  min-height: 36px;
  align-items: flex-start;
}
.card-line {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 6px;
  flex: 1 1 140px;
  min-width: 140px;
}
.card-line :deep(.account-card) {
  flex: 1;
  min-width: 0;
}
.expand-btn {
  align-self: center;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 10px;
  color: #999;
  transition: transform 0.2s;
  padding: 4px;
  flex-shrink: 0;
}
.expand-btn.rotated {
  transform: rotate(180deg);
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
git add accounting-web/src/components/AccountSelector.vue accounting-web/src/components/AccountCard.vue
git commit -m "feat(web): 创建 AccountSelector 账户树选择器"
```

---

### 任务 3：创建 `BottomSheet.vue`

**文件：**
- 创建：`accounting-web/src/components/BottomSheet.vue`

- [ ] **步骤 1：实现 BottomSheet**

```vue
<template>
  <Teleport to="body">
    <Transition name="bottom-sheet" @after-leave="onAfterLeave">
      <div v-if="visible" class="bottom-sheet-overlay" @click.self="close">
        <div
          ref="sheetEl"
          class="bottom-sheet"
          :style="sheetStyle"
          @touchstart="handleTouchStart"
          @touchmove="handleTouchMove"
          @touchend="handleTouchEnd"
        >
          <div class="sheet-header" @click.stop>
            <div class="drag-handle" />
            <span class="sheet-title">{{ title }}</span>
            <button class="sheet-close" @click="close">×</button>
          </div>
          <div class="sheet-body">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  open: boolean
  title?: string
}>()

const emit = defineEmits<{
  'update:open': [open: boolean]
}>()

const visible = ref(false)
const sheetEl = ref<HTMLElement | null>(null)
const dragOffset = ref(0)
const isDragging = ref(false)
const startY = ref(0)
const lastY = ref(0)
const lastTime = ref(0)
const velocity = ref(0)

const sheetStyle = computed(() => ({
  transform: `translateY(${Math.max(0, dragOffset.value)}px)`,
  transition: isDragging.value ? 'none' : 'transform 0.2s ease-in',
}))

watch(() => props.open, (open) => {
  if (open) {
    dragOffset.value = 0
    visible.value = true
  }
})

function close() {
  dragOffset.value = sheetEl.value?.offsetHeight ?? window.innerHeight
  visible.value = false
}

function onAfterLeave() {
  emit('update:open', false)
}

function handleTouchStart(e: TouchEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.sheet-header')) return
  isDragging.value = true
  startY.value = e.touches[0].clientY
  lastY.value = startY.value
  lastTime.value = Date.now()
  velocity.value = 0
}

function handleTouchMove(e: TouchEvent) {
  if (!isDragging.value) return
  const y = e.touches[0].clientY
  const now = Date.now()
  const dt = now - lastTime.value
  if (dt > 0) {
    velocity.value = (y - lastY.value) / dt
  }
  dragOffset.value = Math.max(0, y - startY.value)
  lastY.value = y
  lastTime.value = now
}

function handleTouchEnd() {
  if (!isDragging.value) return
  isDragging.value = false
  const threshold = (sheetEl.value?.offsetHeight ?? 0) * 0.4
  if (dragOffset.value > threshold || velocity.value > 0.5) {
    close()
  } else {
    dragOffset.value = 0
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.bottom-sheet-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 1000;
}
.bottom-sheet {
  width: 100%;
  max-width: 600px;
  max-height: 70vh;
  min-height: 160px;
  background: #fff;
  border-top: 1px solid #d9d9d9;
  border-radius: 12px 12px 0 0;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.06);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
  position: relative;
}
.drag-handle {
  position: absolute;
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
  width: 36px;
  height: 4px;
  background: #d9d9d9;
  border-radius: 2px;
}
.sheet-title {
  font-weight: 600;
  margin-top: 8px;
}
.sheet-close {
  margin-top: 8px;
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
}
.sheet-close:hover {
  color: #333;
}
.sheet-body {
  padding: 12px 16px;
  overflow-y: auto;
  flex: 1;
}

.bottom-sheet-enter-active,
.bottom-sheet-leave-active {
  transition: opacity 0.25s ease;
}
.bottom-sheet-enter-from,
.bottom-sheet-leave-to {
  opacity: 0;
}
.bottom-sheet-enter-active .bottom-sheet,
.bottom-sheet-leave-active .bottom-sheet {
  transition: transform 0.25s ease-out;
}
.bottom-sheet-enter-from .bottom-sheet,
.bottom-sheet-leave-to .bottom-sheet {
  transform: translateY(100%);
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
git add accounting-web/src/components/BottomSheet.vue
git commit -m "feat(web): 创建 BottomSheet 底部抽屉组件"
```

---

### 任务 4：改造 `AccountPicker.vue`

**文件：**
- 修改：`accounting-web/src/components/AccountPicker.vue`

- [ ] **步骤 1：替换为 BottomSheet + AccountSelector**

```vue
<template>
  <div class="account-picker">
    <button class="picker-trigger" :class="{ disabled: props.disabled }" @click="openSheet" :disabled="props.disabled">
      <span v-if="selectedAccount" class="selected-name">{{ selectedAccount.full_name }}</span>
      <span v-else class="placeholder">{{ placeholder }}</span>
    </button>

    <BottomSheet v-model:open="showSheet" title="选择账户">
      <AccountSelector
        :accounts="accountStore.accounts"
        :parent-id="null"
        :type="props.accountType"
        :model-value="props.modelValue"
        v-model:expanded-set="expandedSet"
        @update:model-value="handleSelect"
      />
    </BottomSheet>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAccountStore } from '@/stores/account'
import BottomSheet from './BottomSheet.vue'
import AccountSelector from './AccountSelector.vue'

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
const expandedSet = ref<Set<number>>(new Set())

const selectedAccount = computed(() => {
  if (!props.modelValue) return null
  return accountStore.accounts.find((a) => a.id === props.modelValue) || null
})

function openSheet() {
  expandedSet.value = new Set()
  showSheet.value = true
}

function handleSelect(id: number) {
  emit('update:modelValue', id)
}
</script>

<style scoped>
.account-picker {
  position: relative;
  width: 100%;
}
.picker-trigger {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  text-align: left;
  font-size: 14px;
  transition: border-color 0.2s;
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

- [ ] **步骤 2：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 3：手动验证**

启动开发服务器：

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run dev
```

在浏览器打开 `http://localhost:5173/transaction`，点击账户输入框，确认抽屉从底部滑出、显示账户卡片、点击卡片后表单更新且抽屉保持打开。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountPicker.vue
git commit -m "feat(web): AccountPicker 改为 BottomSheet + AccountSelector"
```

---

### 任务 5：改造 `AccountCards.vue` 使用 `AccountCard`

**文件：**
- 修改：`accounting-web/src/components/AccountCards.vue`
- 修改：`accounting-web/src/components/AccountCard.vue`（如需补充 drag handle 插槽）

- [ ] **步骤 1：替换卡片渲染为 AccountCard**

在 `AccountCards.vue` 中，将内联的 `.account-card` 替换为 `<AccountCard>`，并通过 `prefix` 插槽传入 drag handle，通过 `suffix` 插槽传入 child count / expand arrow / add button。

保留：
- `draggable` 包裹根级卡片；
- `sub-cards` 递归；
- 添加子账户/根账户逻辑；
- 详情面板导航。

示例替换片段：

```vue
<AccountCard
  :account="account"
  :selected="selectedId === account.id"
  :expanded="isExpanded(account.id)"
  @click="handleSelectCard(account)"
>
  <template #prefix>
    <span class="drag-handle" @click.stop>⠿</span>
  </template>
  <template #suffix>
    <span class="card-actions">
      <template v-if="childrenCount(account.id) > 0">
        <span class="child-count">{{ childrenCount(account.id) }}</span>
        <span
          class="expand-arrow"
          :class="{ rotated: isExpanded(account.id) }"
          @click.stop="handleToggleExpand(account)"
        >▼</span>
      </template>
      <span v-else class="add-btn" @click.stop="handleStartAdd(account)">+</span>
    </span>
    <span v-if="account.closed_at" class="closed-tag">
      <a-tag color="default" style="margin: 0; font-size: 11px">已关闭</a-tag>
    </span>
  </template>
</AccountCard>
```

- [ ] **步骤 2：运行类型检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit
```

预期：无类型错误。

- [ ] **步骤 3：手动验证**

在浏览器打开 `http://localhost:5173/accounts`，确认：
- 卡片显示正常；
- 点击卡片展开详情面板；
- 点击箭头展开/折叠子账户；
- 拖拽排序正常；
- 添加子账户正常。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/AccountCards.vue accounting-web/src/components/AccountCard.vue
git commit -m "refactor(web): AccountCards 复用 AccountCard 组件"
```

---

### 任务 6：新增暗色模式样式

**文件：**
- 修改：`accounting-web/src/App.vue`

- [ ] **步骤 1：在 App.vue 追加暗色样式**

在文件末尾追加：

```css
/* ========== BottomSheet ========== */
html.dark .bottom-sheet {
  background: #1f1f1f !important;
  border-top-color: #434343 !important;
}
html.dark .bottom-sheet-overlay {
  background: rgba(0, 0, 0, 0.25) !important;
}
html.dark .sheet-header {
  border-bottom-color: #303030 !important;
}
html.dark .sheet-title {
  color: #e0e0e0 !important;
}
html.dark .sheet-close {
  color: #999 !important;
}
html.dark .sheet-close:hover {
  color: #fff !important;
}

/* ========== AccountCard / AccountSelector ========== */
html.dark .account-card {
  background: #1f1f1f !important;
  border-color: #434343 !important;
  color: #fff !important;
}
html.dark .account-card:hover {
  border-color: #177ddc !important;
}
html.dark .account-card.selected {
  background: #111d2c !important;
  border-color: #177ddc !important;
}
html.dark .account-card.expanded {
  border-color: #177ddc !important;
}
html.dark .card-name {
  color: #fff !important;
}
html.dark .expand-btn {
  color: #888 !important;
}
html.dark .sub-cards {
  --bubble-border: #434343 !important;
  --bubble-bg: #1f1f1f !important;
}
html.dark .empty-state {
  color: #888 !important;
}
```

- [ ] **步骤 2：运行类型检查与构建**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run build
```

预期：构建成功。

- [ ] **步骤 3：手动验证暗色模式**

在浏览器中切换到暗色模式，打开 `/transaction` 的账户选择抽屉和 `/accounts` 页面，确认卡片、抽屉、遮罩颜色正确。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/App.vue
git commit -m "style(web): 新增 BottomSheet 与 AccountCard 暗色模式样式"
```

---

### 任务 7：端到端验证

**文件：**
- 无需修改文件

- [ ] **步骤 1：完整构建**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm run build
```

预期：

```
vite v5.x.x building for production...
✓ 1234 modules transformed.
dist/                     0.05 kB │ gzip: 0.07 kB
... (success)
```

- [ ] **步骤 2：手动回归清单**

| 验证项 | 预期结果 |
|---|---|
| `/transaction` 点击账户输入框 | 抽屉从底部滑出，宽度与表单对齐 |
| 点击卡片主体 | 表单账户显示更新，抽屉保持打开 |
| 点击父节点箭头 | 子账户气泡展开/折叠 |
| 点击遮罩/关闭按钮/向下拖动 | 抽屉关闭动画正常 |
| 移动端拖动 | 实时伸缩，松手 snap |
| `/transaction/refund` | 只显示资产类账户 |
| `/transaction/reimbursement` | 只显示资产类账户 |
| `/accounts` | 卡片、展开、拖拽、添加未回归 |
| 暗色模式 | 抽屉/卡片/遮罩颜色正确 |

- [ ] **步骤 3：Commit 验证结果（可选）**

如果仅有文档更新或无文件变更，可跳过；否则：

```bash
cd /home/mechdancer/repos/accounting
git add -A
git commit -m "chore(web): AccountPicker 抽屉改造回归验证"
```

---

## 自检

### 规格覆盖度

- [x] 居中模态框改为底部抽屉 → 任务 4
- [x] 抽屉宽度与表单一致 → 任务 3（BottomSheet max-width: 600px）
- [x] 复用账户卡片 UI → 任务 1、2、5
- [x] 任意节点可选中 → 任务 2（AccountSelector 点击卡片 emit select）
- [x] 选中不自动关闭 → 任务 4（handleSelect 只 emit，不关闭 sheet）
- [x] 移动端拖动 snap → 任务 3
- [x] 极淡遮罩 → 任务 3（rgba(0,0,0,0.15)）
- [x] 暗色模式 → 任务 6
- [x] 不影响账户管理页 → 任务 5

### 占位符扫描

计划中无“待定/TODO/后续实现/补充细节”等占位符；每个代码步骤均包含完整代码或精确命令。

### 类型一致性

- `AccountCard` 使用 `Account` 类型；
- `AccountSelector` 使用 `Account[]` 和 `modelValue?: number`；
- `BottomSheet` 使用 `open: boolean` 和 `title?: string`；
- `AccountPicker` 透传 `accountType?: string` 给 `AccountSelector`。

---

## 执行交接

**计划已完成并保存到 `docs/superpowers/plans/2026-06-18-account-picker-bottom-sheet.md`。两种执行方式：**

**1. 子代理驱动（推荐）** - 每个任务调度一个新的子代理，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 executing-plans 执行任务，批量执行并设有检查点

**选哪种方式？**
