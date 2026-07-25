<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { paneNames, useResponsiveLayout } from '../../composables/useResponsiveLayout'
import { ringDist, useWheelScroll } from '../../composables/useWheelScroll'
import AccountsView from '../../views/AccountsView.vue'
import AssetsView from '../../views/AssetsView.vue'
import BudgetView from '../../views/BudgetView.vue'
import CalendarView from '../../views/CalendarView.vue'
import TransactionView from '../../views/TransactionView.vue'
import ConfigPanel from './ConfigPanel.vue'
import PageSwitcher from './PageSwitcher.vue'
import ViewPanel from './ViewPanel.vue'

const { width, columns, isMobile, paneLabels } = useResponsiveLayout()
const { scrollPos, beginDrag, updateDrag, endDrag } = useWheelScroll()

const componentMap: Record<string, Component> = {
  transaction: TransactionView,
  assets: AssetsView,
  accounts: AccountsView,
  calendar: CalendarView,
  budget: BudgetView,
}

const paneWidth = computed(() => width.value / columns.value)

function paneStyle(index: number) {
  const cols = columns.value
  const leftmost = Math.floor((cols - 1) / 2)
  const panePos = ringDist(index, scrollPos.value) + leftmost
  return {
    width: `${paneWidth.value}px`,
    transform: `translateX(${panePos * paneWidth.value}px)`,
  }
}

const configVisible = ref(false)

function onOpenConfig() {
  configVisible.value = true
}

function onTouchStart(event: TouchEvent) {
  if (!isMobile.value) return
  beginDrag(event.touches[0].clientX, paneWidth.value)
}

function onTouchMove(event: TouchEvent) {
  updateDrag(event.touches[0].clientX)
}
</script>

<template>
  <div class="shell">
    <PageSwitcher @open-config="onOpenConfig" />

    <ConfigPanel v-if="configVisible" @close="configVisible = false" />

    <div class="viewport" @touchstart="onTouchStart" @touchmove="onTouchMove" @touchend="endDrag">
      <div class="track">
        <div v-for="(pane, index) in paneNames" :key="pane" class="pane" :style="paneStyle(index)">
          <ViewPanel :title="paneLabels[pane]">
            <component :is="componentMap[pane]" />
          </ViewPanel>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shell {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
  color: var(--text);
}

.viewport {
  flex: 1;
  overflow: hidden;
  touch-action: pan-y;
}

.track {
  position: relative;
  height: 100%;
}

.pane {
  position: absolute;
  top: 0;
  height: 100%;
  min-width: 0;
  padding: 0 0.5rem;
  will-change: transform;
}
</style>
