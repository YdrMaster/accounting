# 账户视图扁平网格化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `accounting-web` 的账户视图从递归嵌套卡片树重构为遵循 `compile` 算法的响应式扁平网格。

**Architecture:** 新增纯函数 `compileRows` 负责按固定列数生成扁平行；新增 `AccountCard`、`AccountRowGroup`、`AccountGrid` 展示组件；`AccountsView` 使用 `ResizeObserver` 读取容器查询列数并驱动行计算；最终删除递归组件 `AccountNode`。

**Tech Stack:** Vue 3.6, TypeScript, Pinia, Vitest, @vue/test-utils, @vueuse/core, CSS container queries

## Global Constraints

- 严格遵循 `docs/superpowers/specs/2026-07-09-account-view-grid-design.md`。
- 列数响应式：默认 2 列，容器宽度 ≥ 600px 为 3 列，≥ 900px 为 4 列。
- 展开模型为单路径：`expandedPath: number[]`。
- 空位必须生成占位 `GridItem`，保证网格对齐。
- 所有新代码必须带测试；测试使用 Vitest + happy-dom。
- 不要引入新依赖，项目已有 `@vueuse/core` 与 `@vue/test-utils`。

---

## File Structure

| 文件 | 责任 |
|------|------|
| `src/utils/accountGrid.ts` | 纯函数 `compileRows`、`buildRowTree`、类型定义 `GridItem` / `GridRow` / `RowNode`。 |
| `src/utils/__tests__/accountGrid.spec.ts` | `compileRows` 与 `buildRowTree` 的单元测试。 |
| `src/composables/useGridColumns.ts` | 读取容器查询 `--grid-columns` 并响应式返回当前列数。 |
| `src/composables/__tests__/useGridColumns.spec.ts` | `useGridColumns` 测试（mock `ResizeObserver` / `getComputedStyle`）。 |
| `src/components/layout/AccountCard.vue` | 单张账户卡片（含占位状态）。 |
| `src/components/layout/__tests__/AccountCard.spec.ts` | `AccountCard` 渲染与点击测试。 |
| `src/components/layout/AccountRowGroup.vue` | 递归渲染一行及其子账户行容器。 |
| `src/components/layout/__tests__/AccountRowGroup.spec.ts` | `AccountRowGroup` 递归渲染测试。 |
| `src/components/layout/AccountGrid.vue` | 按类型渲染整个网格，内部使用容器查询检测列数并通过 `columnsChange` 通知父组件。 |
| `src/components/layout/__tests__/AccountGrid.spec.ts` | `AccountGrid` 容器查询与行渲染测试。 |
| `src/views/AccountsView.vue` | 整合 store、expandedPath、列数检测、抽屉。 |
| `src/views/__tests__/AccountsView.spec.ts` | `AccountsView`  smoke 测试（mock store）。 |
| `src/components/layout/AccountNode.vue` | 删除（递归嵌套逻辑被取代）。 |

---

### Task 1: 实现 `compileRows` 与 `buildRowTree`

**Files:**
- Create: `src/utils/accountGrid.ts`
- Test: `src/utils/__tests__/accountGrid.spec.ts`

**Interfaces:**
- Consumes: `AccountDto[]`, `expandedPath: number[]`, `columns: number`, `getChildren(id: number) => AccountDto[]`
- Produces: `GridItem`, `GridRow`（新增 `parentRowIndex`）, `RowNode`, `compileRows`, `buildRowTree`

- [ ] **Step 1: 编写失败测试**

在 `src/utils/__tests__/accountGrid.spec.ts` 中：

