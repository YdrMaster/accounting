<template>
  <div class="account-tree">
    <a-tree
      :tree-data="treeData"
      :default-expanded-keys="expandedKeys"
      class="tree"
    >
      <template #title="{ title, key, dataRef }">
        <template v-if="key === '__new__'">
          <div class="new-node-row">
            <a-input
              ref="newInputRef"
              v-model:value="newAccountName"
              size="small"
              placeholder="输入账户名"
              class="inline-input"
              @press-enter="handleConfirmNew"
            />
            <a-button size="small" type="primary" @click.stop="handleConfirmNew">确认</a-button>
            <a-button size="small" @click.stop="cancelAdd">取消</a-button>
          </div>
        </template>
        <template v-else>
          <span
            class="node-title"
            :class="{ active: selectedKey === key, system: dataRef?.is_system }"
            @click.stop="selectNode(Number(key))"
          >
            {{ title }}
          </span>
          <a-button
            type="link"
            size="small"
            class="add-btn"
            @click.stop="handleAddChild(Number(key))"
          >
            <PlusOutlined />
          </a-button>
        </template>
      </template>
    </a-tree>

    <!-- 详情面板 -->
    <div v-if="selectedAccount" class="detail-panel">
      <div class="detail-header">
        <h3>账户详情</h3>
        <a-tag v-if="selectedAccount.is_system" color="orange">系统内置账户</a-tag>
      </div>
      <div class="detail-row">
        <span class="detail-label">名称</span>
        <span>{{ selectedAccount.full_name }}</span>
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
          @change="handleUpdateOwners"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, computed } from 'vue'
import { PlusOutlined } from '@ant-design/icons-vue'
import { useAccountStore } from '@/stores/account'
import { useMemberStore } from '@/stores/member'

interface TreeNode {
  title: string
  key: string
  is_system?: boolean
  children: TreeNode[]
}

const accountStore = useAccountStore()
const memberStore = useMemberStore()

const treeData = ref<TreeNode[]>([])
const addingParentId = ref<number | null>(null)
const newAccountName = ref('')
const newInputRef = ref<HTMLInputElement | null>(null)
const selectedKey = ref<string | null>(null)
const expandedKeys = ref<string[]>([])

const members = computed(() => memberStore.members)

const memberOptions = computed(() =>
  members.value.map((m) => ({ label: m.name, value: m.id }))
)

const selectedAccount = computed(() => {
  if (!selectedKey.value) return null
  return accountStore.accounts.find((a) => String(a.id) === selectedKey.value) || null
})

function buildTreeData(): TreeNode[] {
  const accounts = accountStore.accounts
  const map = new Map<number, TreeNode>()
  const roots: TreeNode[] = []

  accounts.forEach((acc) => {
    const segments = acc.full_name.split(':')
    map.set(acc.id, {
      title: segments[segments.length - 1],
      key: String(acc.id),
      is_system: acc.is_system,
      children: [],
    })
  })

  accounts.forEach((acc) => {
    const node = map.get(acc.id)
    if (!node) return
    if (acc.parent_id && map.has(acc.parent_id)) {
      map.get(acc.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  if (addingParentId.value !== null) {
    const parentNode = findNode(roots, String(addingParentId.value))
    if (parentNode) {
      parentNode.children.push({
        title: '__new__',
        key: '__new__',
        children: [],
      })
    }
  }

  return roots
}

function findNode(nodes: TreeNode[], key: string): TreeNode | null {
  for (const node of nodes) {
    if (node.key === key) return node
    const found = findNode(node.children, key)
    if (found) return found
  }
  return null
}

function handleAddChild(parentId: number) {
  addingParentId.value = parentId
  newAccountName.value = ''
  treeData.value = buildTreeData()
  nextTick(() => {
    newInputRef.value?.focus()
  })
}

async function handleConfirmNew() {
  const name = newAccountName.value.trim()
  if (!name) {
    cancelAdd()
    return
  }

  let fullName = name
  if (addingParentId.value !== null) {
    const parent = accountStore.accounts.find((a) => a.id === addingParentId.value)
    if (parent) {
      fullName = `${parent.full_name}:${name}`
    }
  }

  addingParentId.value = null
  newAccountName.value = ''
  await accountStore.createAccount(fullName)
}

function cancelAdd() {
  addingParentId.value = null
  newAccountName.value = ''
  treeData.value = buildTreeData()
}

function selectNode(id: number) {
  selectedKey.value = String(id)
}

async function handleUpdateOwners(checkedValues: (string | number)[]) {
  if (!selectedAccount.value) return
  if (selectedAccount.value.is_system) return
  const ids = checkedValues.map((v) => Number(v))
  await accountStore.setOwners(selectedAccount.value.id, ids)
}

watch(
  () => accountStore.accounts,
  () => {
    treeData.value = buildTreeData()
    // 自动展开根节点（1级）
    const roots = treeData.value
    expandedKeys.value = roots.map((r) => r.key)
  },
  { deep: true }
)

onMounted(() => {
  accountStore.fetchAccounts()
  memberStore.fetchMembers()
})
</script>

<style scoped>
.account-tree {
  max-width: 800px;
  margin: 0 auto;
}

.tree {
  background: #fff;
  padding: 16px;
  border-radius: 8px;
  font-size: 15px;
}

.node-title {
  margin-right: 4px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}

.node-title.active {
  background: #e6f7ff;
  color: #1890ff;
}

.node-title.system {
  text-decoration: underline;
}

.add-btn {
  opacity: 0;
  transition: opacity 0.2s;
  padding: 0 4px;
  height: 20px;
  line-height: 20px;
}

:deep(.ant-tree-node-content-wrapper:hover) .add-btn {
  opacity: 1;
}

.inline-input {
  width: 120px;
}

.new-node-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.detail-panel {
  margin-top: 24px;
  background: #fff;
  padding: 24px;
  border-radius: 8px;
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

.owners-row {
  align-items: flex-start;
}

.detail-label {
  width: 60px;
  color: #666;
  font-weight: 500;
  flex-shrink: 0;
}
</style>
