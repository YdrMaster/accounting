<template>
  <div class="members-page">
    <h2>成员</h2>

    <div class="create-bar">
      <a-input
        v-model:value="newName"
        placeholder="新成员名称"
        @press-enter="handleCreate"
      />
      <a-button type="primary" @click="handleCreate">创建</a-button>
    </div>

    <a-list :data-source="memberStore.members" bordered>
      <template #renderItem="{ item }">
        <a-list-item>
          <div class="member-row">
            <span class="member-name">{{ item.name }}</span>
            <a-tag v-if="isCurrent(item.id)" color="blue">当前成员</a-tag>
            <a-space>
              <a-button
                v-if="!isCurrent(item.id)"
                size="small"
                @click="handleSetCurrent(item.id)"
              >
                设为当前
              </a-button>
              <a-button
                v-if="!isCurrent(item.id)"
                size="small"
                danger
                @click="handleDelete(item.id)"
              >
                删除
              </a-button>
            </a-space>
          </div>
        </a-list-item>
      </template>
    </a-list>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '@/api/client'
import { useMemberStore } from '@/stores/member'

const memberStore = useMemberStore()
const newName = ref('')

function isCurrent(id: number) {
  return memberStore.currentMember?.id === id
}

async function handleCreate() {
  const name = newName.value.trim()
  if (!name) return
  try {
    await api.post('/members', { name })
    newName.value = ''
    await memberStore.fetchMembers()
  } catch (e) {
    console.error('创建成员失败', e)
  }
}

async function handleSetCurrent(id: number) {
  await memberStore.setCurrent(id)
}

async function handleDelete(id: number) {
  try {
    await api.delete(`/members/${id}`)
    await memberStore.fetchMembers()
    await memberStore.fetchMe()
  } catch (e) {
    console.error('删除成员失败', e)
  }
}

onMounted(() => {
  memberStore.fetchMembers()
  memberStore.fetchMe()
})
</script>

<style scoped>
.members-page {
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

.member-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.member-name {
  flex: 1;
  font-size: 16px;
}
</style>