```ts
import { describe, expect, it } from 'vitest'
import type { AccountDto } from '../../types/api'
import { buildRowTree, compileRows } from '../accountGrid'

function makeAccount(id: number, parentId: number | null = null, name = `acc-${id}`): AccountDto {
  return {
    id,
    name,
    account_type: 'Asset',
    parent_id: parentId,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

describe('compileRows', () => {
  it('renders roots left-aligned with trailing placeholders when no expansion', () => {
    const roots = [makeAccount(1), makeAccount(2)]
    const rows = compileRows(roots, [], 3, () => [])
    expect(rows).toHaveLength(1)
    expect(rows[0].items).toHaveLength(3)
    expect(rows[0].items[0].account?.id).toBe(1)
    expect(rows[0].items[1].account?.id).toBe(2)
    expect(rows[0].items[2].isPlaceholder).toBe(true)
    expect(rows[0].items[2].hasChildren).toBe(false)
  })

  it('expands a root and renders its children on the next row', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(4, 2), makeAccount(5, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    expect(rows).toHaveLength(2)
    expect(rows[0].expandedAccountId).toBe(2)
    expect(rows[1].items[0].account?.id).toBe(4)
    expect(rows[1].items[1].account?.id).toBe(5)
    expect(rows[1].items[2].isPlaceholder).toBe(true)
  })

  it('renders deeper paths and respects depth-first order', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3)]
    const children: Record<number, AccountDto[]> = {
      3: [makeAccount(4, 3), makeAccount(5, 3)],
      5: [makeAccount(6, 5)],
    }
    const rows = compileRows(roots, [3, 5], 3, id => children[id] ?? [])
    expect(rows.map(r => r.items.map(i => i.account?.id ?? null))).toEqual([
      [1, 2, 3],
      [4, 5, null],
      [6, null, null],
    ])
  })

  it('finishes a subtree before rendering remaining siblings', () => {
    const roots = [makeAccount(1), makeAccount(2), makeAccount(3), makeAccount(4)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(5, 2), makeAccount(6, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    expect(rows.map(r => r.items.map(i => i.account?.id ?? null))).toEqual([
      [1, 2, 3],
      [5, 6, null],
      [4, null, null],
    ])
  })
})

describe('buildRowTree', () => {
  it('groups child rows under their parent row', () => {
    const roots = [makeAccount(1), makeAccount(2)]
    const children: Record<number, AccountDto[]> = { 2: [makeAccount(3, 2)] }
    const rows = compileRows(roots, [2], 3, id => children[id] ?? [])
    const tree = buildRowTree(rows)
    expect(tree).toHaveLength(1)
    expect(tree[0].row.items[0].account?.id).toBe(1)
    expect(tree[0].children).toHaveLength(1)
    expect(tree[0].children[0].row.items[1].account?.id).toBe(2)
    expect(tree[0].children[0].children).toHaveLength(1)
    expect(tree[0].children[0].children[0].row.items[0].account?.id).toBe(3)
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/utils/__tests__/accountGrid.spec.ts
```

Expected: FAIL with `Error: Cannot find module '../accountGrid'` or similar。

- [ ] **Step 3: 实现最小代码**

创建 `src/utils/accountGrid.ts`：

```ts
import type { AccountDto } from '../types/api'

export interface GridItem {
  account: AccountDto | null
  isPlaceholder: boolean
  hasChildren: boolean
}

export interface GridRow {
  items: GridItem[]
  depth: number
  expandedIndex: number | null
  expandedAccountId: number | null
  parentRowIndex: number | null
}

export interface RowNode {
  row: GridRow
  children: RowNode[]
}

export function compileRows(
  roots: AccountDto[],
  expandedPath: number[],
  columns: number,
  getChildren: (id: number) => AccountDto[],
): GridRow[] {
  const rows: GridRow[] = []
  const pathSet = new Set(expandedPath)

  type Level = { accounts: AccountDto[]; index: number }
  const levels: Level[] = [{ accounts: roots, index: 0 }]
  const parentStack: (number | null)[] = [null]
  let depth = 0

  while (true) {
    const level = levels[depth]

    if (level.index >= level.accounts.length) {
      if (depth === 0) break
      depth--
      continue
    }

    const rowItems: GridItem[] = []
    let expandedIndex: number | null = null
    let expandedAccountId: number | null = null

    for (let i = 0; i < columns; i++) {
      const account = level.accounts[level.index]
      if (!account) {
        for (let j = i; j < columns; j++) {
          rowItems.push({ account: null, isPlaceholder: true, hasChildren: false })
        }
        break
      }

      const accountHasChildren = getChildren(account.id).length > 0
      rowItems.push({ account, isPlaceholder: false, hasChildren: accountHasChildren })
      level.index++

      if (pathSet.has(account.id) && accountHasChildren) {
        expandedIndex = i
        expandedAccountId = account.id
        depth++
        levels[depth] = { accounts: getChildren(account.id), index: 0 }
        parentStack[depth] = rows.length
      }
    }

    rows.push({
      items: rowItems,
      depth,
      expandedIndex,
      expandedAccountId,
      parentRowIndex: parentStack[depth],
    })

    if (expandedAccountId !== null) {
      continue
    }
  }

  return rows
}

export function buildRowTree(rows: GridRow[]): RowNode[] {
  const roots: RowNode[] = []
  const stack: RowNode[] = []

  for (const row of rows) {
    const node: RowNode = { row, children: [] }

    while (stack.length > 0 && stack[stack.length - 1].row.depth >= row.depth) {
      stack.pop()
    }

    if (stack.length === 0) {
      roots.push(node)
    } else {
      stack[stack.length - 1].children.push(node)
    }

    stack.push(node)
  }

  return roots
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/utils/__tests__/accountGrid.spec.ts
```

