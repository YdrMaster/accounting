<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BudgetView from './BudgetView.vue'
import SavingPlanView from './SavingPlanView.vue'

type Tab = 'budget' | 'savingPlan'

const { t } = useI18n()
const activeTab = ref<Tab>('budget')
</script>

<template>
  <div class="plans-view">
    <div class="tab-bar">
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'budget' }"
        @click="activeTab = 'budget'"
      >
        {{ t('nav.budget') }}
      </button>
      <button
        class="tab-btn"
        :class="{ active: activeTab === 'savingPlan' }"
        @click="activeTab = 'savingPlan'"
      >
        {{ t('nav.savingPlan') }}
      </button>
    </div>

    <div class="tab-body">
      <BudgetView v-if="activeTab === 'budget'" />
      <SavingPlanView v-else />
    </div>
  </div>
</template>

<style scoped>
.plans-view {
  position: relative;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tab-bar {
  display: flex;
  gap: 0;
  padding: 0 0.5rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.tab-btn {
  padding: 0.5rem 1rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.tab-btn.active {
  color: var(--text-heading);
  border-bottom-color: var(--accent);
}

.tab-body {
  position: relative;
  flex: 1;
  min-height: 0;
}
</style>
