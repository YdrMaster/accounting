<template>
  <div class="account-tree">
    <div class="toolbar">
      <a-button type="primary" @click="handleAddRoot">
        <PlusOutlined />
        新建根账户
      </a-button>
    </div>

    <a-tree
      :tree-data="treeData"
      :default-expand-all="true"
      class="tree"
    >
      <template #title="{ title, key }">
        <template v-if="key === '__new__'">
          <a-input
            ref="newInputRef"
            v-model:value="newAccountName"
            size="small"
            placeholder="输入账户名"
            class="inline-input"
            @press-enter="handleConfirmNew"
            @blur="handleBlurNew"
          />
        </template>
        <template v-else>
          <span class="node-title">{{ title }}</span>
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
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import { PlusOutlined } from '@ant-design/icons-vue'
import { useAccountStore } from '@/stores/account'

interface TreeNode {
  title: string
  key: string
  children: TreeNode[]
}

const accountStore = useAccountStore()
const treeData = ref<TreeNode[]>([])
const addingParentId = ref<number | null>(null)
const newAccountName = ref('')
const newInputRef = ref<HTMLInputElement | null>(null)

function buildTreeData(): TreeNode[] {
  const accounts = accountStore.accounts
  const map = new Map<number, TreeNode>()
  const roots: TreeNode[] = []

  // 先建节点
  accounts.forEach((acc) => {
    const segments = acc.full_name.split(':')
    map.set(acc.id, {
      title: segments[segments.length - 1],
      key: String(acc.id),
      children: [],
    })
  })

  // 再挂父子关系
  accounts.forEach((acc) => {
    const node = map.get(acc.id)
    if (!node) return
    if (acc.parent_id && map.has(acc.parent_id)) {
      map.get(acc.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  // 插入临时输入框节点
  if (addingParentId.value !== null) {
    const parentNode = findNode(roots, String(addingParentId.value))
    if (parentNode) {
      parentNode.children.push({
        title: '__new__',
        key: '__new__',
        children: [],
      })
    } else {
      // 根级别新建
      roots.push({
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

function handleAddRoot() {
  addingParentId.value = -1 // -1 表示根级别
  newAccountName.value = ''
  treeData.value = buildTreeData()
  nextTick(() => {
    newInputRef.value?.focus()
  })
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
  await doCreate()
}

async function handleBlurNew() {
  // 短暂延迟，避免和点击其他按钮冲突
  await new Promise((r) => setTimeout(r, 100))
  await doCreate()
}

async function doCreate() {
  const name = newAccountName.value.trim()
  if (!name) {
    addingParentId.value = null
    treeData.value = buildTreeData()
    return
  }

  let fullName = name
  if (addingParentId.value !== null && addingParentId.value !== -1) {
    const parent = accountStore.accounts.find((a) => a.id === addingParentId.value)
    if (parent) {
      fullName = `${parent.full_name}:${name}`
    }
  }

  addingParentId.value = null
  newAccountName.value = ''
  await accountStore.createAccount(fullName)
}

watch(
  () => accountStore.accounts,
  () => {
    treeData.value = buildTreeData()
  },
  { deep: true }
)

onMounted(() => {
  accountStore.fetchAccounts()
})
</script>

<style scoped>
.account-tree {
  max-width: 800px;
  margin: 0 auto;
}

.toolbar {
  margin-bottom: 16px;
}

.tree {
  background: #fff;
  padding: 16px;
  border-radius: 8px;
}

.node-title {
  margin-right: 4px;
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
  width: 160px;
}
</style>
