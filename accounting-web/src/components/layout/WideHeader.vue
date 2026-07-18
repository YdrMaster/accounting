<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  leftDisabled: boolean
  rightDisabled: boolean
  showControls: boolean
}>()

const emit = defineEmits<{
  left: []
  right: []
}>()

const { t } = useI18n()
</script>

<template>
  <header class="wide-header">
    <button v-if="showControls" type="button" :disabled="leftDisabled" @click="emit('left')">
      ‹
    </button>
    <span v-else class="spacer"></span>

    <h1>{{ t('header.title') }}</h1>

    <button v-if="showControls" type="button" :disabled="rightDisabled" @click="emit('right')">
      ›
    </button>
    <span v-else class="spacer"></span>
  </header>
</template>

<style scoped>
.wide-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  background: var(--bg);
}

.wide-header h1 {
  margin: 0;
  font-size: 1.25rem;
  color: var(--text-heading);
}

.wide-header button {
  width: 2.5rem;
  height: 2.5rem;
  font-size: 1.5rem;
  line-height: 1;
  border-radius: 50%;
  border: none;
  background: var(--card-bg);
  color: var(--text-heading);
  cursor: pointer;
}

.wide-header button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.spacer {
  width: 2.5rem;
}
</style>
