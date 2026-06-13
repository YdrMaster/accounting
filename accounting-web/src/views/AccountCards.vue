<template>
  <draggable
    v-model="localList"
    item-key="id"
    handle=".drag-handle"
    class="cards-grid"
    :animation="200"
    @end="onDragEnd"
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
              <template v-if="renamingId === account.id">
                <a-input
                  ref="renameInputRef"
                  v-model:value="renameValue"
                  size="small"
                  class="rename-input"
                  @press-enter="confirmRename(account)"
                  @click.stop
                  @blur="cancelRename"
                />
              </template>
              <template v-else>
                {{ shortName(account.full_name) }}
              </template>
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
              <span
                v-else
                class="add-btn"
                @click.stop="handleStartAdd(account)"
              >+</span>
            </span>
          </div>
        </div>

        <!-- Add new account inline -->
        <div v-if="addingParentId === account.id" class="inline-add-row">
          <a-input
            ref="addInputRef"
            v-model:value="newChildName"
            size="small"
            placeholder="输入子账户名"
            class="inline-input"
            @press-enter="confirmAdd(account)"
            @click.stop
          />
          <a-button size="small" type="primary" @click.stop="confirmAdd(account)">确认</a-button>
          <a-button size="small" @click.stop="cancelAdd">取消</a-button>
        </div>

        <!-- Recursive children -->
        <div v-if="expandedId === account.id && childrenCount(account.id) > 0" class="sub-cards">
          <AccountCards
            :parent-id="account.id"
            :type="type"
            :accounts="accounts"
            :selected-id="selectedId"
            :expanded-id="expandedId"
            :adding-parent-id="addingParentId"
            @update:selected="$emit('update:selected', $event)"
            @update:expanded="$emit('update:expanded', $event)"
            @start-add="$emit('start-add', $event)"
          />
        </div>
      </div>
    </template>
  </draggable>
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
  addingParentId: number | null
}>()

const emit = defineEmits<{
  'update:selected': [id: number | null]
  'update:expanded': [id: number | null]
  'start-add': [parentId: number | null]
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
    if (props.expandedId !== account.id) {
      emit('update:expanded', null)
    }
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

// --- Add child ---
const newChildName = ref('')
const addInputRef = ref<HTMLInputElement | null>(null)

function handleStartAdd(account: Account) {
  emit('start-add', account.id)
  if (props.expandedId !== account.id) {
    emit('update:expanded', account.id)
    if (props.selectedId !== account.id) {
      emit('update:selected', null)
    }
  }
  newChildName.value = ''
  nextTick(() => {
    addInputRef.value?.focus()
  })
}

async function confirmAdd(parent: Account) {
  const name = newChildName.value.trim()
  if (!name) {
    cancelAdd()
    return
  }
  // Check sibling duplicate
  const siblings = props.accounts.filter((a) => a.parent_id === parent.id)
  const fullName = `${parent.full_name}:${name}`
  if (siblings.some((a) => a.full_name === fullName)) {
    return
  }
  emit('start-add', null)
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function cancelAdd() {
  emit('start-add', null)
  newChildName.value = ''
}

// --- Rename ---
const renamingId = ref<number | null>(null)
const renameValue = ref('')
const renameInputRef = ref<HTMLInputElement | null>(null)

// Expose for parent to trigger rename
defineExpose({
  startRename(account: Account) {
    renamingId.value = account.id
    renameValue.value = account.full_name.split(':').pop() || ''
    nextTick(() => {
      renameInputRef.value?.focus()
      renameInputRef.value?.select?.()
    })
  },
})

async function confirmRename(account: Account) {
  const name = renameValue.value.trim()
  if (!name || name === account.full_name.split(':').pop()) {
    cancelRename()
    return
  }
  const segments = account.full_name.split(':')
  segments[segments.length - 1] = name
  const newFullName = segments.join(':')
  // Check sibling duplicate
  const siblings = props.accounts.filter((a) => a.parent_id === account.parent_id)
  if (siblings.some((a) => a.id !== account.id && a.full_name === newFullName)) {
    cancelRename()
    return
  }
  renamingId.value = null
  renameValue.value = ''
  await accountStore.renameAccount(account.id, newFullName)
}

function cancelRename() {
  renamingId.value = null
  renameValue.value = ''
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
  line-height: 18px;
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
  color: #52c41a;
  cursor: pointer;
  transition: background 0.2s;
}

.add-btn:hover {
  background: rgba(82, 196, 26, 0.12);
}

.inline-add-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 6px;
  background: #fafafa;
  border-radius: 4px;
}

.inline-input {
  width: 130px;
}

.rename-input {
  width: 120px;
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
</style>
