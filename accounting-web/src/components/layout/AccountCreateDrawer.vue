<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { createAccount } from '../../api/client'
import type { AccountDto } from '../../types/api'

const { t } = useI18n()

const props = defineProps<{
  parentAccount: AccountDto
}>()

const emit = defineEmits<{
  close: []
  created: []
}>()

const name = ref('')
const error = ref<string | null>(null)
const submitting = ref(false)

async function handleSubmit() {
  if (!name.value.trim()) {
    error.value = t('createDrawer.nameRequired')
    return
  }
  submitting.value = true
  error.value = null
  try {
    await createAccount({
      name: name.value.trim(),
      parent_id: props.parentAccount.id,
      owner_ids: [],
    })
    emit('created')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="drawer-container">
    <div class="drawer">
      <div class="drawer-header">
        <div class="drag-handle" />
        <span class="drawer-title">{{ t('createDrawer.title') }}</span>
        <button class="drawer-close" @click="emit('close')">×</button>
      </div>

      <div class="drawer-body">
        <div v-if="error" class="error-banner">{{ error }}</div>

        <div class="field">
          <label>{{ t('createDrawer.nameLabel') }}</label>
          <input v-model="name" type="text" :placeholder="t('createDrawer.namePlaceholder')" />
        </div>

        <div class="field">
          <label>{{ t('createDrawer.parentLabel') }}</label>
          <div class="parent-hint">
            {{ t('createDrawer.parentHint', { name: parentAccount.name }) }}
          </div>
        </div>

        <button class="submit-btn" :disabled="submitting || !name.trim()" @click="handleSubmit">
          {{ submitting ? t('createDrawer.creating') : t('createDrawer.confirmCreate') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drawer-container {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  pointer-events: none;
}

.drawer {
  position: relative;
  max-height: 50vh;
  max-width: 600px;
  margin: 0 auto;
  width: 100%;
  background: var(--card-bg, #1e1e1e);
  border-radius: 1rem 1rem 0 0;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease-out;
  pointer-events: auto;
}

@keyframes slideUp {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}

.drawer-header {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  position: relative;
  border-bottom: 1px solid var(--border);
}

.drag-handle {
  position: absolute;
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
  width: 36px;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
}

.drawer-title {
  margin-top: 0.5rem;
  font-weight: 600;
  color: var(--text-heading);
}

.drawer-close {
  margin-left: auto;
  margin-top: 0.5rem;
  background: none;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  color: var(--text-muted);
}

.drawer-body {
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.error-banner {
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.field label {
  color: var(--text-muted);
  font-size: 0.8125rem;
}

.field input {
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  outline: none;
}

.field input:focus {
  border-color: var(--accent, #646cff);
}

.parent-hint {
  padding: 0.5rem 0.75rem;
  color: var(--text-heading);
  font-size: 0.875rem;
}

.submit-btn {
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  margin-top: 0.5rem;
}

.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
