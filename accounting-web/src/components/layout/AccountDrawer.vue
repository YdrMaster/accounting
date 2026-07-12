<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  closeAccount,
  deleteAccount,
  fetchMembers,
  renameAccount,
  reopenAccount,
  setAccountOwners,
  updateAccountFields,
} from '../../api/client'
import type { AccountDto, MemberDto } from '../../types/api'

const props = defineProps<{
  account: AccountDto
}>()

const emit = defineEmits<{
  close: []
  updated: [account: AccountDto]
  deleted: [id: number]
}>()

const members = ref<MemberDto[]>([])
const editingTitle = ref(false)
const titleInputRef = ref<HTMLInputElement | null>(null)
const draftName = ref('')
const draftBillingDay = ref<number | null>(null)
const draftRepaymentDay = ref<number | null>(null)
const selectedOwnerIds = ref<number[]>([])
const error = ref<string | null>(null)
const success = ref<string | null>(null)
const ownerDropdownOpen = ref(false)
const ownerDropdownRef = ref<HTMLElement | null>(null)

onMounted(() => {
  fetchMembers()
    .then(m => {
      members.value = m
    })
    .catch(() => {})
  resetDrafts()
  document.addEventListener('mousedown', onDocClick)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', onDocClick)
})

function onDocClick(e: MouseEvent) {
  if (ownerDropdownRef.value && !ownerDropdownRef.value.contains(e.target as Node)) {
    ownerDropdownOpen.value = false
  }
}

watch(
  () => props.account.id,
  () => {
    resetDrafts()
    error.value = null
    success.value = null
  }
)

function resetDrafts() {
  draftName.value = props.account.name
  draftBillingDay.value = props.account.billing_day
  draftRepaymentDay.value = props.account.repayment_day
  selectedOwnerIds.value = [...props.account.owner_ids]
  editingTitle.value = false
}

const isRoot = props.account.parent_id === null
const isSystem = props.account.is_system
const isClosed = computed(() => props.account.closed_at !== null)

const currentOwners = computed(() =>
  members.value.filter(m => selectedOwnerIds.value.includes(m.id))
)

const availableMembers = computed(() =>
  members.value.filter(m => !selectedOwnerIds.value.includes(m.id))
)

function clampDay(val: number | null): number | null {
  if (val === null) return null
  if (val < 1) return 1
  if (val > 31) return 31
  return val
}

function clearMessages() {
  error.value = null
  success.value = null
}

async function startEditTitle() {
  if (isRoot) return
  draftName.value = props.account.name
  editingTitle.value = true
  await nextTick()
  titleInputRef.value?.focus()
  titleInputRef.value?.select()
}

