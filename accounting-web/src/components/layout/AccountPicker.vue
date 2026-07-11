<script setup lang="ts">
import { ref } from 'vue'
import AccountPickerOverlay from './AccountPickerOverlay.vue'
import type { AccountDto } from '../../types/api'

const props = defineProps<{
  modelValue: number | null
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [accountId: number]
}>()

const showOverlay = ref(false)

function onClick() {
  showOverlay.value = true
}

function onClose() {
  showOverlay.value = false
}

function onSelect(account: AccountDto) {
  emit('update:modelValue', account.id)
  showOverlay.value = false
}
</script>

<template>
  <div class="account-picker">
    <button class="picker-trigger" @click="onClick">
      <span v-if="modelValue" class="selected-id">账户 #{{ modelValue }}</span>
      <span v-else class="placeholder">{{ placeholder || '选择账户' }}</span>
    </button>

    <Teleport to=".picker-portal">
      <AccountPickerOverlay
        v-if="showOverlay"
        @close="onClose"
        @select="onSelect"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.account-picker {
  position: relative;
  width: 100%;
}

.picker-trigger {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.15s;
}

.picker-trigger:hover {
  border-color: var(--accent, #646cff);
}

.placeholder {
  color: var(--text-muted);
}

.selected-id {
  color: var(--text-heading);
}
</style>
