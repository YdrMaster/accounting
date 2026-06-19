<template>
  <div ref="gridEl" class="cards-grid">
    <draggable
      v-model="localList"
      item-key="id"
      handle=".drag-handle"
      :animation="200"
      @end="onDragEnd"
      class="draggable-wrapper"
    >
      <template #item="{ element: account }">
        <div class="card-wrapper">
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
              <span v-if="account.closed_at" class="closed-tag">
                <a-tag color="default" style="margin: 0; font-size: 11px">已关闭</a-tag>
              </span>
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
            </template>
          </AccountCard>

          <!-- Recursive children / inline add -->
          <div v-if="isExpanded(account.id) && (childrenCount(account.id) > 0 || addingChildOf === account.id)" class="sub-cards">
            <AccountCards
              v-if="childrenCount(account.id) > 0"
              :parent-id="account.id"
              :type="type"
              :accounts="accounts"
              :expanded-stack="expandedStack"
              @navigate="(id, pushOnly) => emit('navigate', id, pushOnly)"
            />
            <div v-if="addingChildOf === account.id && selectedId === account.id" class="sub-add-row">
              <a-input
                ref="addInputRef"
                v-model:value="newChildName"
                size="small"
                placeholder="输入子账户名"
                class="sub-add-input"
                @press-enter="confirmAdd(account)"
                @click.stop
              />
              <a-button size="small" type="primary" @click.stop="confirmAdd(account)">确认</a-button>
              <a-button size="small" @click.stop="cancelAdd">取消</a-button>
            </div>
          </div>
        </div>
      </template>
    </draggable>

    <!-- Inline add for this level -->
    <div v-if="addingAtRoot" class="sub-add-row">
      <a-input
        ref="addInputRef"
        v-model:value="newChildName"
        size="small"
        placeholder="输入子账户名"
        class="sub-add-input"
        @press-enter="confirmRootAdd"
        @click.stop
      />
      <a-button size="small" type="primary" @click.stop="confirmRootAdd">确认</a-button>
      <a-button size="small" @click.stop="cancelAdd">取消</a-button>
    </div>

    <!-- Add card: hidden while adding at this level -->
    <div v-if="!addingAtRoot" class="card-wrapper">
      <div class="account-card add-card-box" @click="handleStartAddRoot">
        <span class="add-card-text">+ 添加</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export default { name: 'AccountCards' }
</script>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import draggable from 'vuedraggable'
import { message } from 'ant-design-vue'
import type { Account } from '@/stores/account'
import { useAccountStore } from '@/stores/account'
import AccountCard from '@/components/AccountCard.vue'

const props = defineProps<{
  parentId: number | null
  type: string
  accounts: Account[]
  expandedStack: number[]
}>()

const emit = defineEmits<{
  navigate: [id: number, pushOnly: boolean]
}>()

const accountStore = useAccountStore()

const selectedId = computed(() => {
  const s = props.expandedStack
  return s.length > 0 ? s[s.length - 1] : null
})

function isExpanded(id: number): boolean {
  return props.expandedStack.includes(id)
}

// --- Drag & drop ---
const localList = ref<Account[]>([])

watch(
  () => {
    return props.accounts
      .filter((a) => a.parent_id === props.parentId && a.account_type === props.type)
      .sort((a, b) => a.position - b.position)
  },
  (val) => {
    localList.value = [...val]
  },
  { immediate: true }
)

function onDragEnd() {
  const ids = localList.value.map((a) => a.id)
  accountStore.reorderAccounts(ids)
}

// --- Children count (prebuilt Map for O(1) lookup) ---
const childrenCountMap = computed(() => {
  const map = new Map<number, number>()
  for (const a of props.accounts) {
    if (a.parent_id != null) {
      map.set(a.parent_id, (map.get(a.parent_id) || 0) + 1)
    }
  }
  return map
})

function childrenCount(parentId: number): number {
  return childrenCountMap.value.get(parentId) || 0
}

// --- Selection / Navigation ---
function handleSelectCard(account: Account) {
  emit('navigate', account.id, false)
}

// --- Expand / collapse ---
function handleToggleExpand(account: Account) {
  const stack = props.expandedStack
  const isTop = stack.length > 0 && stack[stack.length - 1] === account.id
  emit('navigate', account.id, isTop)
}

// --- Add child (local state, independent per component instance) ---
const addingChildOf = ref<number | null>(null)
const addingAtRoot = ref(false)
const newChildName = ref('')
const addInputRef = ref<HTMLInputElement | null>(null)

function handleStartAdd(account: Account) {
  emit('navigate', account.id, false)
  addingChildOf.value = account.id
  addingAtRoot.value = false
  newChildName.value = ''
  nextTick(() => addInputRef.value?.focus())
}

