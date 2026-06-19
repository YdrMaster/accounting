<template>
  <div class="cards-grid">
    <AccountTreeList
      :accounts="props.accounts"
      :parent-id="props.parentId"
      :type="props.type"
      :active-id="selectedId"
      mode="manage"
      :allow-add="true"
      :allow-drag="true"
      :adding-parent-id="addingParentId"
      v-model:expanded-stack="expandedStack"
      @update:active-id="handleActiveIdChange"
      @add="handleStartAddFromList"
      @reorder="handleReorder"
      @confirm-add="confirmAdd"
      @cancel-add="cancelAdd"
    />
  </div>
</template>

<script lang="ts">
export default { name: 'AccountCards' }
</script>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import type { Account } from '@/stores/account'
import { useAccountStore } from '@/stores/account'
import AccountTreeList from '@/components/AccountTreeList.vue'

const props = defineProps<{
  parentId: number | null
  type: string
  accounts: Account[]
}>()

const accountStore = useAccountStore()

const expandedStack = defineModel<number[]>('expandedStack', { default: () => [] })

const selectedId = computed(() => {
  const s = expandedStack.value
  return s.length > 0 ? s[s.length - 1] : null
})

function handleActiveIdChange(id: number | null) {
  if (id == null) {
    expandedStack.value = []
    return
  }
  const account = props.accounts.find((a) => a.id === id)
  if (!account) return
  const stack: number[] = []
  let current: Account | undefined = account
  while (current && current.parent_id != null) {
    stack.unshift(current.parent_id)
    current = props.accounts.find((a) => a.id === current!.parent_id)
  }
  stack.push(id)
  expandedStack.value = stack
}

// --- Add child (local state, independent per component instance) ---
const addingChildOf = ref<number | null>(null)
const addingAtRoot = ref(false)
const newChildName = ref('')

const addingParentId = computed(() => {
  if (addingAtRoot.value) return props.parentId
  return addingChildOf.value
})

function handleStartAddFromList(parentId: number | null) {
  if (parentId === props.parentId || parentId == null) {
    addingAtRoot.value = true
    addingChildOf.value = null
  } else {
    addingChildOf.value = parentId
    addingAtRoot.value = false
  }
  newChildName.value = ''
}

async function handleReorder(ids: number[]) {
  await accountStore.reorderAccounts(ids)
  message.success('排序已保存')
}

async function confirmAdd(parentId: number | null, name: string) {
  const trimmed = name.trim()
  if (!trimmed) {
    cancelAdd()
    return
  }
  const parent = props.accounts.find((a) => a.id === parentId)
  const fullName = parent ? `${parent.full_name}:${trimmed}` : trimmed
  const siblings = props.accounts.filter((a) => a.parent_id === parentId)
  if (siblings.some((a) => a.full_name === fullName)) {
    message.warning('同名账户已存在')
    return
  }
  addingChildOf.value = null
  addingAtRoot.value = false
  newChildName.value = ''
  await accountStore.createAccount(fullName)
}

function cancelAdd() {
  addingChildOf.value = null
  addingAtRoot.value = false
  newChildName.value = ''
}
</script>

<style scoped>
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  min-height: 36px;
  align-items: flex-start;
}
</style>
