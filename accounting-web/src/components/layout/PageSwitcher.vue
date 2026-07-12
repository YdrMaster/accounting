<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  labels: string[]
  activeIndex: number
  visibleCount: number
  startIndex: number
  isMobile: boolean
}>()

const emit = defineEmits<{
  goTo: [index: number]
  left: []
  right: []
  openConfig: []
}>()

const trackRef = ref<HTMLElement | null>(null)
const labelsRowRef = ref<HTMLElement | null>(null)
const highlightRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragOffset = ref(0)
const rowOffset = ref(0)
const isTransitioning = ref(false)
const highlightStyle = ref({ left: '0px', width: '0px' })

// Rotate labels so visible window is centered
const rotatedLabels = computed(() => {
  const len = props.labels.length
  if (len === 0) return []
  const count = Math.min(props.visibleCount, len)
  const start = props.startIndex

  // Calculate offset to center the visible window
  const beforeCount = Math.floor((len - count) / 2)
  const rotateStart = (start - beforeCount + len) % len

  const result: Array<{ label: string; originalIndex: number }> = []
  for (let i = 0; i < len; i++) {
    const idx = (rotateStart + i) % len
    result.push({ label: props.labels[idx], originalIndex: idx })
  }
  return result
})

// Indices that are in the visible window (should be highlighted)
const visibleIndices = computed(() => {
  const len = props.labels.length
  const count = Math.min(props.visibleCount, len)
  const start = props.startIndex
  const indices = new Set<number>()
  for (let i = 0; i < count; i++) {
    indices.add((start + i) % len)
  }
  return indices
})

function measureLabels(): number[] {
  const row = labelsRowRef.value
  if (!row) return []
  const children = Array.from(row.children) as HTMLElement[]
  return children.map(el => el.offsetWidth)
}

function updatePosition() {
  const trackEl = trackRef.value
  const row = labelsRowRef.value
  const highlightEl = highlightRef.value
  if (!trackEl || !row || !highlightEl) return

  const widths = measureLabels()
  if (widths.length === 0) return

  const trackWidth = trackEl.clientWidth
  const gap = 4 // matches CSS gap

  const count = Math.min(props.visibleCount, props.labels.length)
  const beforeCount = Math.floor((props.labels.length - count) / 2)

  const visibleWidth =
    widths.slice(beforeCount, beforeCount + count).reduce((a, b) => a + b, 0) +
    Math.max(0, count - 1) * gap

  const visibleStartOffset =
    widths.slice(0, beforeCount).reduce((a, b) => a + b, 0) + Math.max(0, beforeCount) * gap

  const rowTranslate = (trackWidth - visibleWidth) / 2 - visibleStartOffset
  rowOffset.value = rowTranslate

  const highlightLeft = visibleStartOffset + rowTranslate
  highlightStyle.value = {
    left: `${highlightLeft}px`,
    width: `${visibleWidth}px`,
  }
}

async function updateRowPosition() {
  await nextTick()
  updatePosition()
}

onMounted(() => {
  updateRowPosition()
  window.addEventListener('resize', updateRowPosition)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateRowPosition)
})

watch(
  () => props.activeIndex,
  () => {
    isTransitioning.value = true
    updateRowPosition()
    setTimeout(() => {
      isTransitioning.value = false
    }, 300)
  }
)

watch(
  () => props.startIndex,
  () => {
    isTransitioning.value = true
    updateRowPosition()
    setTimeout(() => {
      isTransitioning.value = false
    }, 300)
  }
)

watch(
  () => props.labels.length,
  () => {
    updateRowPosition()
  }
)

watch(
  () => props.visibleCount,
  () => {
    updateRowPosition()
  }
)

function onTrackMouseDown(event: MouseEvent) {
  if (props.isMobile) return
  isDragging.value = true
  dragStartX.value = event.clientX
  dragOffset.value = 0
}

function onTrackTouchStart(event: TouchEvent) {
  isDragging.value = true
  dragStartX.value = event.touches[0].clientX
  dragOffset.value = 0
}

function onTrackMouseMove(event: MouseEvent) {
  if (!isDragging.value) return
  dragOffset.value = event.clientX - dragStartX.value
}

function onTrackTouchMove(event: TouchEvent) {
  if (!isDragging.value) return
  dragOffset.value = event.touches[0].clientX - dragStartX.value
}

function endDrag() {
  if (!isDragging.value) return
  isDragging.value = false

  const threshold = 40
  if (dragOffset.value < -threshold) {
    emit('right')
  } else if (dragOffset.value > threshold) {
    emit('left')
  }
  dragOffset.value = 0
}

function onLabelClick(originalIndex: number) {
  emit('goTo', originalIndex)
}
</script>

<template>
  <header class="page-switcher">
    <button v-if="!isMobile" type="button" class="arrow-btn" @click="emit('left')">‹</button>

    <div
      ref="trackRef"
      class="switcher-track"
      :class="{ dragging: isDragging }"
      @mousedown="onTrackMouseDown"
      @mousemove="onTrackMouseMove"
      @mouseup="endDrag"
      @mouseleave="endDrag"
      @touchstart="onTrackTouchStart"
      @touchmove="onTrackTouchMove"
      @touchend="endDrag"
    >
      <div
        ref="highlightRef"
        class="highlight-box"
        :style="{
          left: highlightStyle.left,
          width: highlightStyle.width,
          transition: isTransitioning
            ? 'left 0.3s ease, width 0.3s ease'
            : isDragging
              ? 'none'
              : 'left 0.3s ease, width 0.3s ease',
        }"
      />
      <div
        ref="labelsRowRef"
        class="labels-row"
        :style="{
          transform: `translateX(${rowOffset + dragOffset}px)`,
          transition: isTransitioning
            ? 'transform 0.3s ease'
            : isDragging
              ? 'none'
              : 'transform 0.3s ease',
        }"
      >
        <button
          v-for="item in rotatedLabels"
          :key="item.originalIndex"
          type="button"
          class="label-btn"
          :class="{ active: visibleIndices.has(item.originalIndex) }"
          @click="onLabelClick(item.originalIndex)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <button v-if="!isMobile" type="button" class="arrow-btn" @click="emit('right')">›</button>

    <button type="button" class="config-btn" @click="emit('openConfig')">⚙</button>
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
  overflow: hidden;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  display: flex;
  align-items: center;
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

.labels-row {
  display: flex;
  gap: 0.25rem;
  position: relative;
  z-index: 1;
  will-change: transform;
}

.label-btn {
  padding: 0.4rem 0.75rem;
  border-radius: 0.625rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  white-space: nowrap;
  transition: color 0.2s;
}

.label-btn.active {
  color: var(--text-heading);
  font-weight: 500;
}

.config-btn {
  width: 2rem;
  height: 2rem;
  font-size: 1rem;
  line-height: 1;
  border-radius: 50%;
  border: none;
  background: var(--card-bg);
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.15s;
}

.config-btn:hover {
  color: var(--text-heading);
}
</style>
