<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <section class="view-panel">
    <h2 class="panel-title">{{ title }}</h2>
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

.panel-title {
  margin: 0;
  padding: 1rem;
  font-size: 1.125rem;
  color: var(--text-heading);
  border-bottom: 1px solid var(--border);
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