Expected: 所有测试通过。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/utils/accountGrid.ts accounting-web/src/utils/__tests__/accountGrid.spec.ts
git commit -m "feat(account-grid): add compileRows and buildRowTree utilities"
```

---

### Task 2: 实现 `useGridColumns`

**Files:**
- Create: `src/composables/useGridColumns.ts`
- Test: `src/composables/__tests__/useGridColumns.spec.ts`

**Interfaces:**
- Consumes: `Ref<HTMLElement | undefined>`
- Produces: `{ columns: Ref<number> }`

- [ ] **Step 1: 编写失败测试**

创建 `src/composables/__tests__/useGridColumns.spec.ts`：

```ts
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, ref } from 'vue'
import { useGridColumns } from '../useGridColumns'

describe('useGridColumns', () => {
  it('reads --grid-columns from the element', async () => {
    const TestComp = defineComponent({
      setup() {
        const el = ref<HTMLElement>()
        const { columns } = useGridColumns(el)
        return { el, columns }
      },
      template: '<div ref="el" style="--grid-columns: 3;"></div>',
    })

    const wrapper = mount(TestComp)
    await wrapper.vm.$nextTick()
    expect(wrapper.vm.columns).toBe(3)
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/composables/__tests__/useGridColumns.spec.ts
```

Expected: FAIL，模块不存在。

- [ ] **Step 3: 实现最小代码**

创建 `src/composables/useGridColumns.ts`：

```ts
import { useResizeObserver } from '@vueuse/core'
import { ref, watch, type Ref } from 'vue'

export function useGridColumns(gridRef: Ref<HTMLElement | undefined>) {
  const columns = ref(2)

  function update() {
    const el = gridRef.value
    if (!el) return
    const raw = getComputedStyle(el).getPropertyValue('--grid-columns')
    const value = parseInt(raw.trim(), 10)
    columns.value = Number.isNaN(value) ? 2 : value
  }

  useResizeObserver(gridRef, update)
  watch(() => gridRef.value, update, { immediate: true })

  return { columns }
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/composables/__tests__/useGridColumns.spec.ts
```

Expected: PASS。如果 happy-dom 的 `getComputedStyle` 不返回自定义属性，在测试中给 `el` 设置 inline style 后手动读取验证。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/composables/useGridColumns.ts accounting-web/src/composables/__tests__/useGridColumns.spec.ts
git commit -m "feat(account-grid): add useGridColumns composable"
```

---

### Task 3: 实现 `AccountCard.vue`

**Files:**
- Create: `src/components/layout/AccountCard.vue`
- Test: `src/components/layout/__tests__/AccountCard.spec.ts`

**Interfaces:**
- Consumes: `GridItem`, `isExpanded?: boolean`, `isSelected?: boolean`
- Produces: `click` 事件（仅当 account 非空时）

注意：`GridItem.hasChildren` 已由 `compileRows` 计算，组件直接读取，不需要额外 prop。

- [ ] **Step 1: 编写失败测试**

创建 `src/components/layout/__tests__/AccountCard.spec.ts`：

```ts
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import AccountCard from '../AccountCard.vue'

const account = {
  id: 1,
  name: 'Test',
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: false,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
}

describe('AccountCard', () => {
  it('renders account name', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false } },
    })
    expect(wrapper.text()).toContain('Test')
  })

  it('renders placeholder as empty and non-clickable', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account: null, isPlaceholder: true, hasChildren: false } },
    })
    expect(wrapper.text()).toBe('')
    expect(wrapper.find('.account-card').classes()).toContain('placeholder')
  })

  it('emits click with account id', async () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false } },
    })
    await wrapper.find('.account-card').trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.emitted('click')![0]).toEqual([account])
  })

  it('shows expand indicator when expanded and item has children', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: true }, isExpanded: true },
    })
    expect(wrapper.text()).toContain('▾')
  })

  it('applies selected class', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false }, isSelected: true },
    })
    expect(wrapper.find('.account-card').classes()).toContain('selected')
  })

  it('applies closed class', () => {
    const closed = { ...account, closed_at: '2024-01-01' }
    const wrapper = mount(AccountCard, {
      props: { item: { account: closed, isPlaceholder: false, hasChildren: false } },
    })
    expect(wrapper.find('.account-card').classes()).toContain('closed')
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountCard.spec.ts
```

Expected: FAIL，模块不存在。

- [ ] **Step 3: 实现最小代码**

创建 `src/components/layout/AccountCard.vue`：

```vue
<script setup lang="ts">
import type { GridItem } from '../../utils/accountGrid'

const props = defineProps<{
  item: GridItem
  isExpanded?: boolean
  isSelected?: boolean
}>()

const emit = defineEmits<{
  click: [account: NonNullable<GridItem['account']>]
}>()

function onClick() {
  if (!props.item.isPlaceholder && props.item.account) {
    emit('click', props.item.account)
  }
}
</script>

<template>
  <div
    class="account-card"
    :class="{
      placeholder: item.isPlaceholder,
      selected: !item.isPlaceholder && isSelected,
      closed: !item.isPlaceholder && item.account.closed_at !== null,
    }"
    @click="onClick"
  >
    <template v-if="!item.isPlaceholder && item.account">
      <span v-if="item.hasChildren" class="expand-indicator">
        {{ isExpanded ? '▾' : '▸' }}
      </span>
      <span class="card-name">{{ item.account.name }}</span>
    </template>
  </div>
</template>

<style scoped>
.account-card {
  min-height: 3rem;
  padding: 0.75rem 0.5rem;
  background: var(--card-bg-alt);
  border-radius: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  overflow: hidden;
  transition: background 0.15s;
}

.account-card:hover {
  background: var(--card-bg);
}

.account-card.placeholder {
  background: transparent;
  cursor: default;
  pointer-events: none;
}

.account-card.selected {
  background: var(--accent);
  color: #fff;
}

.account-card.closed {
  opacity: 0.5;
  text-decoration: line-through;
}

.expand-indicator {
  font-size: 0.625rem;
  color: var(--text-muted);
  flex-shrink: 0;
}

.account-card.selected .expand-indicator {
  color: #fff;
}

.card-name {
  font-size: 0.8125rem;
  color: var(--text-heading);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.account-card.selected .card-name {
  color: #fff;
}
</style>
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountCard.spec.ts
```

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/layout/AccountCard.vue accounting-web/src/components/layout/__tests__/AccountCard.spec.ts
git commit -m "feat(account-grid): add AccountCard component"
```

---

### Task 4: 实现 `AccountRowGroup.vue`

**Files:**
- Create: `src/components/layout/AccountRowGroup.vue`
- Test: `src/components/layout/__tests__/AccountRowGroup.spec.ts`

**Interfaces:**
- Consumes: `RowNode`, `selectedAccountId: number | null`
- Produces: `click` 事件向上冒泡

- [ ] **Step 1: 编写失败测试**

创建 `src/components/layout/__tests__/AccountRowGroup.spec.ts`：

```ts
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import type { RowNode } from '../../utils/accountGrid'
import AccountRowGroup from '../AccountRowGroup.vue'

function account(id: number, name: string) {
  return {
    id,
    name,
    account_type: 'Asset',
    parent_id: null,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

describe('AccountRowGroup', () => {
  it('renders a row of cards', () => {
    const node: RowNode = {
      row: {
        items: [
          { account: account(1, 'A'), isPlaceholder: false, hasChildren: false },
          { account: account(2, 'B'), isPlaceholder: false, hasChildren: false },
          { account: null, isPlaceholder: true, hasChildren: false },
        ],
        depth: 0,
        expandedIndex: null,
        expandedAccountId: null,
        parentRowIndex: null,
      },
      children: [],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    expect(wrapper.text()).toContain('A')
    expect(wrapper.text()).toContain('B')
  })

  it('wraps child rows in a children container', () => {
    const node: RowNode = {
      row: {
        items: [{ account: account(1, 'A'), isPlaceholder: false, hasChildren: false }],
        depth: 0,
        expandedIndex: 0,
        expandedAccountId: 1,
        parentRowIndex: null,
      },
      children: [
        {
          row: {
            items: [{ account: account(2, 'B'), isPlaceholder: false, hasChildren: false }],
            depth: 1,
            expandedIndex: null,
            expandedAccountId: null,
            parentRowIndex: 0,
          },
          children: [],
        },
      ],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    expect(wrapper.find('.children-container').exists()).toBe(true)
    expect(wrapper.text()).toContain('B')
  })

  it('bubbles click events from nested cards', async () => {
    const acc = account(1, 'A')
    const node: RowNode = {
      row: {
        items: [{ account: acc, isPlaceholder: false, hasChildren: false }],
        depth: 0,
        expandedIndex: null,
        expandedAccountId: null,
        parentRowIndex: null,
      },
      children: [],
    }
    const wrapper = mount(AccountRowGroup, {
      props: { node, selectedAccountId: null },
    })
    await wrapper.find('.account-card').trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.emitted('click')![0]).toEqual([acc])
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountRowGroup.spec.ts
```

Expected: FAIL，模块不存在。

- [ ] **Step 3: 实现最小代码**

创建 `src/components/layout/AccountRowGroup.vue`：

```vue
<script setup lang="ts">
import type { RowNode } from '../../utils/accountGrid'
import AccountCard from './AccountCard.vue'

const props = defineProps<{
  node: RowNode
  selectedAccountId: number | null
}>()

const emit = defineEmits<{
  click: [account: NonNullable<RowNode['row']['items'][number]['account']>]
}>()
</script>

<template>
  <div class="row" :class="{ 'child-row': node.row.depth > 0 }">
    <AccountCard
      v-for="(item, index) in node.row.items"
      :key="item.account?.id ?? `placeholder-${node.row.depth}-${index}`"
      :item="item"
      :is-selected="!!item.account && item.account.id === selectedAccountId"
      :is-expanded="node.row.expandedAccountId === item.account?.id"
      @click="emit('click', $event)"
    />
  </div>
  <div v-if="node.children.length" class="children-container">
    <AccountRowGroup
      v-for="(child, index) in node.children"
      :key="index"
      :node="child"
      :selected-account-id="selectedAccountId"
      @click="emit('click', $event)"
    />
  </div>
</template>

<style scoped>
.row {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), 1fr);
  gap: 0.5rem;
}

.children-container {
  box-shadow: -2px 0 0 var(--accent);
  background: rgba(100, 108, 255, 0.05);
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;
}
</style>
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountRowGroup.spec.ts
```

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/layout/AccountRowGroup.vue accounting-web/src/components/layout/__tests__/AccountRowGroup.spec.ts
git commit -m "feat(account-grid): add recursive AccountRowGroup component"
```

---

### Task 5: 实现 `AccountGrid.vue`

**Files:**
- Create: `src/components/layout/AccountGrid.vue`
- Test: `src/components/layout/__tests__/AccountGrid.spec.ts`

**Interfaces:**
- Consumes: `typeLabel: string`, `rows: GridRow[]`, `selectedAccountId: number | null`
- Produces: `click` 事件、`columnsChange: [columns: number]` 事件

- [ ] **Step 1: 编写失败测试**

创建 `src/components/layout/__tests__/AccountGrid.spec.ts`：

```ts
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import AccountGrid from '../AccountGrid.vue'

vi.mock('../../composables/useGridColumns', () => ({
  useGridColumns: () => ({ columns: ref(2) }),
}))

const account = (id: number, name: string) => ({
  id,
  name,
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: false,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
})

describe('AccountGrid', () => {
  it('renders type label and rows', () => {
    const wrapper = mount(AccountGrid, {
      props: {
        typeLabel: '资产',
        rows: [
          {
            items: [
              { account: account(1, 'A'), isPlaceholder: false, hasChildren: false },
              { account: null, isPlaceholder: true, hasChildren: false },
            ],
            depth: 0,
            expandedIndex: null,
            expandedAccountId: null,
            parentRowIndex: null,
          },
        ],
        selectedAccountId: null,
      },
    })
    expect(wrapper.text()).toContain('资产')
    expect(wrapper.text()).toContain('A')
  })

  it('emits columnsChange when columns change', async () => {
    const wrapper = mount(AccountGrid, {
      props: {
        typeLabel: '资产',
        rows: [],
        selectedAccountId: null,
      },
    })
    await wrapper.vm.$nextTick()
    expect(wrapper.emitted('columnsChange')?.[0]).toEqual([2])
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountGrid.spec.ts
```

Expected: FAIL，模块不存在。

- [ ] **Step 3: 实现最小代码**

创建 `src/components/layout/AccountGrid.vue`：

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { AccountDto } from '../../types/api'
import type { GridRow } from '../../utils/accountGrid'
import { buildRowTree } from '../../utils/accountGrid'
import { useGridColumns } from '../../composables/useGridColumns'
import AccountRowGroup from './AccountRowGroup.vue'

const props = defineProps<{
  typeLabel: string
  rows: GridRow[]
  selectedAccountId: number | null
}>()

const emit = defineEmits<{
  click: [account: AccountDto]
  columnsChange: [columns: number]
}>()

const tree = computed(() => buildRowTree(props.rows))
const gridRef = ref<HTMLElement | null>(null)
const { columns } = useGridColumns(gridRef)

watch(columns, value => {
  emit('columnsChange', value)
}, { immediate: true })
</script>

<template>
  <div class="type-section">
    <h3 class="type-label">{{ typeLabel }}</h3>
    <div ref="gridRef" class="account-grid">
      <AccountRowGroup
        v-for="(node, index) in tree"
        :key="index"
        :node="node"
        :selected-account-id="selectedAccountId"
        @click="emit('click', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.type-section {
  container-type: inline-size;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.type-label {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-muted);
  font-weight: 500;
}

.account-grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 2), 1fr);
  gap: 0.5rem;
}

@container (min-width: 600px) {
  .account-grid {
    --grid-columns: 3;
  }
}

@container (min-width: 900px) {
  .account-grid {
    --grid-columns: 4;
  }
}
</style>
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/components/layout/__tests__/AccountGrid.spec.ts
```

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/layout/AccountGrid.vue accounting-web/src/components/layout/__tests__/AccountGrid.spec.ts
git commit -m "feat(account-grid): add AccountGrid with container queries"
```

---

### Task 6: 重构 `AccountsView.vue`

**Files:**
- Modify: `src/views/AccountsView.vue`
- Test: `src/views/__tests__/AccountsView.spec.ts`

**Interfaces:**
- Consumes: `useAccountStore`, `useGridColumns`, `AccountGrid`, `AccountDrawer`
- Produces: 渲染按类型分组的 `AccountGrid`，处理点击、抽屉、删除。

- [ ] **Step 1: 编写失败测试**

创建 `src/views/__tests__/AccountsView.spec.ts`：

```ts
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import AccountsView from '../AccountsView.vue'

vi.mock('../../components/layout/AccountDrawer.vue', () => ({
  default: {
    name: 'AccountDrawer',
    template: '<div data-testid="drawer">Drawer</div>',
  },
}))

vi.mock('../../components/layout/AccountGrid.vue', () => ({
  default: {
    name: 'AccountGrid',
    props: ['typeLabel', 'rows', 'selectedAccountId'],
    template: '<div data-testid="grid">{{ typeLabel }}</div>',
  },
}))

describe('AccountsView', () => {
  it('renders a grid for each account type', async () => {
    setActivePinia(createPinia())
    const wrapper = mount(AccountsView)
    await nextTick()
    const grids = wrapper.findAll('[data-testid="grid"]')
    expect(grids.length).toBeGreaterThan(0)
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/views/__tests__/AccountsView.spec.ts
```

Expected: FAIL（目录或模块问题）。

- [ ] **Step 3: 实现最小代码**

修改 `src/views/AccountsView.vue` 为：

```vue
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAccountStore } from '../stores/account'
import type { AccountDto } from '../types/api'
import AccountDrawer from '../components/layout/AccountDrawer.vue'
import AccountGrid from '../components/layout/AccountGrid.vue'
import { compileRows, type GridRow } from '../utils/accountGrid'

const store = useAccountStore()

onMounted(() => {
  if (store.accounts.length === 0) {
    store.loadAccounts()
  }
})

const typeLabels: Record<string, string> = {
  Asset: '资产',
  Income: '收入',
  Expense: '支出',
  Equity: '权益',
}

const typeOrder = ['Asset', 'Income', 'Expense', 'Equity'] as const

function getChildrenOfType(type: string): AccountDto[] {
  const roots = store.groupedAccounts.get(type) ?? []
  const children: AccountDto[] = []
  for (const root of roots) {
    children.push(...store.getChildren(root.id))
  }
  return children
}

const expandedPath = ref<number[]>([])
const selectedAccountId = ref<number | null>(null)
const drawerVisible = ref(false)
const columnsByType = ref<Record<string, number>>({})

function onColumnsChange(type: string, columns: number) {
  columnsByType.value[type] = columns
}

function isRootAccount(account: AccountDto): boolean {
  return account.parent_id === null
}

function isDescendantOf(accountId: number, ancestorId: number): boolean {
  return store.isDescendant(accountId, ancestorId)
}

function handleAccountClick(account: AccountDto) {
  const clickedId = account.id

  if (selectedAccountId.value === clickedId) {
    if (!account.is_system && !isRootAccount(account)) {
      drawerVisible.value = true
    }
    return
  }

  selectedAccountId.value = clickedId

  const hasChildren = store.getChildren(clickedId).length > 0
  if (hasChildren) {
    const newPath: number[] = []
    for (const id of expandedPath.value) {
      if (isDescendantOf(clickedId, id)) {
        newPath.push(id)
      }
    }
    newPath.push(clickedId)
    expandedPath.value = newPath
  }

  if (!account.is_system && !isRootAccount(account)) {
    drawerVisible.value = true
  }
}

function rowsForType(type: string): GridRow[] {
  const roots = getChildrenOfType(type)
  const columns = columnsByType.value[type] ?? 2
  return compileRows(roots, expandedPath.value, columns, id => store.getChildren(id))
}

const selectedAccount = computed(() => {
  if (selectedAccountId.value === null) return null
  return store.accounts.find(a => a.id === selectedAccountId.value) ?? null
})

function onDrawerClosed() {
  drawerVisible.value = false
}

function onAccountUpdated(updated: AccountDto) {
  store.refreshAccount(updated)
}

function onAccountDeleted(id: number) {
  store.removeAccount(id)
  expandedPath.value = expandedPath.value.filter(x => x !== id)
  if (selectedAccountId.value === id) {
    selectedAccountId.value = null
    drawerVisible.value = false
  }
}
</script>

<template>
  <div class="accounts">
    <div v-if="store.loading" class="loading">加载中...</div>
    <div v-else-if="store.error" class="error">{{ store.error }}</div>

    <template v-else>
      <div class="card-area">
        <AccountGrid
          v-for="type in typeOrder"
          :key="type"
          :type-label="typeLabels[type]"
          :rows="rowsForType(type)"
          :selected-account-id="selectedAccountId"
          @click="handleAccountClick"
          @columns-change="columns => onColumnsChange(type, columns)"
        />
      </div>
    </template>

    <AccountDrawer
      v-if="drawerVisible && selectedAccount"
      :account="selectedAccount"
      @close="onDrawerClosed"
      @updated="onAccountUpdated"
      @deleted="onAccountDeleted"
    />
  </div>
</template>

<style scoped>
.accounts {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.card-area {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  padding: 1rem 0.5rem;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
```

注意：上面用 `:ref="el => ..."` 的动态 ref 在 Vue 3.6 中有效；如果类型推断有问题，改用 `shallowRef<Record<string, any>>({})` 并在模板里用 `:ref="el => setGridRef(type, el)"`。

- [ ] **Step 4: 运行测试确认通过**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run src/views/__tests__/AccountsView.spec.ts
```

Expected: PASS。如果动态 ref 导致警告，先修复再运行。

- [ ] **Step 5: 提交**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/views/AccountsView.vue accounting-web/src/views/__tests__/AccountsView.spec.ts
git commit -m "feat(account-grid): refactor AccountsView to use flat grid"
```

---

### Task 7: 删除 `AccountNode.vue` 并清理引用

**Files:**
- Delete: `src/components/layout/AccountNode.vue`

- [ ] **Step 1: 检查引用**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
grep -r "AccountNode" src/
```

Expected: 仅 `src/components/layout/AccountNode.vue` 自身；`AccountsView` 已移除引用。

- [ ] **Step 2: 删除文件**

```bash
rm /home/mechdancer/repos/accounting/accounting-web/src/components/layout/AccountNode.vue
```

- [ ] **Step 3: 运行测试确认未破坏现有功能**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npx vitest run
npx eslint src/
```

Expected: 所有测试通过，无 lint 错误。

- [ ] **Step 4: 提交**

```bash
cd /home/mechdancer/repos/accounting
git rm accounting-web/src/components/layout/AccountNode.vue
git commit -m "refactor(account-grid): remove recursive AccountNode component"
```

---

### Task 8: 端到端验证

- [ ] **Step 1: 启动开发服务器并手动检查**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npm run dev
```

在浏览器打开 `http://localhost:5173`，切换到账户视图：

- 确认每个类型区块显示为网格。
- 点击有子账户的卡片，子账户在新行中左对齐显示，右侧空位为空白。
- 调整浏览器宽度，观察列数在 2/3/4 之间切换。
- 确认抽屉打开、关闭、重命名、删除等交互正常。

- [ ] **Step 2: 运行完整构建**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npm run build
```

Expected: 构建成功，无 TypeScript 错误。

- [ ] **Step 3: 提交最终验证结果**

```bash
cd /home/mechdancer/repos/accounting
git commit --allow-empty -m "chore(account-grid): verify build and manual tests"
```

---

## Self-Review

### 1. Spec Coverage

| Spec 要求 | 对应 Task |
|-----------|-----------|
| `expandedPath: number[]` 单路径展开 | Task 6 |
| 严格遵循 `compile` 算法生成行 | Task 1 |
| 空位用占位 `GridItem` 补齐 | Task 1、Task 3 |
| 响应式列数（2/3/4） via 容器查询 | Task 2、Task 5 |
| 高亮被展开卡片 | Task 3、Task 4 |
| 子账户分组容器（左边框/背景） | Task 4 |
| 保留抽屉交互 | Task 6 |
| 按账户类型分组 | Task 6 |
| 删除递归 `AccountNode` | Task 7 |

无遗漏。

### 2. Placeholder Scan

检查计划中的以下反模式：
- 无 "TBD" / "TODO" / "实现 later"。
- 每个步骤包含具体代码或命令。
- 测试代码完整，非占位。
- 文件路径均为绝对项目路径或相对 `accounting-web` 的明确路径。

### 3. Type Consistency

- `GridItem.account` 类型始终为 `AccountDto | null`。
- `GridRow` 在 Task 1 中定义，后续组件直接使用。
- `AccountGrid` 通过 `defineExpose` 暴露 `gridElement`，与 Task 2 的 `useGridColumns` 期望的 `Ref<HTMLElement | undefined>` 一致。
- `compileRows` 的 `getChildren` 签名与 store 的 `getChildren` 一致。

计划可执行。
