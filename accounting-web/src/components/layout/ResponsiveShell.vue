<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { paneNames, useResponsiveLayout } from '../../composables/useResponsiveLayout'
import AssetsView from '../../views/AssetsView.vue'
import BudgetView from '../../views/BudgetView.vue'
import CalendarView from '../../views/CalendarView.vue'
import TransactionView from '../../views/TransactionView.vue'
import ViewPanel from './ViewPanel.vue'
import WideHeader from './WideHeader.vue'

const { width, columns, isMobile, startIndex, activeIndex, paneLabels } = useResponsiveLayout()

const componentMap: Record<string, Component> = {
  calendar: CalendarView,
  budget: BudgetView,
  transaction: TransactionView,
  assets: AssetsView,
}

const paneWidth = computed(() => width.value / columns.value)
const showControls = computed(() => !isMobile.value && columns.value < paneNames.length)

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
  transition: isTransitioning.value ? 'transform 0.8s ease' : 'none',
}))

const paneStyle = computed(() => ({
  flex: `0 0 ${paneWidth.value}px`,
}))

let dragStartX = 0

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

function onTouchStart(event: TouchEvent) {
  if (isTransitioning.value) return
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

function onMouseDown(event: MouseEvent) {
  if (!isMobile.value || isTransitioning.value) return
  isDragging.value = true
  dragStartX = event.clientX
  dragOffset.value = 0
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging.value) return
  dragOffset.value = event.clientX - dragStartX
}

function onMouseUp() {
  if (!isDragging.value) return
  onTouchEnd()
}
</script>

<template>
  <div class="shell">
    <WideHeader
      :left-disabled="false"
      :right-disabled="false"
      :show-controls="showControls"
      @left="shiftLeft"
      @right="shiftRight"
    />

    <div
      class="viewport"
      @touchstart="onTouchStart"
      @touchmove="onTouchMove"
      @touchend="onTouchEnd"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
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

    <div v-if="isMobile" class="dot-indicator">
      <span
        v-for="(_, index) in paneNames"
        :key="index"
        :class="{ active: index === activeIndex }"
      />
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

.dot-indicator {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem;
}

.dot-indicator span {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: var(--border);
  transition: background 0.2s;
}

.dot-indicator span.active {
  background: var(--accent);
}
</style>
