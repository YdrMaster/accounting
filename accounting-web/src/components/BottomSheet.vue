<template>
  <Teleport to="body">
    <div
      v-if="mounted"
      class="bottom-sheet-overlay"
      :style="overlayStyle"
      @click.self="close"
    >
      <div
        ref="sheetEl"
        class="bottom-sheet"
        :style="sheetStyle"
        @touchstart="handleTouchStart"
        @touchmove="handleTouchMove"
        @touchend="handleTouchEnd"
      >
        <div class="sheet-header" @click.stop>
          <div class="drag-handle" />
          <span class="sheet-title">{{ title }}</span>
          <button type="button" class="sheet-close" @click="close">×</button>
        </div>
        <div class="sheet-body">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps<{
  open: boolean
  title?: string
}>()

const emit = defineEmits<{
  'update:open': [open: boolean]
}>()

const mounted = ref(false)
const progress = ref(0)
const sheetEl = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const startY = ref(0)
const lastY = ref(0)
const lastTime = ref(0)
const velocity = ref(0)
const closeTimer = ref<number | null>(null)

const sheetHeight = computed(() => sheetEl.value?.offsetHeight ?? 0)

const sheetStyle = computed(() => ({
  transform: `translateY(${(1 - progress.value) * sheetHeight.value}px)`,
  transition: isDragging.value ? 'none' : 'transform 0.25s ease-out',
}))

const overlayStyle = computed(() => ({
  opacity: progress.value,
  transition: isDragging.value ? 'none' : 'opacity 0.25s ease-out',
  pointerEvents: (progress.value > 0 ? 'auto' : 'none') as 'auto' | 'none',
}))

watch(() => props.open, (open) => {
  if (closeTimer.value) {
    clearTimeout(closeTimer.value)
    closeTimer.value = null
  }
  if (open) {
    mounted.value = true
    nextTick(() => {
      requestAnimationFrame(() => {
        progress.value = 1
      })
    })
  } else {
    progress.value = 0
    closeTimer.value = window.setTimeout(() => {
      mounted.value = false
      closeTimer.value = null
    }, 250)
  }
}, { immediate: true })

function close() {
  progress.value = 0
  emit('update:open', false)
}

function handleTouchStart(e: TouchEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.sheet-header')) return
  if (closeTimer.value) {
    clearTimeout(closeTimer.value)
    closeTimer.value = null
  }
  isDragging.value = true
  startY.value = e.touches[0].clientY
  lastY.value = startY.value
  lastTime.value = Date.now()
  velocity.value = 0
}

function handleTouchMove(e: TouchEvent) {
  if (!isDragging.value) return
  const y = e.touches[0].clientY
  const now = Date.now()
  const dt = now - lastTime.value
  if (dt > 0) {
    velocity.value = (y - lastY.value) / dt
  }
  const offset = Math.max(0, y - startY.value)
  progress.value = Math.max(0, 1 - offset / sheetHeight.value)
  lastY.value = y
  lastTime.value = now
}

function handleTouchEnd() {
  if (!isDragging.value) return
  isDragging.value = false
  if (progress.value < 0.6 || velocity.value > 0.5) {
    close()
  } else {
    progress.value = 1
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (closeTimer.value) clearTimeout(closeTimer.value)
})
</script>

<style scoped>
.bottom-sheet-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 1000;
}
.bottom-sheet {
  width: 100%;
  max-width: 600px;
  max-height: 70vh;
  min-height: 160px;
  background: #fff;
  border-top: 1px solid #d9d9d9;
  border-radius: 12px 12px 0 0;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.06);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
@media (max-width: 600px) {
  .bottom-sheet {
    width: calc(100% - 32px);
    margin: 0 16px;
  }
}
.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
  position: relative;
}
.drag-handle {
  position: absolute;
  top: 6px;
  left: 50%;
  transform: translateX(-50%);
  width: 36px;
  height: 4px;
  background: #d9d9d9;
  border-radius: 2px;
}
.sheet-title {
  font-weight: 600;
  margin-top: 8px;
}
.sheet-close {
  margin-top: 8px;
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
}
.sheet-close:hover {
  color: #333;
}
.sheet-body {
  padding: 12px 16px;
  overflow-y: auto;
  flex: 1;
}
</style>
