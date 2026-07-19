<script setup lang="ts">
import { provide, ref } from 'vue'
import { panelActionKey, type PanelAction } from './panelAction'

defineProps<{
  title: string
}>()

const action = ref<PanelAction | null>(null)
provide(panelActionKey, action)
</script>

<template>
  <section class="view-panel">
    <div class="panel-header">
      <h2 class="panel-title">{{ title }}</h2>
      <button
        v-if="action"
        class="panel-action-btn"
        :disabled="action.disabled"
        @click="action.onClick"
      >
        + {{ action.label }}
      </button>
    </div>
    <div class="panel-body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.view-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  background: var(--card-bg);
  border-radius: 1rem;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 1rem;
  border-bottom: 1px solid var(--border);
}

.panel-title {
  margin: 0;
  font-size: 1.125rem;
  color: var(--text-heading);
}

.panel-action-btn {
  flex-shrink: 0;
  background: var(--accent, #646cff);
  color: #fff;
  border: none;
  border-radius: 0.5rem;
  padding: 0.375rem 0.875rem;
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
}

.panel-action-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.panel-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.panel-body::-webkit-scrollbar {
  display: none;
}
</style>
