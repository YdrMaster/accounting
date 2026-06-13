<template>
  <div class="account-picker">
    <button class="picker-trigger" :class="{ disabled: props.disabled }" @click="showPanel = true" :disabled="props.disabled">
      <span v-if="selectedAccount" class="selected-name">{{ selectedAccount.full_name }}</span>
      <span v-else class="placeholder">{{ placeholder }}</span>
    </button>

    <div v-if="showPanel" class="panel-overlay" @click.self="showPanel = false">
      <div class="panel">
        <div class="panel-header">
          <span class="panel-title">选择账户</span>
          <button class="close-btn" @click="showPanel = false">×</button>
        </div>

        <div v-if="currentPath.length > 0" class="breadcrumb">
          <span
            v-for="(item, idx) in breadcrumbItems"
            :key="idx"
            class="breadcrumb-item"
            @click="goToLevel(idx)"
          >
            {{ item.title }}
            <span v-if="idx < breadcrumbItems.length - 1" class="separator">&gt;</span>
          </span>
        </div>

        <div class="panel-body">
          <div
            v-for="acc in currentLevel"
            :key="acc.id"
            class="account-btn"
            :class="{
              'is-leaf': isLeaf(acc),
              'is-selected': selectedAccount?.id === acc.id,
            }"
            @click="handleSelect(acc)"
          >
            {{ getShortName(acc) }}
          </div>
        </div>

        <div v-if="selectedAccount" class="panel-footer">
          已选：{{ selectedAccount.full_name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAccountStore, type Account } from '@/stores/account'

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
const showPanel = ref(false)

const currentPath = ref<number[]>([])

const rootAccounts = computed(() => {
  const accounts = accountStore.accounts
  if (props.accountType) {
    return accounts.filter(a => a.account_type === props.accountType && a.parent_id == null)
  }
  return accounts.filter(a => a.parent_id == null)
})

const currentLevel = computed<Account[]>(() => {
  if (currentPath.value.length === 0) {
    return rootAccounts.value
  }
  const parentId = currentPath.value[currentPath.value.length - 1]
  return accountStore.accounts.filter(a => a.parent_id === parentId)
})

const breadcrumbItems = computed(() => {
  const items: { id: number; title: string }[] = []
  for (const id of currentPath.value) {
    const acc = accountStore.accounts.find(a => a.id === id)
    if (acc) {
      items.push({ id, title: acc.full_name.split(':').pop() || acc.full_name })
    }
  }
  return items
})

const selectedAccount = computed(() => {
  if (!props.modelValue) return null
  return accountStore.accounts.find(a => a.id === props.modelValue) || null
})

function getShortName(acc: Account): string {
  return acc.full_name.split(':').pop() || acc.full_name
}

function isLeaf(acc: Account): boolean {
  return !accountStore.accounts.some(a => a.parent_id === acc.id)
}

function handleSelect(acc: Account) {
  if (isLeaf(acc)) {
    emit('update:modelValue', acc.id)
    showPanel.value = false
  } else {
    currentPath.value.push(acc.id)
  }
}

function goToLevel(idx: number) {
  currentPath.value = currentPath.value.slice(0, idx)
}

watch(() => props.modelValue, () => {
  currentPath.value = []
})
</script>

<style scoped>
.account-picker {
  position: relative;
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

.panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel {
  background: #fff;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.panel-title {
  font-size: 16px;
  font-weight: 500;
  color: #333;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
  padding: 0 4px;
}

.close-btn:hover {
  color: #333;
}

.breadcrumb {
  padding: 12px 16px;
  background: #fafafa;
  border-bottom: 1px solid #f0f0f0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.breadcrumb-item {
  cursor: pointer;
  color: #1890ff;
  font-size: 14px;
}

.breadcrumb-item:hover {
  text-decoration: underline;
}

.separator {
  color: #999;
  margin: 0 4px;
}

.panel-body {
  padding: 16px;
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
  overflow-y: auto;
  flex: 1;
}

.account-btn {
  height: 36px;
  padding: 0 8px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  text-align: center;
  font-size: 13px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  min-width: 0;
}

.account-btn:hover {
  border-color: #40a9ff;
  background: #f0f5ff;
}

.account-btn.is-leaf {
  border-color: #52c41a;
}

.account-btn.is-leaf:hover {
  border-color: #52c41a;
  background: #f6ffed;
}

.account-btn.is-selected {
  border-color: #1890ff;
  background: #e6f7ff;
}

.panel-footer {
  padding: 12px 16px;
  border-top: 1px solid #f0f0f0;
  font-size: 13px;
  color: #666;
  background: #fafafa;
}

/* Dark mode */
html.dark .picker-trigger {
  background: #1f1f1f;
  border-color: #434343;
  color: #fff;
}

html.dark .picker-trigger:hover {
  border-color: #177ddc;
}

html.dark .picker-trigger.disabled {
  background: #141414;
  border-color: #434343;
}

html.dark .selected-name {
  color: #fff;
}

html.dark .panel {
  background: #1f1f1f;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

html.dark .panel-header {
  border-bottom-color: #303030;
}

html.dark .panel-title {
  color: #fff;
}

html.dark .close-btn {
  color: #999;
}

html.dark .close-btn:hover {
  color: #fff;
}

html.dark .breadcrumb {
  background: #141414;
  border-bottom-color: #303030;
}

html.dark .account-btn {
  background: #1f1f1f;
  border-color: #434343;
  color: #fff;
}

html.dark .account-btn:hover {
  border-color: #177ddc;
  background: #111d2c;
}

html.dark .account-btn.is-leaf {
  border-color: #49aa19;
}

html.dark .account-btn.is-leaf:hover {
  border-color: #49aa19;
  background: #162312;
}

html.dark .account-btn.is-selected {
  border-color: #177ddc;
  background: #111d2c;
}

html.dark .panel-footer {
  border-top-color: #303030;
  background: #141414;
  color: #999;
}
</style>