async function confirmAdd(parent: Account) {
  const name = newChildName.value.trim()
  if (!name) {
    cancelAdd()
    return
  }
  const siblings = props.accounts.filter((a) => a.parent_id === parent.id)
  const fullName = `${parent.full_name}:${name}`
  if (siblings.some((a) => a.full_name === fullName)) {
    message.warning('同名账户已存在')
    return
  }
  addingChildOf.value = null
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function handleStartAddRoot() {
  addingAtRoot.value = true
  addingChildOf.value = null
  newChildName.value = ''
  nextTick(() => addInputRef.value?.focus())
}

async function confirmRootAdd() {
  const name = newChildName.value.trim()
  if (!name) { cancelAdd(); return }
  if (props.parentId == null) return
  const parent = props.accounts.find(a => a.id === props.parentId)
  if (!parent) return
  const fullName = `${parent.full_name}:${name}`
  const siblings = props.accounts.filter(a => a.parent_id === parent.id)
  if (siblings.some(a => a.full_name === fullName)) { message.warning('同名账户已存在'); return }
  addingAtRoot.value = false
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function cancelAdd() {
  addingChildOf.value = null
  addingAtRoot.value = false
  newChildName.value = ''
}

// --- Triangle positioning ---
const gridEl = ref<HTMLElement | null>(null)

const expandedIndex = computed(() => {
  const stack = props.expandedStack
  if (stack.length === 0) return -1
  const children = props.accounts
    .filter(a => a.parent_id === props.parentId && a.account_type === props.type)
    .sort((a, b) => a.position - b.position)
  for (let i = stack.length - 1; i >= 0; i--) {
    const idx = children.findIndex(c => c.id === stack[i])
    if (idx >= 0) return idx
  }
  return -1
})

function updateTriangle() {
  const idx = expandedIndex.value
  if (idx < 0 || !gridEl.value) return
  const draggable = gridEl.value.querySelector(':scope > .draggable-wrapper')
  const container = draggable || gridEl.value
  const cards = container.querySelectorAll(':scope > .card-wrapper > .account-card')
  const card = cards[idx] as HTMLElement | undefined
  if (!card) return
  const cardWrapper = card.closest('.card-wrapper')
  const subCards = cardWrapper?.querySelector('.sub-cards') as HTMLElement | undefined
  if (!subCards) return
  const cardRect = card.getBoundingClientRect()
  const gridRect = gridEl.value.getBoundingClientRect()
  const left = cardRect.left + cardRect.width / 2 - gridRect.left
  subCards.style.setProperty('--tri-left', `${left}px`)
}

watch(() => props.expandedStack, () => {
  nextTick(updateTriangle)
}, { deep: true })

onMounted(() => nextTick(updateTriangle))
</script>

<style scoped>
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  min-height: 36px;
  align-items: flex-start;
}

.draggable-wrapper {
  display: contents;
}

.card-wrapper {
  display: contents;
}

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

.drag-handle {
  cursor: grab;
  color: #999;
  font-size: 14px;
  flex-shrink: 0;
  line-height: 1;
}

.drag-handle:active {
  cursor: grabbing;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.child-count {
  font-size: 11px;
  color: #999;
  background: #f5f5f5;
  border-radius: 10px;
  padding: 0 6px;
  line-height: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
}

.expand-arrow {
  font-size: 10px;
  color: #999;
  transition: transform 0.2s;
  cursor: pointer;
  padding: 2px;
}

.expand-arrow.rotated {
  transform: rotate(180deg);
}

.add-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 16px;
  font-weight: bold;
  line-height: 18px;
  color: #52c41a;
  cursor: pointer;
  transition: background 0.2s;
}

.add-btn:hover {
  background: rgba(82, 196, 26, 0.12);
}

.sub-cards {
  order: 1;
  flex-basis: 100%;
  position: relative;
  border: 1px solid var(--bubble-border, #d9d9d9);
  border-radius: 8px;
  background: var(--bubble-bg, #fafafa);
  padding: 12px;
  --tri-left: 80px;
}

.sub-cards::before {
  content: '';
  position: absolute;
  top: -8px;
  left: var(--tri-left);
  transform: translateX(-50%);
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 8px solid var(--bubble-border, #d9d9d9);
}

.sub-cards::after {
  content: '';
  position: absolute;
  top: -7px;
  left: var(--tri-left);
  transform: translateX(-50%);
  border-left: 7px solid transparent;
  border-right: 7px solid transparent;
  border-bottom: 7px solid var(--bubble-bg, #fafafa);
}

.sub-add-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid #52c41a;
  border-radius: 6px;
  background: rgba(82, 196, 26, 0.05);
  min-height: 50px;
  box-sizing: border-box;
}

.sub-add-input {
  width: 130px;
}

.closed-tag {
  display: inline-flex;
  align-items: center;
}

.add-card-box {
  border: 1px dashed #d9d9d9;
  justify-content: center;
  transition: border-color 0.2s;
}

.add-card-box:hover {
  border-color: #52c41a;
}

.add-card-text {
  color: #52c41a;
  font-weight: 500;
  font-size: 14px;
}
</style>
