<script setup lang="ts">
import { useElementSize } from '@vueuse/core'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { paneLabels, paneNames, useResponsiveLayout } from '../../composables/useResponsiveLayout'
import { ringDist, useWheelScroll } from '../../composables/useWheelScroll'

const emit = defineEmits<{
  openConfig: []
}>()

const { columns, isMobile } = useResponsiveLayout()
const { scrollPos, isDragging, dragMoved, beginDrag, updateDrag, endDrag, stepBy, spinTo } =
  useWheelScroll()

const GAP = 4
const MIN_SCALE = 0.85
const SINK_PX = 4

const trackRef = ref<HTMLElement | null>(null)
const { width: trackWidth } = useElementSize(trackRef)

const labels = computed(() => paneNames.map(name => paneLabels[name]))

const labelRefs = ref<HTMLElement[]>([])
const labelWidths = ref<number[]>([])
const measured = ref(false)

function setLabelRef(el: unknown, index: number) {
  if (el) labelRefs.value[index] = el as HTMLElement
}

function measure() {
  labelWidths.value = labelRefs.value.map(el => el?.offsetWidth ?? 0)
  measured.value = true
}

onMounted(() => nextTick(measure))
watch(labels, () => nextTick(measure))

const slotWidth = computed(() => {
  const widest = labelWidths.value.length > 0 ? Math.max(...labelWidths.value) : 0
  return widest + GAP
})

const windowWidth = computed(() => columns.value * slotWidth.value - GAP)

const highlightStyle = computed(() => ({
  left: `${(trackWidth.value - windowWidth.value) / 2}px`,
  width: `${windowWidth.value}px`,
}))

const labelItems = computed(() => {
  const center = trackWidth.value / 2
  const slot = slotWidth.value
  const cols = columns.value
  const windowHalf = windowWidth.value / 2
  const fadeZone = Math.max(1, (trackWidth.value - windowWidth.value) / 2)
  const leftmost = Math.floor((cols - 1) / 2)

  return labels.value.map((label, index) => {
    const panePos = ringDist(index, scrollPos.value) + leftmost
    const x = center + (panePos - (cols - 1) / 2) * slot
    const distFromCenter = Math.abs(x - center)
    const progress = Math.max(0, (distFromCenter - windowHalf) / fadeZone)
    const clamped = Math.min(progress, 1)
    return {
      label,
      index,
      active: panePos >= -0.01 && panePos < cols - 0.01,
      style: {
        transform: `translateX(${x}px) translate(-50%, -50%) translateY(${SINK_PX * clamped}px) scale(${1 - (1 - MIN_SCALE) * clamped})`,
        opacity: (1 - clamped) * (1 - clamped),
        visibility: measured.value && progress < 1 ? ('visible' as const) : ('hidden' as const),
      },
    }
  })
})

function onMouseDown(event: MouseEvent) {
  if (isMobile.value) return
  beginDrag(event.clientX, slotWidth.value)
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging.value) return
  updateDrag(event.clientX)
}

function onTouchStart(event: TouchEvent) {
  beginDrag(event.touches[0].clientX, slotWidth.value)
}

function onTouchMove(event: TouchEvent) {
  if (!isDragging.value) return
  updateDrag(event.touches[0].clientX)
}

function onLabelClick(index: number) {
  if (dragMoved.value) return
  spinTo(index)
}
</script>

<template>
  <header class="page-switcher">
    <button v-if="!isMobile" type="button" class="arrow-btn" @click="stepBy(-1)">‹</button>

    <div
      ref="trackRef"
      class="switcher-track"
      :class="{ dragging: isDragging }"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="endDrag"
      @mouseleave="endDrag"
      @touchstart="onTouchStart"
      @touchmove="onTouchMove"
      @touchend="endDrag"
    >
      <div class="highlight-box" :style="highlightStyle" />
      <button
        v-for="item in labelItems"
        :key="item.index"
        :ref="(el: unknown) => setLabelRef(el, item.index)"
        type="button"
        class="label-btn"
        :class="{ active: item.active }"
        :style="item.style"
        @click="onLabelClick(item.index)"
      >
        {{ item.label }}
      </button>
    </div>

    <button v-if="!isMobile" type="button" class="arrow-btn" @click="stepBy(1)">›</button>

    <button type="button" class="config-btn" title="⚙" @click="emit('openConfig')">
      <svg
        class="config-icon"
        viewBox="0 0 24 24"
        width="17"
        height="17"
        fill="currentColor"
        aria-hidden="true"
      >
        <path
          d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.48.48 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.7-1.62 1.06l-2.39-.96a.49.49 0 0 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.01-1.58ZM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2Z"
        />
      </svg>
    </button>
  </header>
</template>

<style scoped>
.page-switcher {
  display: flex;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--border);
  background: var(--bg);
  gap: 0.5rem;
}

.arrow-btn {
  width: 2rem;
  height: 2rem;
  font-size: 1.25rem;
  line-height: 1;
  border-radius: 50%;
  border: none;
  background: var(--card-bg);
  color: var(--text-heading);
  cursor: pointer;
  flex-shrink: 0;
}

.arrow-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.switcher-track {
  flex: 1;
  position: relative;
  height: 2.25rem;
  overflow: hidden;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  touch-action: pan-y;
}

.switcher-track.dragging {
  cursor: grabbing;
}

.highlight-box {
  position: absolute;
  top: 2px;
  bottom: 2px;
  background: var(--card-bg);
  border-radius: 0.75rem;
  pointer-events: none;
  z-index: 0;
}

.label-btn {
  position: absolute;
  top: 50%;
  left: 0;
  padding: 0.4rem 0.75rem;
  border-radius: 0.625rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  white-space: nowrap;
  will-change: transform, opacity;
  transition: color 0.2s;
  z-index: 1;
}

.label-btn.active {
  color: var(--text-heading);
  font-weight: 500;
}

.config-btn {
  width: 2rem;
  height: 2rem;
  line-height: 1;
  border-radius: 50%;
  border: none;
  background: var(--card-bg);
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.15s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.config-icon {
  display: block;
}

.config-btn:hover {
  color: var(--text-heading);
}
</style>
