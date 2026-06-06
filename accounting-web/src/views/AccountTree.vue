<template>
  <div class="account-tree">
    <div class="toolbar">
      <a-button type="primary" @click="showModal = true">
        <PlusOutlined />
        新建账户
      </a-button>
    </div>

    <a-tree
      :tree-data="treeData"
      :default-expand-all="true"
      class="tree"
    />

    <a-modal
      v-model:open="showModal"
      title="新建账户"
      @ok="handleCreate"
    >
      <a-form layout="vertical">
        <a-form-item label="账户名称" required>
          <a-input v-model:value="newAccountName" placeholder="请输入账户名称" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { PlusOutlined } from '@ant-design/icons-vue'
import { useAccountStore } from '@/stores/account'

interface TreeNode {
  title: string
  key: string
  children: TreeNode[]
}

const accountStore = useAccountStore()
const showModal = ref(false)
const newAccountName = ref('')

const treeData = computed<TreeNode[]>(() => {
  const accounts = accountStore.accounts
  const map = new Map<number, TreeNode>()
  const roots: TreeNode[] = []

  accounts.forEach((acc) => {
    const segments = acc.full_name.split(':')
    map.set(acc.id, {
      title: segments[segments.length - 1],
      key: String(acc.id),
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

  return roots
})

async function handleCreate() {
  if (!newAccountName.value.trim()) return
  await accountStore.createAccount(newAccountName.value.trim())
  showModal.value = false
  newAccountName.value = ''
}

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
</style>
