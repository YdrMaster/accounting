<template>
  <Teleport to="body">
    <Transition name="bottom-sheet-fade">
      <div v-if="visible" class="bottom-sheet-overlay" @click.self="close">
        <div
          ref="sheetEl"
          class="bottom-sheet"
          :style="sheetStyle"
          @transitionend="handleTransitionEnd"
          @touchstart="handleTouchStart"
          @touchmove="handleTouchMove"
          @touchend="handleTouchEnd"
        >
          <div class="sheet-header" @click.stop>
            <div class="drag-handle" />
            <span class="sheet-title">{{ title }}</span>
            <button class="sheet-close" @click="close">×</button>
          </div>
          <div class="sheet-body">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
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

const visible = ref(false)
const sheetEl = ref<HTMLElement | null>(null)
const dragOffset = ref(0)
const isDragging = ref(false)
const isOpening = ref(false)
const isClosing = ref(false)
const startY = ref(0)
const lastY = ref(0)
const lastTime = ref(0)
const velocity = ref(0)

const sheetStyle = computed(() => ({
  transform: `translateY(${Math.max(0, dragOffset.value)}px)`,
  transition: isDragging.value
    ? 'none'
    : isOpening.value
      ? 'transform 0.25s ease-out'
      : 'transform 0.2s ease-in',
}))

watch(() => props.open, (open) => {
  if (open) {
    isClosing.value = false
    isOpening.value = true
    dragOffset.value = sheetEl.value?.offsetHeight ?? window.innerHeight
    visible.value = true
    nextTick(() => {
      requestAnimationFrame(() => {
        dragOffset.value = 0
      })
    })
    setTimeout(() => {
      isOpening.value = false
    }, 250)
  } else {
    close()
  }
})

function close() {
  if (!visible.value) return
  isClosing.value = true
  isDragging.value = false
  isOpening.value = false
  dragOffset.value = sheetEl.value?.offsetHeight ?? window.innerHeight
}

function handleTransitionEnd(e: TransitionEvent) {
  if (e.propertyName === 'transform' && isClosing.value) {
    isClosing.value = false
    visible.value = false
    emit('update:open', false)
  }
}

function handleTouchStart(e: TouchEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.sheet-header')) return
  isClosing.value = false
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
  dragOffset.value = Math.max(0, y - startY.value)
  lastY.value = y
  lastTime.value = now
}

function handleTouchEnd() {
  if (!isDragging.value) return
  isDragging.value = false
  const threshold = (sheetEl.value?.offsetHeight ?? 0) * 0.4
  if (dragOffset.value > threshold || velocity.value > 0.5) {
    close()
  } else {
    isClosing.value = false
    dragOffset.value = 0
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

.bottom-sheet-fade-enter-active,
.bottom-sheet-fade-leave-active {
  transition: opacity 0.25s ease;
}
.bottom-sheet-fade-enter-from,
.bottom-sheet-fade-leave-to {
  opacity: 0;
}
</style>
