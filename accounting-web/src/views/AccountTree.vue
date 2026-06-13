<template>
  <div class="account-tabs">
    <a-tabs v-model:activeKey="activeTab" @change="resetAll">
      <a-tab-pane v-for="t in tabTypes" :key="t.value" :tab="t.label" />
    </a-tabs>

    <div class="cards-area">
      <AccountCards
        ref="rootCardsRef"
        :parent-id="null"
        :type="activeTab"
        :accounts="accounts"
        :selected-id="selectedId"
        :expanded-id="expandedId"
        :adding-parent-id="addingParentId"
        @update:selected="onSelectedChange"
        @update:expanded="onExpandedChange"
        @start-add="addingParentId = $event"
      />
    </div>

    <div v-if="selectedAccount" class="detail-panel">
      <div class="detail-header">
        <h3>账户详情</h3>
        <a-tag v-if="selectedAccount.is_system" color="orange">系统内置账户</a-tag>
        <a-tag v-if="selectedAccount.closed_at" color="default">已关闭</a-tag>
      </div>

      <div class="detail-row">
        <span class="detail-label">名称</span>
        <span v-if="renamingId !== selectedAccount.id" class="detail-value-wrap">
          <span>{{ selectedAccount.full_name }}</span>
          <a-button
            v-if="!selectedAccount.is_system"
            type="link"
            size="small"
            @click="startRename"
          >重命名</a-button>
        </span>
        <span v-else class="rename-row">
          <a-input
            ref="detailRenameInput"
            v-model:value="renameValue"
            size="small"
            class="detail-rename-input"
            @press-enter="confirmRename"
          />
          <a-button size="small" type="primary" @click="confirmRename">确认</a-button>
          <a-button size="small" @click="cancelRename">取消</a-button>
        </span>
      </div>

      <div class="detail-row">
        <span class="detail-label">类型</span>
        <span>{{ selectedAccount.account_type }}</span>
      </div>

      <div v-if="selectedAccount.account_type === 'Asset'" class="detail-row owners-row">
        <span class="detail-label">所有者</span>
        <a-checkbox-group
          :value="selectedAccount.owner_ids || []"
          :options="memberOptions"
          :disabled="selectedAccount.is_system"
          @change="handleUpdateOwners"
        />
      </div>

      <div v-if="!selectedAccount.is_system" class="detail-actions">
        <a-button
          v-if="!selectedAccount.closed_at"
          danger
          size="small"
          @click="handleClose"
        >关闭账户</a-button>
        <a-button
          v-else
          size="small"
          @click="handleReopen"
        >重新打开</a-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted } from 'vue'
import { useAccountStore } from '@/stores/account'
import type { Account } from '@/stores/account'
import { useMemberStore } from '@/stores/member'
import AccountCards from './AccountCards.vue'

const accountStore = useAccountStore()
const memberStore = useMemberStore()

const activeTab = ref('Asset')
const selectedId = ref<number | null>(null)
const expandedId = ref<number | null>(null)
const addingParentId = ref<number | null>(null)

const rootCardsRef = ref<InstanceType<typeof AccountCards> | null>(null)

const accounts = computed(() => accountStore.accounts)

const tabTypes = [
  { value: 'Asset', label: '资产' },
  { value: 'Liability', label: '负债' },
  { value: 'Income', label: '收入' },
  { value: 'Expense', label: '支出' },
  { value: 'Equity', label: '权益' },
]

function resetAll() {
  selectedId.value = null
  expandedId.value = null
  addingParentId.value = null
}

function onSelectedChange(id: number | null) {
  selectedId.value = id
}

function onExpandedChange(id: number | null) {
  expandedId.value = id
}

// --- Selected account ---
const selectedAccount = computed<Account | null>(() => {
  if (selectedId.value === null) return null
  return accountStore.accounts.find((a) => a.id === selectedId.value) || null
})

// --- Rename ---
const renamingId = ref<number | null>(null)
const renameValue = ref('')
const detailRenameInput = ref<HTMLInputElement | null>(null)

function startRename() {
  if (!selectedAccount.value) return
  renamingId.value = selectedAccount.value.id
  renameValue.value = selectedAccount.value.full_name.split(':').pop() || ''
  nextTick(() => {
    detailRenameInput.value?.focus()
    detailRenameInput.value?.select?.()
  })
}

async function confirmRename() {
  if (!selectedAccount.value) return
  const name = renameValue.value.trim()
  if (!name) {
    cancelRename()
    return
  }
  const account = selectedAccount.value
  // Check sibling duplicate
  const siblings = accountStore.accounts.filter((a) => a.parent_id === account.parent_id)
  const segments = account.full_name.split(':')
  segments[segments.length - 1] = name
  const newFullName = segments.join(':')
  if (siblings.some((a) => a.id !== account.id && a.full_name === newFullName)) {
    cancelRename()
    return
  }
  renamingId.value = null
  renameValue.value = ''
  await accountStore.renameAccount(account.id, newFullName)
  // Re-select after refresh (id stays same)
}

function cancelRename() {
  renamingId.value = null
  renameValue.value = ''
}

// --- Close / Reopen ---
async function handleClose() {
  if (!selectedAccount.value) return
  await accountStore.closeAccount(selectedAccount.value.id)
}

async function handleReopen() {
  if (!selectedAccount.value) return
  await accountStore.reopenAccount(selectedAccount.value.id)
}

// --- Owners ---
const members = computed(() => memberStore.members)
const memberOptions = computed(() =>
  members.value.map((m) => ({ label: m.name, value: m.id }))
)

async function handleUpdateOwners(checkedValues: (string | number)[]) {
  if (!selectedAccount.value) return
  if (selectedAccount.value.is_system) return
  const ids = checkedValues.map((v) => Number(v))
  await accountStore.setOwners(selectedAccount.value.id, ids)
}

onMounted(() => {
  accountStore.fetchAccounts()
  memberStore.fetchMembers()
})
</script>

<style scoped>
.account-tabs {
  width: 100%;
}

.cards-area {
  min-height: 48px;
  padding: 4px 0;
}

/* --- Detail panel --- */
.detail-panel {
  margin-top: 24px;
  background: transparent;
  padding: 24px;
  border-top: 1px solid #f0f0f0;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.detail-header h3 {
  margin: 0;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.detail-row.owners-row {
  align-items: flex-start;
}

.detail-label {
  width: 60px;
  color: #666;
  font-weight: 500;
  flex-shrink: 0;
}

.detail-value-wrap {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.rename-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-rename-input {
  width: 160px;
}

.detail-actions {
  margin-top: 16px;
  display: flex;
  gap: 8px;
}
</style>
