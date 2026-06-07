<template>
  <div class="manage-page">
    <a-tabs v-model:activeKey="activeTab">
      <a-tab-pane key="tags" tab="标签">
        <div class="create-bar">
          <a-input
            v-model:value="newTagName"
            placeholder="新标签名称"
            @press-enter="handleCreateTag"
          />
          <a-input
            v-model:value="newTagDesc"
            placeholder="描述（可选）"
            @press-enter="handleCreateTag"
          />
          <a-button type="primary" @click="handleCreateTag">创建</a-button>
        </div>

        <a-list :data-source="tags" bordered>
          <template #renderItem="{ item }">
            <a-list-item>
              <div class="item-row">
                <span class="item-name">{{ item.name }}</span>
                <span v-if="item.description" class="item-desc">{{ item.description }}</span>
                <a-tag v-if="item.is_system" color="orange">系统</a-tag>
                <a-button
                  v-if="!item.is_system"
                  size="small"
                  danger
                  @click="handleDeleteTag(item.id)"
                >
                  删除
                </a-button>
              </div>
            </a-list-item>
          </template>
        </a-list>
      </a-tab-pane>

      <a-tab-pane key="channels" tab="渠道">
        <div class="create-bar">
          <a-input
            v-model:value="newChannelName"
            placeholder="新渠道名称"
            @press-enter="handleCreateChannel"
          />
          <a-input
            v-model:value="newChannelDesc"
            placeholder="描述（可选）"
            @press-enter="handleCreateChannel"
          />
          <a-button type="primary" @click="handleCreateChannel">创建</a-button>
        </div>

        <a-list :data-source="channels" bordered>
          <template #renderItem="{ item }">
            <a-list-item>
              <div class="item-row">
                <span class="item-name">{{ item.name }}</span>
                <span v-if="item.description" class="item-desc">{{ item.description }}</span>
                <a-button size="small" danger @click="handleDeleteChannel(item.id)">
                  删除
                </a-button>
              </div>
            </a-list-item>
          </template>
        </a-list>
      </a-tab-pane>
    </a-tabs>
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

interface Channel {
  id: number
  name: string
  description: string | null
}

const activeTab = ref('tags')

const tags = ref<Tag[]>([])
const newTagName = ref('')
const newTagDesc = ref('')

const channels = ref<Channel[]>([])
const newChannelName = ref('')
const newChannelDesc = ref('')

async function fetchTags() {
  try {
    const res = await api.get<Tag[]>('/tags')
    tags.value = res.data
  } catch (e) {
    console.error('获取标签失败', e)
  }
}

async function handleCreateTag() {
  const name = newTagName.value.trim()
  if (!name) return
  try {
    await api.post('/tags', {
      name,
      description: newTagDesc.value.trim() || undefined,
    })
    newTagName.value = ''
    newTagDesc.value = ''
    await fetchTags()
  } catch (e) {
    console.error('创建标签失败', e)
  }
}

async function handleDeleteTag(id: number) {
  try {
    await api.delete(`/tags/${id}`)
    await fetchTags()
  } catch (e) {
    console.error('删除标签失败', e)
  }
}

async function fetchChannels() {
  try {
    const res = await api.get<Channel[]>('/channels')
    channels.value = res.data
  } catch (e) {
    console.error('获取渠道失败', e)
  }
}

async function handleCreateChannel() {
  const name = newChannelName.value.trim()
  if (!name) return
  try {
    await api.post('/channels', {
      name,
      description: newChannelDesc.value.trim() || undefined,
    })
    newChannelName.value = ''
    newChannelDesc.value = ''
    await fetchChannels()
  } catch (e) {
    console.error('创建渠道失败', e)
  }
}

async function handleDeleteChannel(id: number) {
  try {
    await api.delete(`/channels/${id}`)
    await fetchChannels()
  } catch (e) {
    console.error('删除渠道失败', e)
  }
}

onMounted(() => {
  fetchTags()
  fetchChannels()
})
</script>

<style scoped>
.manage-page {
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

.item-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.item-name {
  font-size: 16px;
  font-weight: 500;
}

.item-desc {
  color: #999;
  flex: 1;
}
</style>
