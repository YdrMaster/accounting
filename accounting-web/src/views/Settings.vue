<template>
  <div class="settings">
    <h2>设置</h2>
    <div class="setting-item">
      <label>当前用户</label>
      <a-select
        :value="memberStore.currentMember?.id"
        style="width: 200px"
        placeholder="选择用户"
        @update:value="handleChange"
      >
        <a-select-option
          v-for="member in memberStore.members"
          :key="member.id"
          :value="member.id"
        >
          {{ member.name }}
        </a-select-option>
      </a-select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useMemberStore } from '@/stores/member'

const memberStore = useMemberStore()

function handleChange(value: number) {
  memberStore.setCurrent(value)
}

onMounted(() => {
  memberStore.fetchMembers()
  memberStore.fetchMe()
})
</script>

<style scoped>
.settings {
  max-width: 600px;
  margin: 0 auto;
  background: #fff;
  padding: 24px;
  border-radius: 8px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.setting-item label {
  font-weight: 500;
}
</style>
