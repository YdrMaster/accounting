<template>
  <div class="cards-grid">
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
          <div
            class="account-card"
            :class="{
              selected: selectedId === account.id,
              expanded: expandedId === account.id,
              closed: account.closed_at,
              system: account.is_system,
            }"
            @click="handleSelectCard(account)"
          >
            <div class="card-header">
              <span class="drag-handle" @click.stop>⠿</span>
              <span class="card-name" :title="account.full_name">
                {{ shortName(account.full_name) }}
              </span>
              <span v-if="account.closed_at" class="closed-tag">
                <a-tag color="default" style="margin: 0; font-size: 11px">已关闭</a-tag>
              </span>
              <span class="card-actions">
                <template v-if="childrenCount(account.id) > 0">
                  <span class="child-count">{{ childrenCount(account.id) }}</span>
                  <span
                    class="expand-arrow"
                    :class="{ rotated: expandedId === account.id }"
                    @click.stop="handleToggleExpand(account)"
                  >▼</span>
                </template>
                <span v-else class="add-btn" @click.stop="handleStartAdd(account)">+</span>
              </span>
            </div>
          </div>

          <!-- Recursive children / inline add -->
          <div v-if="expandedId === account.id" class="sub-cards">
            <AccountCards
              v-if="childrenCount(account.id) > 0"
              :parent-id="account.id"
              :type="type"
              :accounts="accounts"
              :selected-id="selectedId"
              :expanded-id="expandedId"
              @update:selected="$emit('update:selected', $event)"
              @update:expanded="$emit('update:expanded', $event)"
            />
            <div v-if="isAdding && expandedId === account.id" class="sub-add-row">
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
    <div v-if="isAdding" class="sub-add-row">
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
    <div v-if="!isAdding" class="card-wrapper">
      <div class="account-card add-card-box" @click="handleStartAddRoot">
        <span class="add-card-text">+ 添加子账户</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export default { name: 'AccountCards' }
</script>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import draggable from 'vuedraggable'
import type { Account } from '@/stores/account'
import { useAccountStore } from '@/stores/account'

const props = defineProps<{
  parentId: number | null
  type: string
  accounts: Account[]
  selectedId: number | null
  expandedId: number | null
}>()

const emit = defineEmits<{
  'update:selected': [id: number | null]
  'update:expanded': [id: number | null]
}>()

const accountStore = useAccountStore()

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

// --- Children count ---
function childrenCount(parentId: number): number {
  return props.accounts.filter((a) => a.parent_id === parentId).length
}

// --- Selection ---
function handleSelectCard(account: Account) {
  if (props.selectedId === account.id) {
    emit('update:selected', null)
  } else {
    emit('update:selected', account.id)
  }
}

// --- Expand / collapse ---
function handleToggleExpand(account: Account) {
  if (props.expandedId === account.id) {
    emit('update:expanded', null)
  } else {
    emit('update:expanded', account.id)
    if (props.selectedId !== account.id) {
      emit('update:selected', null)
    }
  }
}

// --- Add child (local state, independent per component instance) ---
const isAdding = ref(false)
const newChildName = ref('')
const addInputRef = ref<HTMLInputElement | null>(null)

function handleStartAdd(account: Account) {
  isAdding.value = true
  if (props.expandedId !== account.id) {
    emit('update:expanded', account.id)
  }
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
    cancelAdd()
    return
  }
  isAdding.value = false
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function handleStartAddRoot() {
  isAdding.value = true
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
  if (siblings.some(a => a.full_name === fullName)) { cancelAdd(); return }
  isAdding.value = false
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function cancelAdd() {
  isAdding.value = false
  newChildName.value = ''
}

// --- Helpers ---
function shortName(fullName: string): string {
  return fullName.split(':').pop() || fullName
}
</script>

<style scoped>
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  min-height: 36px;
}

.draggable-wrapper {
  display: contents;
}

.card-wrapper {
  display: flex;
  flex-direction: column;
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
}

.account-card:hover {
  border-color: #91d5ff;
}

.account-card.selected {
  border-color: #1890ff;
  background: #e6f7ff;
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

.card-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
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

.sub-add-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid #52c41a;
  border-radius: 6px;
  background: rgba(82, 196, 26, 0.05);
  min-width: 140px;
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

.sub-cards {
  padding-left: 24px;
  margin-top: 8px;
  border-left: 2px solid #f0f0f0;
}

.add-card-box {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px dashed #d9d9d9;
  border-radius: 6px;
  cursor: pointer;
  min-width: 140px;
  min-height: 50px;
  transition: border-color 0.2s;
  box-sizing: border-box;
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
