<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BalanceSheetPanel from '../components/BalanceSheetPanel.vue'
import CashFlowPanel from '../components/CashFlowPanel.vue'

type Tab = 'balance' | 'cashflow'

const { t } = useI18n()
const activeTab = ref<Tab>('balance')
</script>

<template>
  <div class="assets">
    <div class="tabs">
      <button :class="{ active: activeTab === 'balance' }" @click="activeTab = 'balance'">
        {{ t('assets.tabs.balanceSheet') }}
      </button>
      <button :class="{ active: activeTab === 'cashflow' }" @click="activeTab = 'cashflow'">
        {{ t('assets.tabs.cashFlow') }}
      </button>
    </div>

    <BalanceSheetPanel v-if="activeTab === 'balance'" />
    <CashFlowPanel v-else />
  </div>
</template>

<style scoped>
.assets {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.tabs {
  display: flex;
  gap: 0.5rem;
}

.tabs button {
  flex: 1;
  padding: 0.5rem;
  background: var(--card-bg-alt);
  color: var(--text-muted);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  font-size: 0.875rem;
  cursor: pointer;
}

.tabs button.active {
  color: var(--text-heading);
  border-color: #4ade80;
}
</style>
