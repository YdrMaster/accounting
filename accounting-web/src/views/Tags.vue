<template>
  <div class="tags-page">
    <h2>标签</h2>

    <div class="create-bar">
      <a-input
        v-model:value="newName"
        placeholder="新标签名称"
        @press-enter="handleCreate"
      />
      <a-input
        v-model:value="newDesc"
        placeholder="描述（可选）"
        @press-enter="handleCreate"
      />
      <a-button type="primary" @click="handleCreate">创建</a-button>
    </div>

    <a-list :data-source="tags" bordered>
      <template #renderItem="{ item }">
        <a-list-item>
          <div class="tag-row">
            <span class="tag-name">{{ item.name }}</span>
            <span v-if="item.description" class="tag-desc">{{ item.description }}</span>
            <a-tag v-if="item.is_system" color="orange">系统</a-tag>
            <a-button
              v-if="!item.is_system"
              size="small"
              danger
              @click="handleDelete(item.id)"
            >
              删除
            </a-button>
          </div>
        </a-list-item>
      </template>
    </a-list>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '@/api/client'

interface Tag {
  id: number
  name: string
  description: string | null
  is_system: boolean
}

const tags = ref<Tag[]>([])
const newName = ref('')
const newDesc = ref('')

async function fetchTags() {
  try {
    const res = await api.get<Tag[]>('/tags')
    tags.value = res.data
  } catch (e) {
    console.error('获取标签失败', e)
  }
}

async function handleCreate() {
  const name = newName.value.trim()
  if (!name) return
  try {
    await api.post('/tags', {
      name,
      description: newDesc.value.trim() || undefined,
    })
    newName.value = ''
    newDesc.value = ''
    await fetchTags()
  } catch (e) {
    console.error('创建标签失败', e)
  }
}

async function handleDelete(id: number) {
  try {
    await api.delete(`/tags/${id}`)
    await fetchTags()
  } catch (e) {
    console.error('删除标签失败', e)
  }
}

onMounted(() => {
  fetchTags()
})
</script>

<style scoped>
.tags-page {
  max-width: 600px;
  margin: 0 auto;
}

.create-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.create-bar .ant-input {
  flex: 1;
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.tag-name {
  font-size: 16px;
  font-weight: 500;
}

.tag-desc {
  color: #999;
  flex: 1;
}
</style>