async function doRename() {
  if (!editingTitle.value) return
  editingTitle.value = false
  clearMessages()
  const name = draftName.value.trim()
  if (!name || name === props.account.name) {
    return
  }
  try {
    await renameAccount(props.account.id, name)
    emit('updated', { ...props.account, name })
    success.value = '名称已更新'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function doBillingDay() {
  clearMessages()
  const val = clampDay(draftBillingDay.value)
  if (val === null || val === props.account.billing_day) return
  draftBillingDay.value = val
  try {
    await updateAccountFields(props.account.id, val, props.account.repayment_day)
    emit('updated', { ...props.account, billing_day: val })
    success.value = '账单日已更新'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function doRepaymentDay() {
  clearMessages()
  const val = clampDay(draftRepaymentDay.value)
  if (val === null || val === props.account.repayment_day) return
  draftRepaymentDay.value = val
  try {
    await updateAccountFields(props.account.id, props.account.billing_day, val)
    emit('updated', { ...props.account, repayment_day: val })
    success.value = '还款日已更新'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function addOwner(memberId: number) {
  clearMessages()
  if (selectedOwnerIds.value.includes(memberId)) return
  const newIds = [...selectedOwnerIds.value, memberId]
  try {
    await setAccountOwners(props.account.id, newIds)
    selectedOwnerIds.value = newIds
    emit('updated', { ...props.account, owner_ids: newIds })
    success.value = '所有者已更新'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function removeOwner(memberId: number) {
  clearMessages()
  const newIds = selectedOwnerIds.value.filter(id => id !== memberId)
  try {
    await setAccountOwners(props.account.id, newIds)
    selectedOwnerIds.value = newIds
    emit('updated', { ...props.account, owner_ids: newIds })
    success.value = '所有者已更新'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function doClose() {
  clearMessages()
  try {
    await closeAccount(props.account.id)
    emit('updated', { ...props.account, closed_at: new Date().toISOString() })
    success.value = '账户已关闭'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function doReopen() {
  clearMessages()
  try {
    await reopenAccount(props.account.id)
    emit('updated', { ...props.account, closed_at: null })
    success.value = '账户已重开'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function doDelete() {
  if (isRoot || isSystem) return
  clearMessages()
  if (!confirm(`确定删除账户「${props.account.name}」？`)) return
  try {
    await deleteAccount(props.account.id)
    emit('deleted', props.account.id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="drawer-container">
    <div class="drawer">
      <div class="drawer-handle" />

      <div class="drawer-header">
        <input
          v-show="editingTitle"
          ref="titleInputRef"
          v-model="draftName"
          class="title-input"
          @keyup.enter="doRename"
          @blur="doRename"
          @keydown.escape="editingTitle = false"
        />
        <h2
          v-show="!editingTitle"
          class="drawer-title"
          :class="{ clickable: !isRoot }"
          @click="startEditTitle"
        >
          {{ account.name }}
        </h2>
        <button type="button" class="close-btn" @click="emit('close')">&times;</button>
      </div>

      <div v-if="error" class="message error-msg">{{ error }}</div>
      <div v-if="success" class="message success-msg">{{ success }}</div>

      <div class="drawer-body">
        <div v-if="isRoot" class="info-note">根账户，不可编辑</div>

        <template v-else>
          <div class="field">
            <label class="field-label">账单日（每月第 N 日）</label>
            <input
              v-model.number="draftBillingDay"
              type="number"
              min="1"
              max="31"
              class="input small"
              @change="doBillingDay"
            />
          </div>

          <div class="field">
            <label class="field-label">还款日（每月第 N 日）</label>
            <input
              v-model.number="draftRepaymentDay"
              type="number"
              min="1"
              max="31"
              class="input small"
              @change="doRepaymentDay"
            />
          </div>

          <div class="field">
            <label class="field-label">所有者</label>
            <div ref="ownerDropdownRef" class="owner-input-tag">
              <div class="owner-tag-list">
                <button
                  v-for="owner in currentOwners"
                  :key="owner.id"
                  type="button"
                  class="owner-tag"
                  @click="removeOwner(owner.id)"
                >
                  {{ owner.name }} <span class="tag-remove">&times;</span>
                </button>
                <button
                  v-if="availableMembers.length > 0"
                  type="button"
                  class="owner-tag add-tag"
                  @click="ownerDropdownOpen = !ownerDropdownOpen"
                >
                  +
                </button>
              </div>
              <div v-if="ownerDropdownOpen && availableMembers.length > 0" class="owner-dropdown">
                <button
                  v-for="member in availableMembers"
                  :key="member.id"
                  type="button"
                  class="dropdown-item"
                  @click="addOwner(member.id); ownerDropdownOpen = false"
                >
                  {{ member.name }}
                </button>
              </div>
            </div>
          </div>

          <div class="actions">
            <button v-if="!isClosed" type="button" class="action-btn warn" @click="doClose">
              关闭账户
            </button>
            <button v-else type="button" class="action-btn" @click="doReopen">重开账户</button>
            <button
              v-if="!isRoot && !isSystem"
              type="button"
              class="action-btn danger"
              @click="doDelete"
            >
              删除账户
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drawer-container {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  pointer-events: none;
}

.drawer {
  position: relative;
  width: 100%;
  max-width: 600px;
  max-height: 50vh;
  background: var(--bg);
  border-radius: 1rem 1rem 0 0;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease;
  overflow: hidden;
  margin: 0 auto;
  pointer-events: auto;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
}

@keyframes slideUp {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}

.drawer-handle {
  width: 2.5rem;
  height: 0.25rem;
  background: var(--border);
  border-radius: 0.125rem;
  margin: 0.75rem auto 0;
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem 0.75rem;
  box-sizing: border-box;
}

.drawer-title {
  margin: 0 !important;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-heading);
  line-height: 1.5;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid transparent;
  box-sizing: border-box;
  flex: 1;
  display: flex;
  align-items: center;
}

.drawer-title.clickable {
  cursor: pointer;
  transition: background 0.15s;
}

.drawer-title.clickable:hover {
  background: var(--card-bg-alt);
}

.title-input {
  font-size: 1.125rem;
  font-weight: 600;
  line-height: 1.5;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid var(--accent);
  background: var(--card-bg-alt);
  color: var(--text-heading);
  outline: none;
  box-sizing: border-box;
  margin: 0;
  flex: 1;
  display: flex;
  align-items: center;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
  flex-shrink: 0;
}

.message {
  margin: 0 1.25rem 0.5rem;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
}

.error-msg {
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
}

.success-msg {
  background: rgba(39, 174, 96, 0.15);
  color: #27ae60;
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.info-note {
  color: var(--text-muted);
  font-size: 0.875rem;
  text-align: center;
  padding: 1rem 0;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.field-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-weight: 500;
}

.field-value {
  font-size: 0.9375rem;
  color: var(--text-heading);
}

.input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border);
  background: var(--card-bg-alt);
  color: var(--text-heading);
  font-size: 0.9375rem;
  outline: none;
}

.input:focus {
  border-color: var(--accent);
}

.input.small {
  width: 5rem;
}

.owner-input-tag {
  position: relative;
}

.owner-tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--card-bg-alt);
  min-height: 2.25rem;
  align-items: center;
  cursor: text;
}

.owner-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 0.8125rem;
  cursor: pointer;
  transition: opacity 0.15s;
  line-height: 1.4;
}

.owner-tag:hover {
  opacity: 0.8;
}

.owner-tag .tag-remove {
  font-size: 0.875rem;
  line-height: 1;
}

.owner-tag.add-tag {
  background: transparent;
  color: var(--text-muted);
  font-size: 1rem;
  padding: 0 0.25rem;
  cursor: pointer;
}

.owner-tag.add-tag:hover {
  color: var(--accent);
  opacity: 1;
}

.owner-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 10;
  margin-top: 0.25rem;
  background: var(--card-bg-alt);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  max-height: 10rem;
  overflow-y: auto;
}

.dropdown-item {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  background: transparent;
  color: var(--text-heading);
  font-size: 0.875rem;
  text-align: left;
  cursor: pointer;
}

.dropdown-item:hover {
  background: var(--accent);
  color: #fff;
}

.actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 0.5rem;
}

.action-btn {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border);
  background: var(--card-bg-alt);
  color: var(--text-heading);
  font-size: 0.8125rem;
  cursor: pointer;
}

.action-btn.warn {
  border-color: #f39c12;
  color: #f39c12;
}

.action-btn.danger {
  border-color: #e74c3c;
  color: #e74c3c;
}
</style>
