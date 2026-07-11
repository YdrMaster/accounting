<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { paneNames, useResponsiveLayout } from '../../composables/useResponsiveLayout'
import AccountsView from '../../views/AccountsView.vue'
import AssetsView from '../../views/AssetsView.vue'
import BudgetView from '../../views/BudgetView.vue'
import CalendarView from '../../views/CalendarView.vue'
import TransactionView from '../../views/TransactionView.vue'
import ConfigPanel from './ConfigPanel.vue'
import PageSwitcher from './PageSwitcher.vue'
import ViewPanel from './ViewPanel.vue'

const { width, columns, isMobile, startIndex, activeIndex, paneLabels, goTo } =
  useResponsiveLayout()

const componentMap: Record<string, Component> = {
  transaction: TransactionView,
  assets: AssetsView,
  accounts: AccountsView,
  calendar: CalendarView,
  budget: BudgetView,
}

const paneWidth = computed(() => width.value / columns.value)

const orderedPanes = computed(() => {
  const len = paneNames.length
  const prev = (startIndex.value - 1 + len) % len
  const rotated = Array.from({ length: len }, (_, index) => paneNames[(prev + index) % len])
  return [...rotated, ...rotated]
})

const trackBase = computed(() => -paneWidth.value)
const targetOffset = ref<number | null>(null)
const isTransitioning = ref(false)
const pendingNewIndex = ref<number | null>(null)
const isDragging = ref(false)
const dragOffset = ref(0)

const trackOffset = computed(() => {
  if (isDragging.value) return trackBase.value + dragOffset.value
  if (targetOffset.value !== null) return targetOffset.value
  return trackBase.value
})

const trackStyle = computed(() => ({
  transform: `translateX(${trackOffset.value}px)`,
  transition: isTransitioning.value ? 'transform 0.3s ease' : 'none',
}))

const paneStyle = computed(() => ({
  flex: `0 0 ${paneWidth.value}px`,
}))

let dragStartX = 0

const configVisible = ref(false)

function onOpenConfig() {
  configVisible.value = true
}

function moveTo(offset: number, newIndex?: number) {
  if (isTransitioning.value) return
  isTransitioning.value = true
  targetOffset.value = offset
  pendingNewIndex.value = newIndex ?? null
}

function onTransitionEnd() {
  if (pendingNewIndex.value !== null) {
    startIndex.value = pendingNewIndex.value
  }
  targetOffset.value = null
  isTransitioning.value = false
  pendingNewIndex.value = null
}

function shiftLeft() {
  const len = paneNames.length
  moveTo(0, (startIndex.value - 1 + len) % len)
}

function shiftRight() {
  const len = paneNames.length
  moveTo(-2 * paneWidth.value, (startIndex.value + 1) % len)
}

function onSwitcherGoTo(index: number) {
  goTo(index)
}

function onTouchStart(event: TouchEvent) {
  if (!isMobile.value || isTransitioning.value) return
  isDragging.value = true
  dragStartX = event.touches[0].clientX
  dragOffset.value = 0
}

function onTouchMove(event: TouchEvent) {
  if (!isDragging.value) return
  dragOffset.value = event.touches[0].clientX - dragStartX
}

function onTouchEnd() {
  if (!isDragging.value) return
  isDragging.value = false

  const threshold = paneWidth.value * 0.15
  if (dragOffset.value < -threshold) {
    shiftRight()
  } else if (dragOffset.value > threshold) {
    shiftLeft()
  } else {
    moveTo(trackBase.value)
  }
}
</script>

<template>
  <div class="shell">
    <PageSwitcher
      :labels="paneNames.map(n => paneLabels[n])"
      :active-index="activeIndex"
      :visible-count="columns"
      :start-index="startIndex"
      :is-mobile="isMobile"
      @go-to="onSwitcherGoTo"
      @left="shiftLeft"
      @right="shiftRight"
      @open-config="onOpenConfig"
    />

    <ConfigPanel v-if="configVisible" @close="configVisible = false" />

    <div
      class="viewport"
      @touchstart="onTouchStart"
      @touchmove="onTouchMove"
      @touchend="onTouchEnd"
    >
      <div class="track" :style="trackStyle" @transitionend="onTransitionEnd">
        <div
          v-for="(pane, index) in orderedPanes"
          :key="`${pane}-${index}`"
          class="pane"
          :style="paneStyle"
        >
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
  display: flex;
  height: 100%;
  will-change: transform;
}

.pane {
  min-width: 0;
  height: 100%;
  padding: 0 0.5rem;
}

.pane:first-child {
  padding-left: 1rem;
}

.pane:last-child {
  padding-right: 1rem;
}
</style>
