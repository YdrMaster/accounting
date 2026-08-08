<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { dialogState, resolveDialog } from '../utils/dialog'

const { t } = useI18n()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="dialogState.visible"
      class="app-dialog-overlay"
      @click.self="resolveDialog(false)"
    >
      <div class="app-dialog" role="dialog" aria-modal="true">
        <p class="app-dialog-message">{{ dialogState.message }}</p>
        <div class="app-dialog-actions">
          <button
            v-if="dialogState.kind === 'confirm'"
            type="button"
            class="app-dialog-cancel"
            @click="resolveDialog(false)"
          >
            {{ t('common.cancel') }}
          </button>
          <button type="button" class="app-dialog-confirm" @click="resolveDialog(true)">
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.app-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
}

.app-dialog {
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  padding: 1.25rem;
  max-width: 22rem;
  width: calc(100% - 2rem);
  box-shadow: 0 0.5rem 2rem rgba(0, 0, 0, 0.4);
}

.app-dialog-message {
  margin: 0 0 1rem;
  color: var(--text-heading);
  font-size: 0.9375rem;
  line-height: 1.5;
  word-break: break-word;
}

.app-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.app-dialog-cancel,
.app-dialog-confirm {
  padding: 0.375rem 0.875rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
}

.app-dialog-cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
}

.app-dialog-confirm {
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--text-heading);
}
</style>
